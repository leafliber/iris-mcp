mod mouse;
mod keyboard;
mod server;

use std::io;

fn main() -> io::Result<()> {
    server::run_server()
}
