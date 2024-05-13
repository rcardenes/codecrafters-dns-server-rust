use std::{collections::HashMap, fmt::Debug, net::UdpSocket};

use anyhow::{Result, bail};
use once_cell::sync::Lazy;

use crate::{common::{Name, Record}, message::{Answer, Query, Response}};

static DEFAULT_ADDRESS: &str = "127.0.0.1";
static DEFAULT_PORT: u16 = 2053;
static FAKE_RECORD: Lazy<Record> = Lazy::new(|| {
    Record::from_ip_v4("8.8.8.8").unwrap()
});

#[derive(Debug)]
pub struct ServerBuilder {
    address: String,
    port: u16,
}

impl ServerBuilder {
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
            records: HashMap::new(),
        })
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        ServerBuilder { address: DEFAULT_ADDRESS.into(), port: DEFAULT_PORT }
    }
}

pub struct Server {
    address: String,
    port: u16,
    socket: UdpSocket,
    records: HashMap<Name, Record>,
}

impl Server {
    pub fn new() -> Result<Server> {
        ServerBuilder::default().build()
    }

    fn process_query(&self, query: Query) -> Response {
        let answers = query.questions()
                       .iter()
//                       .flat_map(|q| self.lookup(q.name()).map(|r| (q, r)))
//                       .map(|(q, r)| Answer::new(q.name(), r, 60))
                       .map(|q| { eprintln!("Building answer from {q:?}"); Answer::new(q.name(), &FAKE_RECORD, 60) } )
                       .collect::<Vec<_>>();
        let response = Response::builder()
            .id(query.id())
            .opcode(query.opcode())
            .recursion_desired(query.recursion_desired())
            .questions(query.questions())
            .answers(answers)
            .response_code(query.response_code());

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

    pub fn add_record(&mut self, name: &str, record: Record) {
        self.records.insert(
            Name::from(name.split('.').collect::<Vec<_>>()),
            record
            );
    }

    pub fn lookup(&self, name: &Name) -> Option<&Record> {
        self.records.get(name)
    }
}

impl Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Server<{}, {}>", self.address, self.port)
    }
}
