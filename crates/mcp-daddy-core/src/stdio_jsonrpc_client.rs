use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::{json, Value};

use crate::stdio_framing::{write_jsonrpc_line, JsonRpcLineReader, StdioFramingError};
use crate::upstream_process::{RunningUpstreamProcess, UpstreamProcessError};

#[derive(Debug, thiserror::Error)]
pub enum JsonRpcStdioClientError {
    #[error("upstream process error: {0}")]
    Process(#[from] UpstreamProcessError),

    #[error("stdio framing error: {0}")]
    Framing(#[from] StdioFramingError),

    #[error("timeout waiting for response to request {id}")]
    Timeout { id: u64 },

    #[error("upstream stdout closed")]
    StdoutClosed,

    #[error("remote error {code}: {message}")]
    RemoteError {
        code: i64,
        message: String,
        data: Option<Value>,
    },

    #[error("unexpected json-rpc message")]
    UnexpectedMessage,
}

pub struct JsonRpcStdioClient {
    process: RunningUpstreamProcess,
    incoming: Receiver<Result<Value, StdioFramingError>>,
    reader_thread: Option<JoinHandle<()>>,
    next_id: u64,
}

impl JsonRpcStdioClient {
    pub fn new(mut process: RunningUpstreamProcess) -> Result<Self, JsonRpcStdioClientError> {
        let stdout = process.take_stdout()?;

        let (tx, incoming) = mpsc::channel::<Result<Value, StdioFramingError>>();
        let reader_thread = thread::spawn(move || {
            let mut framing = JsonRpcLineReader::new(stdout);
            loop {
                match framing.next_message() {
                    Ok(Some(v)) => {
                        if tx.send(Ok(v)).is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(e) => {
                        let _ = tx.send(Err(e));
                        break;
                    }
                }
            }
        });

        Ok(Self {
            process,
            incoming,
            reader_thread: Some(reader_thread),
            next_id: 1,
        })
    }

    pub fn stderr_lines_snapshot(&self) -> Vec<String> {
        self.process.stderr_lines_snapshot()
    }

    pub fn send_notification(
        &mut self,
        method: &str,
        params: Option<Value>,
    ) -> Result<(), JsonRpcStdioClientError> {
        let msg = match params {
            Some(params) => json!({"jsonrpc": "2.0", "method": method, "params": params}),
            None => json!({"jsonrpc": "2.0", "method": method}),
        };
        write_jsonrpc_line(&mut self.process.stdin, &msg)?;
        Ok(())
    }

    pub fn request(
        &mut self,
        method: &str,
        params: Option<Value>,
        timeout: Duration,
    ) -> Result<Value, JsonRpcStdioClientError> {
        let id = self.next_id;
        self.next_id += 1;

        let msg = match params {
            Some(params) => json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}),
            None => json!({"jsonrpc": "2.0", "id": id, "method": method}),
        };
        write_jsonrpc_line(&mut self.process.stdin, &msg)?;

        self.wait_for_response(id, timeout)
    }

    fn wait_for_response(
        &mut self,
        id: u64,
        timeout: Duration,
    ) -> Result<Value, JsonRpcStdioClientError> {
        let deadline = Instant::now() + timeout;

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(JsonRpcStdioClientError::Timeout { id });
            }

            match self.incoming.recv_timeout(remaining) {
                Ok(Ok(msg)) => {
                    // Ignore notifications.
                    if msg.get("id").is_none() {
                        continue;
                    }
                    if msg.get("id") != Some(&json!(id)) {
                        // MVP: one in-flight request at a time, so ignore anything else.
                        continue;
                    }

                    if let Some(result) = msg.get("result") {
                        return Ok(result.clone());
                    }
                    if let Some(err) = msg.get("error") {
                        let code = err.get("code").and_then(|v| v.as_i64()).unwrap_or(-32000);
                        let message = err
                            .get("message")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown error")
                            .to_string();
                        let data = err.get("data").cloned();
                        return Err(JsonRpcStdioClientError::RemoteError {
                            code,
                            message,
                            data,
                        });
                    }

                    return Err(JsonRpcStdioClientError::UnexpectedMessage);
                }
                Ok(Err(e)) => return Err(JsonRpcStdioClientError::Framing(e)),
                Err(RecvTimeoutError::Timeout) => {
                    return Err(JsonRpcStdioClientError::Timeout { id });
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(JsonRpcStdioClientError::StdoutClosed);
                }
            }
        }
    }

    pub fn stop(&mut self) {
        let _ = self.process.stop_with_timeout(Duration::from_millis(500));
        if let Some(handle) = self.reader_thread.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for JsonRpcStdioClient {
    fn drop(&mut self) {
        self.stop();
    }
}
