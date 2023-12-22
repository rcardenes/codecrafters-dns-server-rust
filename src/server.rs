use std::{net::UdpSocket, fmt::Debug};

use anyhow::{Result, bail};

static DEFAULT_ADDRESS: &str = "127.0.0.1";
static DEFAULT_PORT: u16 = 2053;

#[derive(Debug)]
pub struct ServerBuilder {
    address: String,
    port: u16,
}

impl ServerBuilder {
    pub fn default() -> Self {
        ServerBuilder { address: DEFAULT_ADDRESS.into(), port: DEFAULT_PORT }
    }

    pub fn address(mut self, addr: &str) -> Self {
        self.address = String::from(addr);
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn build(self) -> Result<Server> {
        Ok(Server {
            socket: UdpSocket::bind((self.address.as_str(), self.port)).expect("Failed to bind to address"),
            address: self.address,
            port: self.port,
        })
    }
}

pub struct Server {
    address: String,
    port: u16,
    socket: UdpSocket,
}

impl Server {
    pub fn default() -> Result<Server> {
        ServerBuilder::default().build()
    }

    pub fn serve(&mut self) -> Result<()> {
        let mut buf = [0; 512];

        match self.socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                let _received_data = String::from_utf8_lossy(&buf[0..size]);
                println!("Received {} bytes from {}", size, source);
                let response = [];
                self.socket
                    .send_to(&response, source)
                    .expect("Failed to send response");
            }
            Err(e) => {
                bail!("Error receiving data: {}", e);
            }
        }

        Ok(())
    }
}

impl Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Server<{}, {}>", self.address, self.port)
    }
}
