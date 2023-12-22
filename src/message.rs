use anyhow::{Result, bail};

#[derive(Clone, Debug, Default)]
#[repr(u8)]
pub enum OpCode {
    #[default]
    Query = 0,
    IQuery,
    Status,
    Reserved(u8)
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Query,
            1 => Self::IQuery,
            2 => Self::Status,
            other => Self::Reserved(other),
        }
    }
}

impl Into<u8> for OpCode {
    fn into(self) -> u8 {
        match self {
            Self::Query => 0,
            Self::IQuery => 1,
            Self::Status => 2,
            Self::Reserved(value) => value,
        }
    }
}

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

#[derive(Debug)]
pub struct Query {
    id: u16,
    opcode: OpCode,
    truncation: bool,
    recursion_desired: bool,
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
}

impl TryFrom<&[u8]> for Query {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        let l = value.len();
        
        if l < 12 {
            bail!("No header! {value:?}");
        } else if (value[2] & 0x84) != 0x00 || value[3] != 0 {
            bail!("This message is not a valid query: {value:?}");
        }

        let id = u16::from_be_bytes([value[0], value[1]]);
        let opcode = ((value[2] & 78) >> 3).into();


        Ok(Query {
            id,
            opcode,
            truncation: value[2] & 0x20 == 0x20,
            recursion_desired: value[2] & 0x10 == 0x10,
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
}

impl Response {
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::default()
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
}

impl Into<Vec<u8>> for Response {
    fn into(self) -> Vec<u8> {
        let oc: u8 = self.opcode.into();
        let aa = if self.authoritative_answer { 4u8 } else { 0 };
        let tc = if self.truncation { 2u8 } else { 0 };
        let rd = if self.recursion_desired { 1u8 } else { 0 };
        let rc = self.response_code as u8;
        let ra = if self.recursion_available { 0x80u8 } else { 0 };

        let res = vec![
            (self.id >> 8) as u8, (self.id & 0xff) as u8,
            (0x80 | oc << 3 | aa | tc | rd) , ra | rc,
            0, 0,
            0, 0,
            0, 0,
            0, 0,
        ];

        res
    }
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
}
