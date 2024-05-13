use super::byte_packet_buffer::BytePacketBuffer;
use super::error::Result;
use super::query_type::QueryType;

#[derive(Debug)]
pub struct Question {
    pub name: String,
    pub qtype: QueryType,
}

impl Question {
    pub fn new(name: impl Into<String>, qtype: QueryType) -> Self {
        Self {
            name: name.into(),
            qtype,
        }
    }

    pub fn read(buf: &mut BytePacketBuffer) -> Result<Self> {
        let name = buf.read_qname()?;
        let qtype = QueryType::from(buf.read_u16()?);
        let _ = buf.read_u16()?; // class
        Ok(Self { name, qtype })
    }

    pub fn write(&self, buffer: &mut BytePacketBuffer) -> Result<()> {
        buffer.write_qname(&self.name)?;

        let typenum = self.qtype.into();
        buffer.write_u16(typenum)?;
        buffer.write_u16(1)?;

        Ok(())
    }
}
