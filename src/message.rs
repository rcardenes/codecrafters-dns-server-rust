use anyhow::{Result, bail};
use crate::common::{Name, OpCode, QType, QClass};

#[derive(Debug, Default)]
pub enum ResponseCode {
    #[default]
    NoError = 0,
    FormatError,
    ServerFailure,
    NameError,
    NotImplemented,
    Refused,
    Reserved,
}

#[derive(Clone, Debug)]
pub struct Question {
    qname: Name,
    qtype: QType,
    qclass: QClass,
}

impl Question {
    fn to_vec(&self) -> Vec<u8> {
        let mut result = self.qname.to_vec();
        result.extend(u16::to_be_bytes(self.qtype.clone().into()));
        result.extend(u16::to_be_bytes(self.qclass.clone().into()));

        result
    }
}

#[derive(Debug)]
pub struct Query {
    id: u16,
    opcode: OpCode,
    truncation: bool,
    recursion_desired: bool,
    questions: Vec<Question>,
}

impl Query {
    pub fn id(&self) -> u16 {
        self.id
    }

    pub fn opcode(&self) -> OpCode {
        self.opcode.clone()
    }

    pub fn recursion_desired(&self) -> bool {
        self.recursion_desired
    }

    pub fn questions(&self) -> Vec<Question> {
        self.questions.clone()
    }
}

fn decode_name(buffer: &[u8]) -> Result<(Vec<String>, usize)> {
    let length = buffer.len();
    let mut last_pos = 0;
    let mut labels = vec![];

    while last_pos < length {
        let marker = last_pos as usize;
        match buffer[marker..].first().unwrap() {
            0 => { return Ok((labels, last_pos + 1)) },
            64.. => bail!("Corrupt name: label length > 63"),
            label_length => {
                let start = marker + 1;
                let end = start + *label_length as usize;
                if end >= length {
                    bail!("Corrupt name: truncated label")
                }

                let label = String::from_utf8(buffer[start..end].to_vec())?;
                labels.push(label);
                last_pos = end;
            },
        }
    }

    bail!("Corrupt name: no null label terminator")
}

impl TryFrom<&[u8]> for Query {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        if value.len() < 12 {
            bail!("No header! {value:?}");
        }

        if (value[2] & 0x84) != 0x00 || value[3] != 0 {
            bail!("This message is not a valid query: {value:?}");
        }

        let id = u16::from_be_bytes([value[0], value[1]]);
        let opcode = ((value[2] & 78) >> 3).into();
        let mut qdcount = u16::from_be_bytes([value[4], value[5]]);

        let mut ptr = 12;
        let mut questions = vec![];
        while qdcount > 0 {
            if ptr >= value.len() {
                bail!("Truncated message: query count larger than contents");
            }

            if let Some(pos) = value[ptr..].iter().position(|&x| x == 0) {
                let meta = ptr + pos;
                let query_end = meta + 4;

                if query_end >= value.len() {
                    bail!("Corrupt message: truncated query");
                }

                questions.push(Question {
                    qname: Name::try_from(&value[ptr..=pos])?,
                    qtype: QType::try_from(u16::from_be_bytes([value[meta], value[meta + 1]]))?,
                    qclass: QClass::try_from(u16::from_be_bytes([value[meta], value[meta + 1]]))?,
                });

                ptr = query_end + 1;
                qdcount = qdcount - 1;
            } else {
                bail!("Corrupt name: no null label terminator")
            };
        }

        Ok(Query {
            id,
            opcode,
            truncation: value[2] & 0x20 == 0x20,
            recursion_desired: value[2] & 0x10 == 0x10,
            questions,
        })
    }
}

#[derive(Debug)]
pub struct Response {
    id: u16,
    opcode: OpCode,
    truncation: bool,
    authoritative_answer: bool,
    recursion_desired: bool,
    recursion_available: bool,
    response_code: ResponseCode,
    questions: Vec<Question>,
}

impl Response {
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::default()
    }
}

impl Into<Vec<u8>> for Response {
    fn into(self) -> Vec<u8> {
        let oc: u8 = self.opcode.into();
        let aa = if self.authoritative_answer { 4u8 } else { 0 };
        let tc = if self.truncation { 2u8 } else { 0 };
        let rd = if self.recursion_desired { 1u8 } else { 0 };
        let rc = self.response_code as u8;
        let ra = if self.recursion_available { 0x80u8 } else { 0 };
        let qdcount = self.questions.len() as u16;

        let mut res = vec![
            (self.id >> 8) as u8, (self.id & 0xff) as u8,
            (0x80 | oc << 3 | aa | tc | rd) , ra | rc,
            (qdcount & 0x0f) as u8, (qdcount >> 4) as u8,
            0, 0,
            0, 0,
            0, 0,
        ];

        res.extend(self.questions
                   .iter()
                   .map(|q| q.to_vec())
                   .flatten()
                   .collect::<Vec<u8>>());

        res
    }
}

#[derive(Debug, Default)]
pub struct ResponseBuilder {
    id: u16,
    opcode: OpCode,
    truncation: bool,
    authoritative_answer: bool,
    recursion_desired: bool,
    recursion_available: bool,
    response_code: ResponseCode,
    questions: Vec<Question>,
}

impl ResponseBuilder {
    pub fn build(self) -> Response {
        Response {
            id: self.id,
            opcode: self.opcode,
            truncation: self.truncation,
            authoritative_answer: self.authoritative_answer,
            recursion_desired: self.recursion_desired,
            recursion_available: self.recursion_available,
            response_code: self.response_code,
            questions: self.questions,
        }
    }

    pub fn id(mut self, id: u16) -> Self {
        self.id = id;
        self
    }

    pub fn opcode(mut self, opcode: OpCode) -> Self {
        self.opcode = opcode;
        self
    }

    pub fn recursion_desired(mut self, recursion_desired: bool) -> Self {
        self.recursion_desired = recursion_desired;
        self
    }

    pub fn set_truncation(mut self) -> Self {
        self.truncation = true;
        self
    }

    pub fn set_recursion_available(mut self) -> Self {
        self.recursion_available = true;
        self
    }

    pub fn set_authoritative_answer(mut self) -> Self {
        self.authoritative_answer = true;
        self
    }

    pub fn response_code(mut self, response_code: ResponseCode) -> Self {
        self.response_code = response_code;
        self
    }

    pub fn questions(mut self, questions: Vec<Question>) -> Self {
        self.questions = questions;
        self
    }
}
