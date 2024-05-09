use std::{net::UdpSocket, fmt::Debug};

use anyhow::{Result, bail};

use crate::message::{Response, Query};

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

    fn process_query(&self, query: Query) -> Response {
        let response = Response::builder()
            .id(query.id())
            .opcode(query.opcode())
            .recursion_desired(query.recursion_desired())
            .questions(query.questions());

        response.build()
    }

    pub fn serve(&mut self) -> Result<()> {
        let mut buf = [0; 512];

        match self.socket.recv_from(&mut buf) {
            Ok((size, source)) => {
                println!("Received {} bytes from {}", size, source);
                let response = self.process_query(Query::try_from(&buf[..size])?);
                let resp_vec: Vec<u8> = response.into();
                self.socket
                    .send_to(&resp_vec, source)
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
