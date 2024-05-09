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


#[derive(Clone, Debug)]
pub enum RRType {
    A,
    NS,
    MD,
    MF,
    CNAME,
    SOA,
    MB,
    MG,
    MR,
    NULL,
    WKS,
    PTR,
    HINFO,
    MINFO,
    MX,
    TXT,
}

impl TryFrom<u16> for RRType {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> Result<Self> {
        Ok(match value {
            1 => RRType::A,
            2 => RRType::NS,
            3 => RRType::MD,
            4 => RRType::MF,
            5 => RRType::CNAME,
            6 => RRType::SOA,
            7 => RRType::MB,
            8 => RRType::MG,
            9 => RRType::MR,
            10 => RRType::NULL,
            11 => RRType::WKS,
            12 => RRType::PTR,
            13 => RRType::HINFO,
            14 => RRType::MINFO,
            15 => RRType::MX,
            16 => RRType::TXT,
            other => bail!("{other} is not a valid Type"),
        })
    }
}

impl Into<u16> for RRType {
    fn into(self) -> u16 {
        match self {
            RRType::A => 1,
            RRType::NS => 2,
            RRType::MD => 3,
            RRType::MF => 4,
            RRType::CNAME => 5,
            RRType::SOA => 6,
            RRType::MB => 7,
            RRType::MG => 8,
            RRType::MR => 9,
            RRType::NULL => 10,
            RRType::WKS => 11,
            RRType::PTR => 12,
            RRType::HINFO => 13,
            RRType::MINFO => 14,
            RRType::MX => 15,
            RRType::TXT => 16,
        }
    }
}

#[derive(Clone, Debug)]
pub enum QType {
    RRType(RRType),
    AXFR,
    MAILB,
    MAILA,
    ANY,
}

impl TryFrom<u16> for QType {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> Result<Self> {
        Ok(match value {
            252 => QType::AXFR,
            253 => QType::MAILB,
            254 => QType::MAILA,
            255 => QType::ANY,
            other => match RRType::try_from(value) {
                Ok(res) => QType::RRType(res),
                Err(_) => bail!("{other} is not a valid QType")
            },
        })
    }
}

impl Into<u16> for QType {
    fn into(self) -> u16 {
        match self {
            QType::RRType(rr) => rr as u16,
            QType::AXFR => 252,
            QType::MAILB => 253,
            QType::MAILA => 254,
            QType::ANY => 255,
        }
    }
}

#[derive(Clone, Debug)]
pub enum RRClass {
    IN,
    CS,
    CH,
    HS,
}

impl Into<u16> for RRClass {
    fn into(self) -> u16 {
        match self {
            RRClass::IN => 1,
            RRClass::CS => 2,
            RRClass::CH => 3,
            RRClass::HS => 4,
        }
    }
}

impl TryFrom<u16> for RRClass {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> Result<Self> {
        Ok(match value {
            1 => RRClass::IN,
            2 => RRClass::CS,
            3 => RRClass::CH,
            4 => RRClass::HS,
            other => bail!("{other} is not a valid Class")
        })
    }
}

#[derive(Clone, Debug)]
pub enum QClass {
    RRClass(RRClass),
    ANY,
}

impl TryFrom<u16> for QClass {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> Result<Self> {
        Ok(match value {
            255 => QClass::ANY,
            other => match RRClass::try_from(value) {
                Ok(res) => QClass::RRClass(res),
                Err(_) => bail!("{other} is not a valid QClass")
            },
        })
    }
}

impl Into<u16> for QClass {
    fn into(self) -> u16 {
        match self {
            QClass::ANY => 255,
            QClass::RRClass(rr) => rr.into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Name {
    labels: Vec<String>,
}

impl From<Vec<String>> for Name {
    fn from(value: Vec<String>) -> Self {
        Name { labels: value }
    }
}

impl Name {
    pub fn to_vec(&self) -> Vec<u8> {
        let mut result: Vec<u8> = self.labels
            .iter()
            .map(|s| {
                let mut v = vec![s.len() as u8];
                v.extend(s.as_bytes());
                v})
            .flatten()
            .collect();

        result.push(0);

        result
    }
}
