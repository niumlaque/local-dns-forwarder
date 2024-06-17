use super::byte_packet_buffer::BytePacketBuffer;
use super::error::Result;
use super::header::Header;
use super::question::Question;
use super::record::Record;

/// All communications inside of the domain protocol are carried in a single
/// format called a message.  The top level format of message is divided
/// into 5 sections (some of which are empty in certain cases) shown below:
///     +---------------------+
///     |        Header       |
///     +---------------------+
///     |       Question      | the question for the name server
///     +---------------------+
///     |        Answer       | RRs answering the question
///     +---------------------+
///     |      Authority      | RRs pointing toward an authority
///     +---------------------+
///     |      Additional     | RRs holding additional information
///     +---------------------+
/// The header section is always present.  The header includes fields that
/// specify which of the remaining sections are present, and also specify
/// whether the message is a query or a response, a standard query or some
/// other opcode, etc.
///
/// The names of the sections after the header are derived from their use in
/// standard queries.  The question section contains fields that describe a
/// question to a name server.  These fields are a query type (QTYPE), a
/// query class (QCLASS), and a query domain name (QNAME).  The last three
/// sections have the same format: a possibly empty list of concatenated
/// resource records (RRs).  The answer section contains RRs that answer the
/// question; the authority section contains RRs that point toward an
/// authoritative name server; the additional records section contains RRs
/// which relate to the query, but are not strictly answers for the
/// question.
#[derive(Default)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<Record>,
    pub authorities: Vec<Record>,
    pub resources: Vec<Record>,
}

impl Message {
    pub fn new() -> Self {
        Self {
            header: Header::new(),
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            resources: Vec::new(),
        }
    }

    pub fn read(buf: &mut BytePacketBuffer) -> Result<Self> {
        let mut result = Message::new();
        result.header.read(buf)?;

        for _ in 0..result.header.questions {
            let question = Question::read(buf)?;
            result.questions.push(question);
        }

        for _ in 0..result.header.answers {
            let rec = Record::read(buf)?;
            result.answers.push(rec);
        }
        for _ in 0..result.header.authoritative_entries {
            let rec = Record::read(buf)?;
            result.authorities.push(rec);
        }
        for _ in 0..result.header.resource_entries {
            let rec = Record::read(buf)?;
            result.resources.push(rec);
        }

        Ok(result)
    }

    pub fn write(&mut self, buf: &mut BytePacketBuffer) -> Result<()> {
        self.header.questions = self.questions.len() as u16;
        self.header.answers = self.answers.len() as u16;
        self.header.authoritative_entries = self.authorities.len() as u16;
        self.header.resource_entries = self.resources.len() as u16;

        self.header.write(buf)?;

        for question in &self.questions {
            question.write(buf)?;
        }
        for rec in &self.answers {
            rec.write(buf)?;
        }
        for rec in &self.authorities {
            rec.write(buf)?;
        }
        for rec in &self.resources {
            rec.write(buf)?;
        }

        Ok(())
    }

    pub fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Header:")?;
        self.header.debug_fmt(f, 1)?;
        writeln!(f, "Questions({})", self.questions.len())?;
        for (i, v) in self.questions.iter().enumerate() {
            println!("\tQuestion[{i}]");
            v.debug_fmt(f, 2)?;
        }
        writeln!(f, "Answers({})", self.answers.len())?;
        for (i, v) in self.answers.iter().enumerate() {
            println!("\tAnswer[{i}]");
            v.debug_fmt(f, 2)?;
        }
        writeln!(f, "Authorities({})", self.authorities.len())?;
        for (i, v) in self.authorities.iter().enumerate() {
            println!("\tAuthority[{i}]");
            v.debug_fmt(f, 2)?;
        }
        writeln!(f, "Resources({})", self.resources.len())?;
        for (i, v) in self.resources.iter().enumerate() {
            println!("\tResource[{i}]");
            v.debug_fmt(f, 2)?;
        }
        Ok(())
    }
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.debug_fmt(f)
    }
}
