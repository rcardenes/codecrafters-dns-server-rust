use std::collections::HashMap;

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

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        match value {
            OpCode::Query => 0,
            OpCode::IQuery => 1,
            OpCode::Status => 2,
            OpCode::Reserved(value) => value,
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

impl From<RRType> for u16 {
    fn from(value: RRType) -> u16 {
        match value {
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

impl From<QType> for u16 {
    fn from(value: QType) -> u16 {
        match value {
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

impl From<RRClass> for u16 {
    fn from(value: RRClass) -> u16 {
        match value {
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

impl From<QClass> for u16 {
    fn from(value: QClass) -> u16 {
        match value {
            QClass::ANY => 255,
            QClass::RRClass(rr) => rr.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Name {
    labels: Vec<String>,
    pointer: Option<u16>,
}

impl From<Vec<String>> for Name {
    fn from(value: Vec<String>) -> Self {
        Name {
            labels: value,
            pointer: None
        }
    }
}

impl From<Vec<&str>> for Name {
    fn from(value: Vec<&str>) -> Self {
        Name {
            labels: value.iter().map(|&s| s.to_string()).collect(),
            pointer: None,
        }
    }
}

impl TryFrom<&[u8]> for Name {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        let length = value.len();
        let mut last_pos = 0;
        let mut labels = vec![];

        while last_pos < length {
            let marker = last_pos;
            match value[marker..].first().unwrap() {
                0 => { return Ok(Name::from(labels)) },
                &upper_byte if upper_byte >= 0xc0 => {
                    if let Some(&lower_byte) = value.get(marker + 1) {
                        return Ok(Name {
                            labels,
                            pointer: Some(u16::from_be_bytes([upper_byte & 0x03, lower_byte])),
                        })
                    }
                },
                &label_length if label_length < 64 => {
                    let start = marker + 1;
                    let end = start + label_length as usize;
                    if end >= length {
                        bail!("Corrupt name: truncated label")
                    }

                    let label = String::from_utf8(value[start..end].to_vec())?;
                    labels.push(label);
                    last_pos = end;
                },
                &other => bail!("Corrupt name: label length {other} is illegal"),
            }
        }

        bail!("Corrupt name: no null label terminator")
    }
}

impl Name {
    pub fn new(labels: Vec<String>, pointer: Option<u16>) -> Self {
        Name { labels, pointer }
    }

    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        let labels_len = self.labels.iter().map(|l| l.len() + 1).sum::<usize>();

        labels_len + if self.pointer.is_none() { 1 } else { 2 }
    }

    pub fn labels(&self) -> &Vec<String> {
        &self.labels
    }

    pub fn pointer(&self) -> &Option<u16> {
        &self.pointer
    }
    
    pub fn expand(&self, references: &HashMap<u16, Vec<String>>) -> Result<Name> {
        let expanded = if let Some(ptr) = self.pointer {
            if let Some(suffix) = references.get(&ptr) {
                suffix.clone()
            } else {
                bail!("Corrupt name: compression pointing to non-registered name at {ptr}")
            }
        } else {
            vec![]
        };

        Ok(Name {
            labels: [ self.labels.clone(), expanded ].iter()
                                                     .flatten()
                                                     .cloned()
                                                     .collect::<Vec<_>>(),
            pointer: None,
        })
    }

    pub fn compress(&self, references: &HashMap<Vec<String>, u16>) -> Result<Name> {
        if self.pointer.is_some() {
            bail!("Can compress only uncompressed names")
        }

        for k in 0..self.labels.len() {
            if let Some(pointer) = references.get(&self.labels[k..].to_vec()) {
                return Ok(Name {
                    labels: self.labels[..k].to_vec(),
                    pointer: Some(*pointer)
                });
            }
        }

        Ok(self.clone())
    }

    pub fn to_vec(&self) -> Vec<u8> {
        [
            self.labels
                .iter()
                .flat_map(|s| {
                    let mut v = vec![s.len() as u8];
                    v.extend(s.as_bytes());
                    v})
                .collect(),
            match self.pointer {
                Some(ptr) => { u16::to_be_bytes(0xc000 | ptr).to_vec() },
                None => { vec![0] }
            }
        ].iter().flatten().copied().collect::<Vec<_>>()
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
            source.split('.')
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

        data.iter().flatten().copied().collect()
    }
}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;

    use super::*;

    static LABELS: &[&str] = &["www", "server", "com"];
    static ENCODED_LABELS: &str = "\x03www\x06server\x03com\0";

    static ENCODED_LABELS_AND_POINTER: &[u8] = b"\x03www\xc0\x0c";
    static LABELS_AND_POINTER: Lazy<Name> = Lazy::new(|| {
        Name {
            labels: vec![String::from("www")],
            pointer: Some(12),
        }
    });
    static REFERENCES:     Lazy<HashMap::<u16, Vec<String>>> = Lazy::new(|| {
        [ (12, vec![String::from("server"), String::from("com")]) ]
            .into_iter()
            .collect()
    });
    static REV_REFERENCES: Lazy<HashMap::<Vec<String>, u16>> = Lazy::new(|| {
        [ (vec![String::from("server"), String::from("com")], 12) ]
            .into_iter()
            .collect()
    });

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
    fn decoding_name_with_pointer() -> Result<()> {
        assert_eq!(LABELS_AND_POINTER.clone(), Name::try_from(ENCODED_LABELS_AND_POINTER)?);

        Ok(())
    }

    #[test]
    fn encoding_name_with_pointer() -> Result<()> {
        assert_eq!(ENCODED_LABELS_AND_POINTER, LABELS_AND_POINTER.to_vec());

        Ok(())
    }

    #[test]
    fn expand_name_with_pointer() -> Result<()> {
        assert_eq!(LABELS, LABELS_AND_POINTER.expand(&REFERENCES)?.labels());

        Ok(())
    }

    #[test]
    fn compress_name() -> Result<()> {
        let name = Name::from(LABELS.to_vec());
        assert_eq!(LABELS_AND_POINTER.clone(), name.compress(&REV_REFERENCES)?);

        Ok(())
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
