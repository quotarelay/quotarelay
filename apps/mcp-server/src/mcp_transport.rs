use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

use crate::tool_calls::bootstrap_tool_call;
use crate::tools::current_tool_registry;

enum MessageFraming {
    ContentLength,
    JsonLine,
}

struct IncomingMessage {
    body: String,
    framing: MessageFraming,
}

pub(crate) fn run_stdio<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> io::Result<()> {
    while let Some(message) = read_message(&mut reader)? {
        if let Some(response) = handle_message(&message.body) {
            write_message(&mut writer, &response, &message.framing)?;
            writer.flush()?;
        }
    }

    Ok(())
}

fn read_message<R: BufRead>(reader: &mut R) -> io::Result<Option<IncomingMessage>> {
    let mut first_line = String::new();

    loop {
        first_line.clear();
        let read = reader.read_line(&mut first_line)?;
        if read == 0 {
            return Ok(None);
        }

        let trimmed = first_line.trim_end_matches(['\r', '\n']);
        if !trimmed.is_empty() {
            break;
        }
    }

    let trimmed = first_line.trim_end_matches(['\r', '\n']);
    let protocol_line = trimmed.trim_start_matches('\u{feff}').trim_start();
    if protocol_line.starts_with('{') {
        return Ok(Some(IncomingMessage {
            body: protocol_line.to_string(),
            framing: MessageFraming::JsonLine,
        }));
    }

    let mut content_length = parse_content_length_header(trimmed)?;

    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            break;
        }

        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }

        if content_length.is_none() {
            content_length = parse_content_length_header(trimmed)?;
        }
    }

    let len = content_length.ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "missing Content-Length header")
    })?;

    let mut body = vec![0_u8; len];
    reader.read_exact(&mut body)?;
    let body = String::from_utf8(body)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(Some(IncomingMessage {
        body,
        framing: MessageFraming::ContentLength,
    }))
}

fn parse_content_length_header(line: &str) -> io::Result<Option<usize>> {
    let mut parts = line.splitn(2, ':');
    let name = parts
        .next()
        .unwrap_or_default()
        .trim_start_matches('\u{feff}')
        .trim();
    let value = parts.next().unwrap_or_default().trim();

    if !name.eq_ignore_ascii_case("Content-Length") {
        return Ok(None);
    }

    value.parse::<usize>().map(Some).map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid content length: {error}"),
        )
    })
}

fn write_message<W: Write>(writer: &mut W, body: &str, framing: &MessageFraming) -> io::Result<()> {
    match framing {
        MessageFraming::ContentLength => {
            write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)
        }
        MessageFraming::JsonLine => writeln!(writer, "{body}"),
    }
}

fn handle_message(message: &str) -> Option<String> {
    let request: Value = serde_json::from_str(message).ok()?;
    let method = request.get("method")?.as_str()?;
    if request.get("id").is_none() {
        return None;
    }
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
