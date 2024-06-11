use super::byte_packet_buffer::BytePacketBuffer;
use super::error::Result;
use super::result_code::ResultCode;

#[derive(Debug)]
pub struct Header {
    pub id: u16,
    pub recursion_desired: bool,
    pub truncated_message: bool,
    pub authoritative_answer: bool,
    pub opcode: u8,
    pub response: bool,

    pub rescode: ResultCode,
    pub checking_disabled: bool,
    pub authed_data: bool,
    pub z: bool,
    pub recursion_available: bool,

    pub questions: u16,
    pub answers: u16,
    pub authoritative_entries: u16,
    pub resource_entries: u16,
}

impl Header {
    pub fn new() -> Self {
        Self {
            id: 0,
            recursion_desired: false,
            truncated_message: false,
            authoritative_answer: false,
            opcode: 0,
            response: false,

            rescode: ResultCode::NoError,
            checking_disabled: false,
            authed_data: false,
            z: false,
            recursion_available: false,

            questions: 0,
            answers: 0,
            authoritative_entries: 0,
            resource_entries: 0,
        }
    }

    pub fn read(&mut self, buffer: &mut BytePacketBuffer) -> Result<()> {
        self.id = buffer.read_u16()?;
        let flags = buffer.read_u16()?;
        let a = (flags >> 8) as u8;
        let b = (flags & 0xFF) as u8;

        self.recursion_desired = (a & (1 << 0)) > 0;
        self.truncated_message = (a & (1 << 1)) > 0;
        self.authoritative_answer = (a & (1 << 2)) > 0;
        self.opcode = (a >> 3) & 0x0F;
        self.response = (a & (1 << 7)) > 0;

        self.rescode = ResultCode::from(b & 0x0F);
        self.checking_disabled = (b & (1 << 4)) > 0;
        self.authed_data = (b & (1 << 5)) > 0;
        self.z = (b & (1 << 6)) > 0;
        self.recursion_available = (b & (1 << 7)) > 0;

        self.questions = buffer.read_u16()?;
        self.answers = buffer.read_u16()?;
        self.authoritative_entries = buffer.read_u16()?;
        self.resource_entries = buffer.read_u16()?;
        Ok(())
    }

    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<()> {
        buffer.write_u16(self.id)?;

        buffer.write_u8(
            (self.recursion_desired as u8)
                | ((self.truncated_message as u8) << 1)
                | ((self.authoritative_answer as u8) << 2)
                | (self.opcode << 3)
                | ((self.response as u8) << 7),
        )?;

        buffer.write_u8(
            (self.rescode as u8)
                | ((self.checking_disabled as u8) << 4)
                | ((self.authed_data as u8) << 5)
                | ((self.z as u8) << 6)
                | ((self.recursion_available as u8) << 7),
        )?;

        buffer.write_u16(self.questions)?;
        buffer.write_u16(self.answers)?;
        buffer.write_u16(self.authoritative_entries)?;
        buffer.write_u16(self.resource_entries)?;

        Ok(())
    }

    pub fn debug_fmt(&self, f: &mut std::fmt::Formatter<'_>, indent: usize) -> std::fmt::Result {
        let t = "\t".repeat(indent);
        writeln!(f, "{t}id: {}", self.id)?;
        writeln!(f, "{t}recursion_desired: {}", self.recursion_desired)?;
        writeln!(f, "{t}truncated_message: {}", self.truncated_message)?;
        writeln!(f, "{t}authoritative_answer: {}", self.authoritative_answer)?;
        writeln!(f, "{t}opcode: {}", self.opcode)?;
        writeln!(f, "{t}response: {}", self.response)?;
        writeln!(f, "{t}rescode: {}", self.rescode)?;
        writeln!(f, "{t}checking_disabled: {}", self.checking_disabled)?;
        writeln!(f, "{t}authed_data: {}", self.authed_data)?;
        writeln!(f, "{t}z: {}", self.z)?;
        writeln!(f, "{t}recursion_available: {}", self.recursion_available)?;
        writeln!(f, "{t}questions: {}", self.questions)?;
        writeln!(f, "{t}answers: {}", self.answers)?;
        writeln!(
            f,
            "{t}authoritative_entries: {}",
            self.authoritative_entries
        )?;
        writeln!(f, "{t}resource_entries: {}", self.resource_entries)?;
        Ok(())
    }
}

impl Default for Header {
    fn default() -> Self {
        Self::new()
    }
}
