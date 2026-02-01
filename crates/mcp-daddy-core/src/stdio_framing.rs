use serde::Serialize;
use serde_json::Value;
use std::io::{self, BufRead, Write};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StdioFramingError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),

    #[error("invalid json line")]
    InvalidJsonLine {
        line: String,
        #[source]
        source: serde_json::Error,
    },

    #[error("json serialization error")]
    JsonSerialize {
        #[source]
        source: serde_json::Error,
    },
}

pub struct JsonRpcLineReader<R> {
    reader: R,
    buf: String,
}

impl<R: BufRead> JsonRpcLineReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            buf: String::new(),
        }
    }

    /// Reads the next newline-delimited JSON-RPC message.
    ///
    /// - Skips empty/whitespace-only lines.
    /// - Returns `Ok(None)` on EOF.
    /// - Returns an error for invalid JSON without panicking.
    pub fn next_message(&mut self) -> Result<Option<Value>, StdioFramingError> {
        loop {
            self.buf.clear();
            let n = self.reader.read_line(&mut self.buf)?;
            if n == 0 {
                return Ok(None);
            }

            let line = self.buf.trim_end_matches(&['\r', '\n'][..]);
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<Value>(line) {
                Ok(v) => return Ok(Some(v)),
                Err(source) => {
                    return Err(StdioFramingError::InvalidJsonLine {
                        line: line.to_string(),
                        source,
                    });
                }
            }
        }
    }
}

pub fn write_jsonrpc_line<W: Write, T: Serialize>(
    writer: &mut W,
    message: &T,
) -> Result<(), StdioFramingError> {
    serde_json::to_writer(&mut *writer, message)
        .map_err(|source| StdioFramingError::JsonSerialize { source })?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::io::{BufReader, Cursor};

    #[test]
    fn reads_multiple_messages_and_continues_after_invalid_json() {
        let input = b"\n{\"jsonrpc\":\"2.0\",\"id\":1}\nnot json\n{\"jsonrpc\":\"2.0\",\"id\":2}\n";
        let mut reader = JsonRpcLineReader::new(BufReader::new(Cursor::new(input)));

        let msg1 = reader.next_message().unwrap().unwrap();
        assert_eq!(msg1["id"], 1);

        let err = reader.next_message().unwrap_err();
        let StdioFramingError::InvalidJsonLine { line, .. } = err else {
            panic!("expected InvalidJsonLine");
        };
        assert_eq!(line, "not json");

        let msg2 = reader.next_message().unwrap().unwrap();
        assert_eq!(msg2["id"], 2);
        assert!(reader.next_message().unwrap().is_none());
    }

    #[test]
    fn reads_last_line_without_trailing_newline() {
        let input = b"{\"jsonrpc\":\"2.0\",\"id\":123}";
        let mut reader = JsonRpcLineReader::new(BufReader::new(Cursor::new(input)));

        let msg = reader.next_message().unwrap().unwrap();
        assert_eq!(msg["id"], 123);
        assert!(reader.next_message().unwrap().is_none());
    }

    #[test]
    fn writes_newline_delimited_json() {
        let mut out = Vec::new();
        write_jsonrpc_line(&mut out, &json!({"jsonrpc": "2.0", "id": 9})).unwrap();
        assert_eq!(out.last().copied(), Some(b'\n'));
        assert_eq!(
            std::str::from_utf8(&out).unwrap(),
            "{\"id\":9,\"jsonrpc\":\"2.0\"}\n"
        );
    }
}
