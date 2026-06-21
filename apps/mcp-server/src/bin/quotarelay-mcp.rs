fn main() -> std::io::Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.first().map(String::as_str) == Some("--cli") {
        let mut stdout = std::io::stdout();
        mcp_server::run_cli(args.into_iter().skip(1), &mut stdout)
    } else if args.first().map(String::as_str) == Some("--http") {
        let addr = args
            .get(1)
            .map(String::as_str)
            .unwrap_or("127.0.0.1:3030")
            .parse()
            .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidInput, error))?;
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?
            .block_on(mcp_server::run_http(addr))
    } else {
        mcp_server::run()
    }
}
