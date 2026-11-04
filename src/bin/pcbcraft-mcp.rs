//! Stdio MCP adapter to an explicitly chosen running PCBcraft window.
fn main() {
    if let Err(e) = run() {
        eprintln!("PCBcraft MCP: {e}");
        std::process::exit(1);
    }
}
fn run() -> std::io::Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.len() != 2 || args[0] != "--connect" {
        return Err(std::io::Error::other(
            "Usage: pcbcraft-mcp --connect /path/from/app/connection.json\nEnable the bridge in PCBcraft's AI / MCP panel. No headless fallback.",
        ));
    }
    let connection = pcbcraft_foundation::bridge::connect(std::path::Path::new(&args[1]))?;
    pcbcraft_foundation::mcp::serve(
        &mut std::io::stdin().lock(),
        &mut std::io::stdout().lock(),
        |call| connection.call(call),
    )
}
