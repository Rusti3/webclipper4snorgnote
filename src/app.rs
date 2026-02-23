use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Result, bail};
use serde_json::json;

use crate::bridge::process::{BridgeClient, BridgeProcessConfig};
use crate::deeplink::{parse_clip_uri, write_start_file_from_payload};
use crate::io::end_writer::{EndReport, ImportFailure, write_end_file};
use crate::io::start_parser::parse_start_file;
use crate::logging::AppLogger;

#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub input: PathBuf,
    pub output: PathBuf,
    pub title: String,
    pub sidecar_script: PathBuf,
    pub node_path: PathBuf,
    pub profile_dir: Option<PathBuf>,
    pub browser_path: Option<PathBuf>,
    pub timeout_sec: u64,
}

pub fn run_once(config: &RunnerConfig) -> Result<()> {
    let started_at = Instant::now();
    let logger = AppLogger::new()?;
    logger.info("run started");

    let mut report = EndReport {
        status: "failed".to_string(),
        notebook_title: Some(config.title.clone()),
        notebook_url: None,
        prompt: String::new(),
        answer: None,
        imported: 0,
        failed: Vec::new(),
        errors: Vec::new(),
        duration_ms: 0,
    };

    let parsed_input = match parse_start_file(&config.input) {
        Ok(input) => {
            report.prompt = input.prompt.clone();
            input
        }
        Err(err) => {
            report
                .errors
                .push(format!("Failed to parse start.txt: {err:#}"));
            finalize_run(config, &logger, &mut report, started_at)?;
            bail!("Failed to parse start.txt");
        }
    };

    match BridgeClient::spawn(
        BridgeProcessConfig {
            node_path: config.node_path.clone(),
            sidecar_script: config.sidecar_script.clone(),
            profile_dir: config.profile_dir.clone(),
            browser_path: config.browser_path.clone(),
            timeout_sec: config.timeout_sec,
        },
        logger.clone(),
    ) {
        Ok(mut bridge) => {
            let mut pipeline_ok = true;

            if let Err(err) = bridge.send_command("connect", json!({})) {
                report.errors.push(format!("connect failed: {err:#}"));
                pipeline_ok = false;
            }

            if pipeline_ok {
                match bridge
                    .send_command("create_notebook", json!({ "title": config.title.clone() }))
                {
                    Ok(data) => {
                        report.notebook_title = data
                            .get("title")
                            .and_then(|v| v.as_str())
                            .map(str::to_string)
                            .or_else(|| Some(config.title.clone()));
                        report.notebook_url =
                            data.get("url").and_then(|v| v.as_str()).map(str::to_string);
                    }
                    Err(err) => {
                        report
                            .errors
                            .push(format!("create_notebook failed: {err:#}"));
                        pipeline_ok = false;
                    }
                }
            }

            if pipeline_ok {
                match bridge
                    .send_command("import_urls", json!({ "urls": parsed_input.urls.clone() }))
                {
                    Ok(data) => {
                        report.imported =
                            data.get("imported").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                        report.failed = parse_import_failures(&data);
                    }
                    Err(err) => {
                        report.errors.push(format!("import_urls failed: {err:#}"));
                        pipeline_ok = false;
                    }
                }
            }

            if pipeline_ok {
                match bridge.send_command("ask", json!({ "prompt": parsed_input.prompt.clone() })) {
                    Ok(data) => {
                        report.answer = data
                            .get("answer")
                            .and_then(|v| v.as_str())
                            .map(str::to_string);
                        if report
                            .answer
                            .as_deref()
                            .unwrap_or_default()
                            .trim()
                            .is_empty()
                        {
                            report
                                .errors
                                .push("ask succeeded but returned empty answer".to_string());
                            pipeline_ok = false;
                        }
                    }
                    Err(err) => {
                        report.errors.push(format!("ask failed: {err:#}"));
                        pipeline_ok = false;
                    }
                }
            }

            if let Err(err) = bridge.close() {
                report.errors.push(format!("bridge close failed: {err:#}"));
            }

            if pipeline_ok && report.errors.is_empty() {
                report.status = "ok".to_string();
            }
        }
        Err(err) => {
            report
                .errors
                .push(format!("failed to start bridge process: {err:#}"));
        }
    }

    finalize_run(config, &logger, &mut report, started_at)?;
    if report.status == "ok" {
        Ok(())
    } else {
        bail!("Run failed. See {} for details", config.output.display())
    }
}

pub fn run_from_deeplink(uri: &str, config: &RunnerConfig) -> Result<()> {
    let payload = parse_clip_uri(uri)?;
    write_start_file_from_payload(&config.input, &payload)?;

    let mut run_config = config.clone();
    if let Some(title) = payload.title {
        run_config.title = title;
    }

    run_once(&run_config)
}

fn parse_import_failures(data: &serde_json::Value) -> Vec<ImportFailure> {
    let Some(items) = data.get("failed").and_then(|v| v.as_array()) else {
        return Vec::new();
    };

    items
        .iter()
        .map(|item| ImportFailure {
            url: item
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            reason: item
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
        })
        .collect()
}

fn finalize_run(
    config: &RunnerConfig,
    logger: &AppLogger,
    report: &mut EndReport,
    started_at: Instant,
) -> Result<()> {
    report.duration_ms = started_at.elapsed().as_millis();
    write_end_file(&config.output, report)?;
    logger.info(&format!("run finished with status={}", report.status));
    Ok(())
}
