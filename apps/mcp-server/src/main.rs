fn main() -> std::io::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().map(String::as_str) == Some("--cli") {
        let mut stdout = std::io::stdout();
        mcp_server::run_cli(args.into_iter().skip(1), &mut stdout)
    } else {
        mcp_server::run()
    }
}
