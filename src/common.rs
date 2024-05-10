use anyhow::{Result, bail};

#[derive(Clone, Debug, Default, PartialEq)]
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


#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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
            QType::RRType(rr) => rr.into(),
            QType::AXFR => 252,
            QType::MAILB => 253,
            QType::MAILA => 254,
            QType::ANY => 255,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Name {
    labels: Vec<String>,
}

impl From<Vec<String>> for Name {
    fn from(value: Vec<String>) -> Self {
        Name { labels: value }
    }
}

impl From<Vec<&str>> for Name {
    fn from(value: Vec<&str>) -> Self {
        Name { labels: value.iter().map(|&s| s.to_string()).collect() }
    }
}

impl TryFrom<&[u8]> for Name {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        let length = value.len();
        let mut last_pos = 0;
        let mut labels = vec![];

        while last_pos < length {
            let marker = last_pos as usize;
            match value[marker..].first().unwrap() {
                0 => { return Ok(Name { labels }) },
                64.. => bail!("Corrupt name: label length > 63"),
                label_length => {
                    let start = marker + 1;
                    let end = start + *label_length as usize;
                    if end >= length {
                        bail!("Corrupt name: truncated label")
                    }

                    let label = String::from_utf8(value[start..end].to_vec())?;
                    labels.push(label);
                    last_pos = end;
                },
            }
        }

        bail!("Corrupt name: no null label terminator")
    }
}

impl Name {
    pub fn len(&self) -> usize {
        self.labels.iter().map(|l| l.len() + 1).sum::<usize>() + 1
    }

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

#[derive(Clone, Debug, Default, PartialEq)]
pub enum ResponseCode {
    NoError = 0,
    #[default]
    FormatError,
    ServerFailure,
    NameError,
    NotImplemented,
    Refused,
    Reserved,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    rrtype: RRType,
    rrclass: RRClass,
    data: Vec<u8>,
}

impl Record {
    pub fn from_ip_v4(source: &str) -> Result<Self> {
        let components: std::result::Result<Vec<_>, _> =
            source.split(".")
                  .map(|c| c.parse::<u8>())
                  .collect();

        Ok(Record {
            rrtype: RRType::A,
            rrclass: RRClass::IN,
            data: components?,
        })
    }

    pub fn rrtype(&self) -> &RRType { &self.rrtype }
    pub fn rrclass(&self) -> &RRClass { &self.rrclass }
    pub fn data(&self) -> &Vec<u8> { &self.data }

    pub fn to_vec(&self) -> Vec<u8> {
        let data = [
            u16::to_be_bytes(self.rrtype.clone().into()).to_vec(),
            u16::to_be_bytes(self.rrclass.clone().into()).to_vec(),
            u16::to_be_bytes(self.data.len() as u16).to_vec(),
            self.data.clone()
        ];

        data.iter().flatten().map(|&val| val).collect()
    }
}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;

    use super::*;

    static LABELS: &[&str] = &["www", "server", "com"];
    static ENCODED_LABELS: &str = "\x03www\x06server\x03com\0";
    static IPV4: &str = "1.2.3.4";
    static IPV4_RECORD: Lazy<Record> = Lazy::new(|| {
        Record {
            rrtype: RRType::A,
            rrclass: RRClass::IN,
            data: b"\x01\x02\x03\x04".to_vec()
        }
    });
    static ENCODED_IPV4_RECORD: Lazy<Vec<u8>> = Lazy::new(|| {
        b"\x00\x01\x00\x01\x00\x04\x01\x02\x03\x04".to_vec()
    });

    #[test]
    fn vec_to_name() {
        let encoded_vec: Vec<u8> = ENCODED_LABELS.into();

        let name: Name = LABELS.iter()
            .map(|&s| String::from(s))
            .collect::<Vec<String>>()
            .into();

        assert_eq!(encoded_vec, name.to_vec());
    }

    #[test]
    fn name_to_vec() {
        let encoded_vec: Vec<u8> = ENCODED_LABELS.into();
        let name = Name::try_from(&encoded_vec[..]).unwrap();

        let target: Name = LABELS.iter()
            .map(|&s| String::from(s))
            .collect::<Vec<String>>()
            .into();

        assert_eq!(target, name);
    }

    #[test]
    fn ipv4_str_to_record() -> Result<()> {
        assert_eq!(IPV4_RECORD.clone(), Record::from_ip_v4(IPV4)?);
        Ok(())
    }

    #[test]
    fn encode_record() -> Result<()> {
        assert_eq!(ENCODED_IPV4_RECORD.clone(), IPV4_RECORD.to_vec());
        Ok(())
    }
}
