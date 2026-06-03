use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

use crate::tool_calls::bootstrap_tool_call;
use crate::tools::current_tool_registry;

pub(crate) fn run_stdio<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<()> {
    while let Some(message) = read_message(&mut reader)? {
        if let Some(response) = handle_message(&message) {
            write_message(&mut writer, &response)?;
            writer.flush()?;
        }
    }

    Ok(())
}

fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<String>> {
    let mut content_length = None;
    let mut saw_header = false;

    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            return Ok(None);
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }

        saw_header = true;
        let mut parts = trimmed.splitn(2, ':');
        let name = parts.next().unwrap_or_default().trim();
        let value = parts.next().unwrap_or_default().trim();

        if name.eq_ignore_ascii_case("Content-Length") {
            let parsed = value.parse::<usize>().map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("invalid content length: {error}"),
                )
            })?;
            content_length = Some(parsed);
        }
    }

    if !saw_header {
        return Ok(None);
    }

    let len = content_length.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length header")
    })?;

    let mut body = vec![0_u8; len];
    reader.read_exact(&mut body)?;
    String::from_utf8(body)
        .map(Some)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
}

fn write_message<W: Write>(writer: &mut W, body: &str) -> io::Result<()> {
    write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)
}

fn handle_message(message: &str) -> Option<String> {
    let request: Value = serde_json::from_str(message).ok()?;
    let method = request.get("method")?.as_str()?;
    let id = request.get("id").cloned().unwrap_or(Value::Null);

    let response = match method {
        "initialize" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "serverInfo": {
                    "name": "quotarelay",
                    "version": env!("CARGO_PKG_VERSION")
                },
                "capabilities": {
                    "tools": {}
                }
            }
        }),
        "tools/list" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "tools": current_tool_registry()
            }
        }),
        "tools/call" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [bootstrap_tool_call(&request)]
            }
        }),
        _ => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": {
                "code": -32601,
                "message": "method not found"
            }
        }),
    };

    Some(response.to_string())
}
