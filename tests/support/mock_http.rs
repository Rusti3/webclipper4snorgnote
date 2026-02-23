use std::collections::VecDeque;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone)]
pub struct ExpectedRequest {
    pub method: &'static str,
    pub path: String,
    pub response_status: u16,
    pub response_body: String,
    pub response_content_type: &'static str,
}

impl ExpectedRequest {
    pub fn json(
        method: &'static str,
        path: impl Into<String>,
        response_status: u16,
        response_body: impl Into<String>,
    ) -> Self {
        Self {
            method,
            path: path.into(),
            response_status,
            response_body: response_body.into(),
            response_content_type: "application/json; charset=utf-8",
        }
    }

    pub fn empty(method: &'static str, path: impl Into<String>, response_status: u16) -> Self {
        Self {
            method,
            path: path.into(),
            response_status,
            response_body: String::new(),
            response_content_type: "text/plain; charset=utf-8",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReceivedRequest {
    pub method: String,
    pub path: String,
}

pub struct MockHttpServer {
    base_url: String,
    received: Arc<Mutex<Vec<ReceivedRequest>>>,
    join: Option<JoinHandle<Result<()>>>,
}

impl MockHttpServer {
    pub fn start(expected: Vec<ExpectedRequest>) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").context("bind mock server")?;
        listener
            .set_nonblocking(true)
            .context("set_nonblocking mock server")?;
        let addr = listener.local_addr().context("mock server local_addr")?;
        let base_url = format!("http://{}", addr);

        let expected_queue = Arc::new(Mutex::new(VecDeque::from(expected)));
        let received = Arc::new(Mutex::new(Vec::new()));
        let received_for_thread = Arc::clone(&received);
        let expected_for_thread = Arc::clone(&expected_queue);

        let join = thread::spawn(move || -> Result<()> {
            let deadline = Instant::now() + Duration::from_secs(15);

            loop {
                if Instant::now() > deadline {
                    bail!("mock server timed out waiting for expected requests");
                }

                let done = expected_for_thread
                    .lock()
                    .map_err(|_| anyhow::anyhow!("expected queue poisoned"))?
                    .is_empty();
                if done {
                    break;
                }

                match listener.accept() {
                    Ok((stream, _)) => {
                        let request = read_http_request(stream)?;

                        let expected_item = expected_for_thread
                            .lock()
                            .map_err(|_| anyhow::anyhow!("expected queue poisoned"))?
                            .pop_front()
                            .ok_or_else(|| anyhow::anyhow!("received unexpected extra request"))?;

                        if request.method != expected_item.method {
                            bail!(
                                "unexpected method: expected {}, got {}",
                                expected_item.method,
                                request.method
                            );
                        }
                        if request.path != expected_item.path {
                            bail!(
                                "unexpected path: expected {}, got {}",
                                expected_item.path,
                                request.path
                            );
                        }

                        received_for_thread
                            .lock()
                            .map_err(|_| anyhow::anyhow!("received list poisoned"))?
                            .push(ReceivedRequest {
                                method: request.method.clone(),
                                path: request.path.clone(),
                            });

                        write_http_response(
                            request.stream,
                            expected_item.response_status,
                            &expected_item.response_body,
                            expected_item.response_content_type,
                        )?;
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(err) => return Err(err.into()),
                }
            }

            Ok(())
        });

        Ok(Self {
            base_url,
            received,
            join: Some(join),
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    pub fn finish(mut self) -> Result<Vec<ReceivedRequest>> {
        let join = self
            .join
            .take()
            .ok_or_else(|| anyhow::anyhow!("mock server join handle missing"))?;
        let result = join
            .join()
            .map_err(|_| anyhow::anyhow!("mock server panicked"))?;
        result?;
        let received = self
            .received
            .lock()
            .map_err(|_| anyhow::anyhow!("received list poisoned"))?
            .clone();
        Ok(received)
    }
}

struct ParsedRequestWithStream {
    method: String,
    path: String,
    stream: TcpStream,
}

fn read_http_request(mut stream: TcpStream) -> Result<ParsedRequestWithStream> {
    stream
        .set_read_timeout(Some(Duration::from_secs(5)))
        .context("set read timeout")?;

    let mut buffer = Vec::new();
    let mut temp = [0_u8; 1024];
    let mut header_end = None;

    while header_end.is_none() {
        let read = stream.read(&mut temp).context("read request headers")?;
        if read == 0 {
            bail!("unexpected EOF while reading headers");
        }
        buffer.extend_from_slice(&temp[..read]);
        header_end = find_subslice(&buffer, b"\r\n\r\n");
    }

    let header_end_index = header_end.expect("header end index") + 4;
    let header_text = String::from_utf8(buffer[..header_end_index].to_vec())
        .context("headers are not valid utf-8")?;

    let mut lines = header_text.split("\r\n");
    let request_line = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing request line"))?;
    let mut request_line_parts = request_line.split_whitespace();
    let method = request_line_parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing request method"))?
        .to_string();
    let path = request_line_parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("missing request path"))?
        .to_string();

    let mut content_length = 0_usize;
    for line in lines {
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            content_length = value.trim().parse::<usize>().unwrap_or(0);
        }
    }

    let mut body_bytes = buffer[header_end_index..].to_vec();
    while body_bytes.len() < content_length {
        let read = stream.read(&mut temp).context("read request body")?;
        if read == 0 {
            break;
        }
        body_bytes.extend_from_slice(&temp[..read]);
    }
    body_bytes.truncate(content_length);
    Ok(ParsedRequestWithStream {
        method,
        path,
        stream,
    })
}

fn write_http_response(
    mut stream: TcpStream,
    status: u16,
    body: &str,
    content_type: &str,
) -> Result<()> {
    let status_text = match status {
        200 => "OK",
        201 => "Created",
        204 => "No Content",
        400 => "Bad Request",
        404 => "Not Found",
        500 => "Internal Server Error",
        _ => "OK",
    };

    let response = format!(
        "HTTP/1.1 {status} {status_text}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream
        .write_all(response.as_bytes())
        .context("write response")?;
    stream.flush().context("flush response")?;
    Ok(())
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}
