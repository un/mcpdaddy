use std::collections::{HashMap, VecDeque};
use std::io::{self, BufReader, Read};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[derive(Debug, thiserror::Error)]
pub enum UpstreamProcessError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("failed to spawn upstream process: {command}")]
    Spawn { command: String, source: io::Error },

    #[error("upstream process missing piped stdin")]
    MissingStdin,

    #[error("upstream process missing piped stdout")]
    MissingStdout,

    #[error("upstream process missing piped stderr")]
    MissingStderr,
}

#[derive(Debug, Clone)]
pub struct UpstreamProcessSpec {
    pub command: String,
    pub args: Vec<String>,
    pub env: HashMap<String, String>,
    pub cwd: Option<PathBuf>,
}

impl UpstreamProcessSpec {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: HashMap::new(),
            cwd: None,
        }
    }
}

#[derive(Debug, Default)]
struct StderrCapture {
    // Keep the last N stderr lines; enough for diagnostics without unbounded growth.
    lines: Mutex<VecDeque<String>>,
    max_lines: usize,
}

impl StderrCapture {
    fn push_line(&self, line: String) {
        let mut guard = self.lines.lock().expect("stderr capture mutex poisoned");
        guard.push_back(line);
        while guard.len() > self.max_lines {
            guard.pop_front();
        }
    }

    fn snapshot(&self) -> Vec<String> {
        let guard = self.lines.lock().expect("stderr capture mutex poisoned");
        guard.iter().cloned().collect()
    }
}

pub struct RunningUpstreamProcess {
    child: Child,
    pub stdin: ChildStdin,
    stdout: Option<BufReader<std::process::ChildStdout>>,
    stderr_capture: Arc<StderrCapture>,
    stderr_thread: Option<JoinHandle<()>>,
}

impl RunningUpstreamProcess {
    pub fn spawn(spec: &UpstreamProcessSpec) -> Result<Self, UpstreamProcessError> {
        let mut cmd = Command::new(&spec.command);
        cmd.args(&spec.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(cwd) = &spec.cwd {
            cmd.current_dir(cwd);
        }

        for (k, v) in &spec.env {
            cmd.env(k, v);
        }

        let mut child = cmd.spawn().map_err(|source| UpstreamProcessError::Spawn {
            command: spec.command.clone(),
            source,
        })?;

        let stdin = child
            .stdin
            .take()
            .ok_or(UpstreamProcessError::MissingStdin)?;
        let stdout = child
            .stdout
            .take()
            .ok_or(UpstreamProcessError::MissingStdout)?;
        let stderr = child
            .stderr
            .take()
            .ok_or(UpstreamProcessError::MissingStderr)?;

        let stderr_capture = Arc::new(StderrCapture {
            lines: Mutex::new(VecDeque::new()),
            max_lines: 500,
        });
        let stderr_capture_for_thread = Arc::clone(&stderr_capture);
        let stderr_thread = thread::spawn(move || {
            // Capture stderr as lines, but tolerate non-UTF8 bytes.
            let mut reader = BufReader::new(stderr);
            let mut buf = Vec::<u8>::new();
            loop {
                buf.clear();
                match read_until(&mut reader, b'\n', &mut buf) {
                    Ok(0) => break,
                    Ok(_) => {
                        // Trim trailing \r?\n for stable snapshots.
                        while matches!(buf.last(), Some(b'\n' | b'\r')) {
                            buf.pop();
                        }
                        if buf.is_empty() {
                            continue;
                        }
                        let line = String::from_utf8_lossy(&buf).to_string();
                        stderr_capture_for_thread.push_line(line);
                    }
                    Err(_) => break,
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            stdout: Some(BufReader::new(stdout)),
            stderr_capture,
            stderr_thread: Some(stderr_thread),
        })
    }

    pub fn take_stdout(
        &mut self,
    ) -> Result<BufReader<std::process::ChildStdout>, UpstreamProcessError> {
        self.stdout
            .take()
            .ok_or(UpstreamProcessError::MissingStdout)
    }

    pub fn try_wait(&mut self) -> Result<Option<ExitStatus>, UpstreamProcessError> {
        Ok(self.child.try_wait()?)
    }

    pub fn stderr_lines_snapshot(&self) -> Vec<String> {
        self.stderr_capture.snapshot()
    }

    pub fn stop(&mut self) -> Result<(), UpstreamProcessError> {
        // If already exited, just join stderr thread.
        if self.child.try_wait()?.is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }

        if let Some(handle) = self.stderr_thread.take() {
            let _ = handle.join();
        }

        Ok(())
    }

    pub fn stop_with_timeout(&mut self, timeout: Duration) -> Result<(), UpstreamProcessError> {
        if self.child.try_wait()?.is_some() {
            if let Some(handle) = self.stderr_thread.take() {
                let _ = handle.join();
            }
            return Ok(());
        }

        let _ = self.child.kill();
        let started = Instant::now();
        while started.elapsed() < timeout {
            if self.child.try_wait()?.is_some() {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        let _ = self.child.wait();

        if let Some(handle) = self.stderr_thread.take() {
            let _ = handle.join();
        }
        Ok(())
    }
}

fn read_until<R: Read>(reader: &mut R, delim: u8, buf: &mut Vec<u8>) -> io::Result<usize> {
    let mut total = 0;
    let mut byte = [0u8; 1];
    loop {
        let n = reader.read(&mut byte)?;
        if n == 0 {
            return Ok(total);
        }
        total += 1;
        buf.push(byte[0]);
        if byte[0] == delim {
            return Ok(total);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn spawns_process_and_captures_stderr() {
        let mut spec = UpstreamProcessSpec::new("sh");
        spec.args = vec!["-c".into(), "echo hello-from-stderr 1>&2; sleep 2".into()];

        let mut proc = RunningUpstreamProcess::spawn(&spec).unwrap();

        // Poll briefly for stderr capture to populate.
        let deadline = Instant::now() + Duration::from_millis(500);
        loop {
            let lines = proc.stderr_lines_snapshot();
            if lines.iter().any(|l| l.contains("hello-from-stderr")) {
                break;
            }
            if Instant::now() >= deadline {
                panic!("expected stderr line not captured: {lines:?}");
            }
            thread::sleep(Duration::from_millis(10));
        }

        proc.stop_with_timeout(Duration::from_millis(500)).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn stop_is_ok_when_process_exits_quickly() {
        let mut spec = UpstreamProcessSpec::new("sh");
        spec.args = vec!["-c".into(), "exit 0".into()];
        let mut proc = RunningUpstreamProcess::spawn(&spec).unwrap();
        proc.stop_with_timeout(Duration::from_millis(500)).unwrap();
    }
}
