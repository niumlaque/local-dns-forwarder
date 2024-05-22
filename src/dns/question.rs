use super::byte_packet_buffer::BytePacketBuffer;
use super::error::Result;
use super::query_type::QueryType;

#[derive(Debug)]
pub struct Question {
    pub name: String,
    pub qtype: QueryType,
    pub class: u16,
}

impl Question {
    pub fn new(name: impl Into<String>, qtype: QueryType, class: u16) -> Self {
        Self {
            name: name.into(),
            qtype,
            class,
        }
    }

    pub fn read(buf: &mut BytePacketBuffer) -> Result<Self> {
        let name = buf.read_qname()?;
        let qtype = QueryType::from(buf.read_u16()?);
        let class = buf.read_u16()?;
        Ok(Question::new(name, qtype, class))
    }

    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<()> {
        buffer.write_qname(&self.name)?;

        let typenum = self.qtype.into();
        buffer.write_u16(typenum)?;
        buffer.write_u16(self.class)?;

        Ok(())
    }
}
