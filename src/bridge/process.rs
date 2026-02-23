use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

use super::protocol::{BridgeRequest, BridgeResponse};
use crate::logging::AppLogger;

#[derive(Debug, Clone)]
pub struct BridgeProcessConfig {
    pub node_path: PathBuf,
    pub sidecar_script: PathBuf,
    pub profile_dir: Option<PathBuf>,
    pub browser_path: Option<PathBuf>,
    pub timeout_sec: u64,
}

pub struct BridgeClient {
    logger: AppLogger,
    child: Child,
    stdin: ChildStdin,
    stdout_rx: Receiver<String>,
    stdout_thread: Option<JoinHandle<()>>,
    stderr_thread: Option<JoinHandle<()>>,
    next_id: u64,
    timeout: Duration,
    closed: bool,
}

impl BridgeClient {
    pub fn spawn(config: BridgeProcessConfig, logger: AppLogger) -> Result<Self> {
        let mut cmd = Command::new(&config.node_path);
        cmd.arg(&config.sidecar_script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(profile_dir) = &config.profile_dir {
            cmd.arg("--profile-dir").arg(profile_dir);
        }
        if let Some(browser_path) = &config.browser_path {
            cmd.arg("--browser-path").arg(browser_path);
        }

        logger.info(&format!(
            "Spawning sidecar: {} {}",
            config.node_path.display(),
            config.sidecar_script.display()
        ));

        let mut child = cmd.spawn().with_context(|| {
            format!(
                "failed to spawn sidecar process via `{}` using script `{}`",
                config.node_path.display(),
                config.sidecar_script.display()
            )
        })?;

        let stdin = child
            .stdin
            .take()
            .context("failed to acquire sidecar stdin")?;
        let stdout = child
            .stdout
            .take()
            .context("failed to acquire sidecar stdout")?;
        let stderr = child
            .stderr
            .take()
            .context("failed to acquire sidecar stderr")?;

        let (stdout_tx, stdout_rx) = mpsc::channel::<String>();
        let stdout_thread = thread::spawn(move || {
            let reader = std::io::BufReader::new(stdout);
            for line in std::io::BufRead::lines(reader).map_while(std::result::Result::ok) {
                if stdout_tx.send(line).is_err() {
                    break;
                }
            }
        });

        let stderr_logger = logger.clone();
        let stderr_thread = thread::spawn(move || {
            let reader = std::io::BufReader::new(stderr);
            for line in std::io::BufRead::lines(reader).map_while(std::result::Result::ok) {
                stderr_logger.warn(&format!("sidecar stderr: {line}"));
            }
        });

        Ok(Self {
            logger,
            child,
            stdin,
            stdout_rx,
            stdout_thread: Some(stdout_thread),
            stderr_thread: Some(stderr_thread),
            next_id: 1,
            timeout: Duration::from_secs(config.timeout_sec.max(1)),
            closed: false,
        })
    }

    pub fn send_command(&mut self, cmd: &str, payload: Value) -> Result<Value> {
        if self.closed {
            bail!("bridge is already closed");
        }

        let id = self.next_id.to_string();
        self.next_id += 1;
        let request = BridgeRequest::new(id.clone(), cmd, payload);
        let request_text = serde_json::to_string(&request)?;

        use std::io::Write;
        writeln!(self.stdin, "{request_text}")?;
        self.stdin.flush()?;
        self.logger.info(&format!("bridge -> {cmd} (id={id})"));

        let deadline = Instant::now() + self.timeout;
        loop {
            let now = Instant::now();
            if now >= deadline {
                bail!("timeout waiting for response: cmd={cmd}, id={id}");
            }
            let remaining = deadline.saturating_duration_since(now);

            let raw = match self.stdout_rx.recv_timeout(remaining) {
                Ok(line) => line,
                Err(RecvTimeoutError::Timeout) => {
                    bail!("timeout waiting for response: cmd={cmd}, id={id}");
                }
                Err(RecvTimeoutError::Disconnected) => {
                    let status = self.child.try_wait()?.map(|s| s.to_string());
                    bail!("bridge output channel disconnected, sidecar status={status:?}");
                }
            };

            let response: BridgeResponse = match serde_json::from_str(&raw) {
                Ok(parsed) => parsed,
                Err(err) => {
                    self.logger
                        .warn(&format!("invalid sidecar JSON line ignored: {err}: {raw}"));
                    continue;
                }
            };

            if response.is_progress_event() {
                let phase = response.phase.as_deref().unwrap_or("unknown");
                let message = response.message.as_deref().unwrap_or("progress");
                self.logger.info(&format!("progress[{phase}] {message}"));
                continue;
            }

            if response.id.as_deref() != Some(id.as_str()) {
                self.logger.warn(&format!(
                    "ignoring sidecar response for another id: expected={id}, got={:?}",
                    response.id
                ));
                continue;
            }

            let ok = response.ok.unwrap_or(false);
            if ok {
                self.logger.info(&format!("bridge <- ok cmd={cmd} id={id}"));
                return Ok(response.data.unwrap_or_else(|| json!({})));
            }

            let error = response
                .error
                .unwrap_or_else(|| "unknown sidecar error".to_string());
            self.logger
                .error(&format!("bridge <- error cmd={cmd} id={id}: {error}"));
            bail!("{error}");
        }
    }

    pub fn close(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }

        if let Err(err) = self.send_command("close", json!({})) {
            self.logger.warn(&format!("close command failed: {err:#}"));
        }

        self.closed = true;

        if self.child.try_wait()?.is_none() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();

        if let Some(handle) = self.stdout_thread.take() {
            let _ = handle.join();
        }
        if let Some(handle) = self.stderr_thread.take() {
            let _ = handle.join();
        }

        Ok(())
    }
}
