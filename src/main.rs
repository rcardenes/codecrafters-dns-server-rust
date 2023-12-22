use dns_starter_rust::server::Server;

use anyhow::Result;

fn main() -> Result<()> {
    let mut server = Server::default()?;

    eprintln!("{server:?}");

    loop {
        server.serve()?;
    }
}
