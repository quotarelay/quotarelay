use std::io;

mod args;
mod cli;
mod http;
mod mcp_transport;
mod tool_calls;
mod tools;
mod truth;

pub use cli::run_cli;
#[cfg(test)]
pub(crate) use http::http_router;
pub use http::run_http;
pub(crate) use mcp_transport::run_stdio;
pub(crate) use truth::backend_truth_payload;

pub fn run() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    run_stdio(stdin.lock(), stdout.lock())
}

#[cfg(test)]
mod tests;
