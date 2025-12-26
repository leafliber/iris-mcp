use iris_mcp::server;
use std::io;

fn main() -> io::Result<()> {
    server::run_server()
}
