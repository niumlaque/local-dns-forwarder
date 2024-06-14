use super::{Error, Result};

const BUF_SIZE: usize = 512;

#[derive(Debug)]
pub struct BytePacketBuffer {
    pub buf: [u8; BUF_SIZE],
    pub pos: usize,
}

impl BytePacketBuffer {
    pub fn new() -> Self {
        Self {
            buf: [0; BUF_SIZE],
            pos: 0,
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn step(&mut self, steps: usize) -> Result<()> {
        self.pos += steps;
        Ok(())
    }

    pub fn seek(&mut self, pos: usize) -> Result<()> {
        self.pos = pos;
        Ok(())
    }

    pub fn read(&mut self) -> Result<u8> {
        if self.pos < BUF_SIZE {
            let v = self.buf[self.pos];
            self.pos += 1;
            Ok(v)
        } else {
            Err(Error::EndOfBuffer)
        }
    }
    pub fn read_range(&mut self, len: usize) -> Result<&[u8]> {
        if self.pos + len < BUF_SIZE {
            let v = &self.buf[self.pos..self.pos + len];
            self.pos += len;
            Ok(v)
        } else {
            Err(Error::EndOfBuffer)
        }
    }

    pub fn get(&self, pos: usize) -> Result<u8> {
        if self.pos < BUF_SIZE {
            Ok(self.buf[pos])
        } else {
            Err(Error::EndOfBuffer)
        }
    }

    pub fn get_range(&self, pos: usize, len: usize) -> Result<&[u8]> {
        if pos + len < BUF_SIZE {
            Ok(&self.buf[pos..pos + len])
        } else {
            Err(Error::EndOfBuffer)
        }
    }

    pub fn get_all(&self) -> Result<&[u8]> {
        let len = self.pos();
        if len < BUF_SIZE {
            Ok(&self.buf[0..len])
        } else {
            Err(Error::EndOfBuffer)
        }
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        Ok(((self.read()? as u16) << 8) | (self.read()? as u16))
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        Ok(((self.read()? as u32) << 24)
            | ((self.read()? as u32) << 16)
            | ((self.read()? as u32) << 8)
            | self.read()? as u32)
    }

    pub fn read_u128(&mut self) -> Result<u128> {
        Ok(((self.read()? as u128) << 120)
            | ((self.read()? as u128) << 112)
            | ((self.read()? as u128) << 104)
            | ((self.read()? as u128) << 96)
            | ((self.read()? as u128) << 88)
            | ((self.read()? as u128) << 80)
            | ((self.read()? as u128) << 72)
            | ((self.read()? as u128) << 64)
            | ((self.read()? as u128) << 56)
            | ((self.read()? as u128) << 48)
            | ((self.read()? as u128) << 40)
            | ((self.read()? as u128) << 32)
            | ((self.read()? as u128) << 24)
            | ((self.read()? as u128) << 16)
            | ((self.read()? as u128) << 8)
            | self.read()? as u128)
    }

    pub fn read_qname(&mut self) -> Result<String> {
        let mut pos = self.pos();
        let mut jumped = false;
        let max_jumps = 5;
        let mut jumps_performed = 0;
        let mut ret = String::default();

        let mut delim = "";
        loop {
            if jumps_performed > max_jumps {
                return Err(Error::JumpLimit(max_jumps));
            }

            let len = self.get(pos)?;

            if (len & 0xC0) == 0xC0 {
                if !jumped {
                    self.seek(pos + 2)?;
                }

                let b2 = self.get(pos + 1)? as u16;
                let offset = (((len as u16) ^ 0xC0) << 8) | b2;
                pos = offset as usize;

                jumped = true;
                jumps_performed += 1;

                continue;
            } else {
                pos += 1;
                if len == 0 {
                    break;
                }

                ret.push_str(delim);

                let str_buffer = self.get_range(pos, len as usize)?;
                ret.push_str(&String::from_utf8_lossy(str_buffer).to_lowercase());

                delim = ".";

                pos += len as usize;
            }
        }

        if !jumped {
            self.seek(pos)?;
        }

        Ok(ret)
    }

    pub fn write(&mut self, v: u8) -> Result<()> {
        if self.pos < BUF_SIZE {
            self.buf[self.pos] = v;
            self.pos += 1;
            Ok(())
        } else {
            Err(Error::EndOfBuffer)
        }
    }

    pub fn write_u8(&mut self, v: u8) -> Result<()> {
        self.write(v)?;
        Ok(())
    }

    pub fn write_u16(&mut self, v: u16) -> Result<()> {
        self.write((v >> 8) as u8)?;
        self.write((v & 0xFF) as u8)?;
        Ok(())
    }
    pub fn write_u32(&mut self, v: u32) -> Result<()> {
        self.write(((v >> 24) & 0xFF) as u8)?;
        self.write(((v >> 16) & 0xFF) as u8)?;
        self.write(((v >> 8) & 0xFF) as u8)?;
        self.write((v & 0xFF) as u8)?;
        Ok(())
    }

    pub fn write_qname(&mut self, qname: &str) -> Result<()> {
        for label in qname.split('.') {
            let len = label.len();
            if len > 0x3f {
                return Err(Error::SingleLabelLimit);
            }

            self.write_u8(len as u8)?;
            for b in label.as_bytes() {
                self.write_u8(*b)?;
            }
        }

        self.write_u8(0)?;

        Ok(())
    }

    pub fn write_range(&mut self, v: &[u8]) -> Result<()> {
        let end = self.pos + v.len();
        if end < BUF_SIZE {
            self.buf[self.pos..end].copy_from_slice(v);
            Ok(())
        } else {
            Err(Error::EndOfBuffer)
        }
    }
}

impl Default for BytePacketBuffer {
    fn default() -> Self {
        Self::new()
    }
}
