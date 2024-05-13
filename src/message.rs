use std::collections::{hash_map::Entry, HashMap};

use anyhow::{Result, bail};
use crate::common::{Name, OpCode, QClass, QType, Record, ResponseCode};

#[derive(Clone, Debug, PartialEq)]
pub struct Question {
    qname: Name,
    qtype: QType,
    qclass: QClass,
}

impl Question {
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.qname.len() + 4
    }

    pub fn name(&self) -> &Name {
        &self.qname
    }

    pub fn expand(&self, references: &HashMap<u16, Vec<String>>) -> Result<Question> {
        Ok(Question {
            qname: self.qname.expand(references)?,
            ..self.clone()
        })
    }

    pub fn compress(&self, references: &HashMap<Vec<String>, u16>) -> Result<Question> {
        Ok(Question {
            qname: self.qname.compress(references)?,
            ..self.clone()
        })
    }

    pub fn to_vec(&self) -> Vec<u8> {
        let mut result = self.qname.to_vec();
        result.extend(u16::to_be_bytes(self.qtype.clone().into()));
        result.extend(u16::to_be_bytes(self.qclass.clone().into()));

        result
    }
}

impl TryFrom<&[u8]> for Question {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        if !value.is_empty() {
            let qname = Name::try_from(value)?;
            let meta = qname.len();
            let query_end = meta + 4;

            if query_end > value.len() {
                bail!("Corrupt message: truncated question");
            }

            let qtype = QType::try_from(u16::from_be_bytes([value[meta], value[meta + 1]]))?;
            let qclass = QClass::try_from(u16::from_be_bytes([value[meta + 2], value[meta + 3]]))?;

            Ok(Question { qname, qtype, qclass })
        } else {
            bail!("Empty question")
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Query {
    response_code: ResponseCode,
    id: u16,
    opcode: OpCode,
    truncation: bool,
    recursion_desired: bool,
    questions: Vec<Question>,
    dict: HashMap<u16, Vec<String>>,
}

impl Query {
    fn new() -> Self {
        Self {
            response_code: ResponseCode::FormatError,
            ..Self::default()
        }
    }

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

    pub fn response_code(&self) -> ResponseCode {
        self.response_code.clone()
    }
}

impl TryFrom<&[u8]> for Query {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        if value.len() < 2 {
            bail!("Not even an id!");
        }

        let mut query = Query::new();

        query.id = u16::from_be_bytes([value[0], value[1]]);

        if value.len() > 2 {
            if (value[2] & 0x80) != 0 {
                bail!("This is a response, not a query!");
            }
            query.opcode = ((value[2] >> 3) & 0x0f).into();
            query.truncation = (value[2] & 0x2) == 0x2;
            query.recursion_desired = (value[2] & 0x1) == 0x1;
        }

        if value.len() > 3 && ((value[3] & 0xf0) != 0) {
            eprintln!("Field 'Z' is not all zeros");
            return Ok(query);
        }

        if value.len() < 12 {
            eprintln!("Truncated package with len: {}", value.len());
            return Ok(query);
        }

        let mut ref_store: HashMap<u16, Vec<String>> = HashMap::new();
        let mut qdcount = u16::from_be_bytes([value[4], value[5]]);
        let mut ptr = 12;
        while qdcount > 0 {
            if ptr >= value.len() {
                eprintln!("Truncated message: query count larger than contents");
                return Ok(query)
            }

            let question = match Question::try_from(&value[ptr..]) {
                Ok(q) => q,
                Err(err) => { println!("{err}"); return Ok(query); }
            };

            let n_labels = question.name().labels().len();

            let delta = question.len();

            let expanded_question = {
                if question.name().pointer().is_some() {
                    question.expand(&ref_store)?
                } else {
                    question
                }
            };

            let expanded_labels = expanded_question.name().labels().clone();

            let mut ref_ptr = ptr as u16;
            for start in 0..n_labels {
                let partial = &expanded_labels[start..];
                ref_store.insert(ref_ptr, partial.to_vec());

                ref_ptr += (partial[0].len() + 1) as u16;
            }

            ref_store.insert(ptr as u16, expanded_question.name().labels().clone());

            ptr += delta;
            qdcount -= 1;
            query.questions.push(expanded_question);
        }

        query.response_code = match query.opcode {
            OpCode::Query => ResponseCode::NoError,
            _ => ResponseCode::NotImplemented,
        };

        Ok(query)
    }
}

#[derive(Clone, Debug)]
pub struct Answer {
    name: Name,
    record: Record,
    ttl: u32,
}

impl Answer {
    pub fn new(name: &Name, record: &Record, ttl: u32) -> Self {
        Answer {
            name: name.clone(),
            record: record.clone(),
            ttl
        }
    }

    pub fn name(&self) -> &Name {
        &self.name
    }

    pub fn compress(&self, references: &HashMap<Vec<String>, u16>) -> Result<Answer> {
        Ok(Answer {
            name: self.name.compress(references)?,
            ..self.clone()
        })
    }

    pub fn to_vec(&self) -> Vec<u8> {
        [self.name.to_vec(),
         u16::to_be_bytes(self.record.rrtype().clone().into()).to_vec(),
         u16::to_be_bytes(self.record.rrclass().clone().into()).to_vec(),
         u32::to_be_bytes(self.ttl).to_vec(),
         u16::to_be_bytes(self.record.data().len() as u16).to_vec(),
         self.record.data().clone()
        ].iter().flatten().cloned().collect()
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    id: u16,
    opcode: OpCode,
    truncation: bool,
    authoritative_answer: bool,
    recursion_desired: bool,
    recursion_available: bool,
    response_code: ResponseCode,
    questions: Vec<Question>,
    answers: Vec<Answer>,
}

impl Response {
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder::default()
    }
}

impl From<Response> for Vec<u8> {
    fn from(value: Response) -> Self {
        let oc: u8 = value.opcode.into();
        let aa = if value.authoritative_answer { 4u8 } else { 0 };
        let tc = if value.truncation { 2u8 } else { 0 };
        let rd = if value.recursion_desired { 1u8 } else { 0 };
        let rc = value.response_code as u8;
        let ra = if value.recursion_available { 0x80u8 } else { 0 };
        let qdcount = u16::to_be_bytes(value.questions.len() as u16);
        let ancount = u16::to_be_bytes(value.answers.len() as u16);

        let mut res = vec![
            (value.id >> 8) as u8, (value.id & 0xff) as u8,
            (0x80 | oc << 3 | aa | tc | rd) , ra | rc,
            qdcount[0], qdcount[1],
            ancount[0], ancount[1],
            0, 0,
            0, 0,
        ];

        let mut ref_store: HashMap::<Vec<String>, u16> = HashMap::new();
        let mut ptr: u16 = 12;
        for question in value.questions {
            let compressed = question.compress(&ref_store).unwrap().to_vec();
            let labels = question.name().labels().clone();
            let mut ref_ptr = ptr;
            for start in 0..labels.len() {
                let partial = &labels[start..];
                if let Entry::Vacant(v) = ref_store.entry(partial.to_vec()) {
                    v.insert(ref_ptr);
                }

                ref_ptr += (partial[0].len() + 1) as u16;
            }
            ptr += compressed.len() as u16;
            res.extend(compressed);
        }

        for answer in value.answers {
            let compressed = answer.compress(&ref_store).unwrap().to_vec();
            let labels = answer.name().labels().clone();
            let mut ref_ptr = ptr;
            for start in 0..labels.len() {
                let partial = &labels[start..];
                if let Entry::Vacant(v) = ref_store.entry(partial.to_vec()) {
                    v.insert(ref_ptr);
                }

                ref_ptr += (partial[0].len() + 1) as u16;
            }
            ptr += compressed.len() as u16;
            res.extend(compressed);
        }

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
    answers: Vec<Answer>,
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
            answers: self.answers,
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

    pub fn answers(mut self, answers: Vec<Answer>) -> Self {
        self.answers = answers;
        self
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use once_cell::sync::Lazy;
    use std::iter::zip;

    use super::*;
    use crate::common::{RRType, RRClass};

    static SAMPLE_BIN_QUERIES: &[&[u8]] = &[
        b"\xfd\xf0\x01\x00\x00\x01\x00\x00\x00\x00\x00\x00\x0ccodecrafters\x02io\x00\x00\x01\x00\x01",
    ];

    static SAMPLE_BIN_RESPONSES: &[&[u8]] = &[
        b"\xfd\xf0\x81\x00\x00\x01\x00\x00\x00\x00\x00\x00\x0ccodecrafters\x02io\x00\x00\x01\x00\x01",
        b"\xfd\xf0\x81\x00\x00\x01\x00\x01\x00\x00\x00\x00\x0ccodecrafters\x02io\x00\x00\x01\x00\x01\xc0\x0c\x00\x01\x00\x01\x00\x00\x00\x3c\x00\x04\x08\x08\x08\x08",
    ];

    static SAMPLE_BIN_QUESTION: &[u8] = b"\x0ccodecrafters\x02io\x00\x00\x01\x00\x01";

    static SAMPLE_QUESTION: Lazy<Question> = Lazy::new(|| {
        Question {
            qname: vec!["codecrafters", "io"].into(),
            qtype: QType::RRType(RRType::A),
            qclass: QClass::RRClass(RRClass::IN),
        }
    });

    static SAMPLE_ANSWER: Lazy<Answer> = Lazy::new(|| {
        Answer {
            name: Name::from(vec!["codecrafters", "io"]),
            record: Record::from_ip_v4("8.8.8.8").expect("Not a valid IPv4"),
            ttl: 60,
        }
    });

    static SAMPLE_QUERIES: Lazy<Vec<Query>> = Lazy::new(|| {
        vec![
            Query {
                response_code: ResponseCode::NoError,
                id: 0xfdf0,
                recursion_desired: true,
                questions: vec![SAMPLE_QUESTION.clone()],
                ..Query::new()
            },
        ]
    });

    static SAMPLE_RESPONSES: Lazy<Vec<Response>> = Lazy::new(|| {
        vec![
            Response::builder()
                .id(0xfdf0)
                .opcode(OpCode::Query)
                .recursion_desired(true)
                .questions(vec![SAMPLE_QUESTION.clone()])
                .response_code(ResponseCode::NoError)
                .build(),
            Response::builder()
                .id(0xfdf0)
                .opcode(OpCode::Query)
                .recursion_desired(true)
                .questions(vec![SAMPLE_QUESTION.clone()])
                .answers(vec![SAMPLE_ANSWER.clone()])
                .response_code(ResponseCode::NoError)
                .build(),
        ]
    });

    #[test]
    fn encode_decode_question() -> Result<()> {
        let question = Question::try_from(SAMPLE_BIN_QUESTION)?;

        assert_eq!(SAMPLE_QUESTION.clone(), question);
        assert_eq!(SAMPLE_BIN_QUESTION, question.to_vec());

        Ok(())
    }

    #[test]
    fn build_query() -> Result<()> {
        for (&bin, target) in zip(SAMPLE_BIN_QUERIES, SAMPLE_QUERIES.clone()) {
            let query = Query::try_from(bin)?;

            assert_eq!(target, query);
        }

        Ok(())
    }

    #[test]
    fn build_response() {
        for (&target, response) in zip(SAMPLE_BIN_RESPONSES, SAMPLE_RESPONSES.clone()) {
            let bin: Vec<u8> = response.into();

            assert_eq!(target, bin);
        }
    }
    
    #[test]
    fn compressed_query() -> Result<()> {
        let bytes = b"\xce5\x01\x00\x00\x02\x00\x00\x00\x00\x00\x00\x03abc\x11longassdomainname\x03com\x00\x00\x01\x00\x01\x03def\xc0\x10\x00\x01\x00\x01".to_vec();
        let _ = Query::try_from(&bytes[..])?;

        Ok(())
    }
}
