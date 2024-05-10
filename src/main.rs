use dns_starter_rust::{common::Record, server::Server};

use anyhow::Result;

fn config_server(server: &mut Server) -> Result<()> {
    server.add_record(
        "codecrafters.io",
        Record::from_ip_v4("8.8.8.8")?
        );

    Ok(())
}

fn main() -> Result<()> {
    let mut server = Server::default()?;
    config_server(&mut server)?;

    eprintln!("{server:?}");

    loop {
        match server.serve() {
            Err(err) => eprintln!("{err}"),
            _ => {}
        }
    }
}
