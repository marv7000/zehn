use std::io::{self, Read, Seek, Write};

use crate::object::Endianness;

pub(crate) type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Aligns a given number to a given multiple.
///
/// # Arguments
/// * `pos` - The number to align.
/// * `align` - The multiple to align by.
pub fn align_to(pos: &u64, align: &u64) -> u64 {
    if *align == 0 {
        return pos.clone();
    }
    if pos % align == 0 {
        return pos.clone();
    }
    return pos + (align - pos % align);
}

pub trait SeekExt {
    fn align(&mut self, align: u64) -> Result<()>;
}

impl<S: Seek> SeekExt for S {
    fn align(&mut self, align: u64) -> Result<()> {
        let cur = self.stream_position()?;
        self.seek(io::SeekFrom::Start(align_to(&cur, &align)))?;

        return Ok(());
    }
}

pub trait ReadExt {
    fn read_bytes<const COUNT: usize>(&mut self) -> Result<[u8; COUNT]>;
    fn read_cstr(&mut self) -> Result<String>;
    fn read_u8(&mut self) -> Result<u8>;
    fn read_u16(&mut self, endian: &Endianness) -> Result<u16>;
    fn read_u32(&mut self, endian: &Endianness) -> Result<u32>;
    fn read_u64(&mut self, endian: &Endianness) -> Result<u64>;
}

impl<R: Read> ReadExt for R {
    fn read_bytes<const COUNT: usize>(&mut self) -> Result<[u8; COUNT]> {
        let mut x = [0; COUNT];
        self.read(&mut x)?;
        Ok(x)
    }

    fn read_cstr(&mut self) -> Result<String> {
        // Most strings will probably be at least 16 bytes long.
        let mut result = Vec::with_capacity(16);
        let mut c;
        loop {
            c = self.read_u8()?;
            if c == 0 {
                break;
            }
            result.push(c);
        }
        return Ok(String::from_utf8(result)?);
    }

    fn read_u8(&mut self) -> Result<u8> {
        Ok(u8::from_ne_bytes(self.read_bytes()?))
    }

    fn read_u16(&mut self, endian: &Endianness) -> Result<u16> {
        match endian {
            Endianness::Little => Ok(u16::from_le_bytes(self.read_bytes()?)),
            Endianness::Big => Ok(u16::from_be_bytes(self.read_bytes()?)),
        }
    }

    fn read_u32(&mut self, endian: &Endianness) -> Result<u32> {
        match endian {
            Endianness::Little => Ok(u32::from_le_bytes(self.read_bytes()?)),
            Endianness::Big => Ok(u32::from_be_bytes(self.read_bytes()?)),
        }
    }

    fn read_u64(&mut self, endian: &Endianness) -> Result<u64> {
        match endian {
            Endianness::Little => Ok(u64::from_le_bytes(self.read_bytes()?)),
            Endianness::Big => Ok(u64::from_be_bytes(self.read_bytes()?)),
        }
    }
}

pub trait WriteExt {
    fn write_bytes<const COUNT: usize>(&mut self, value: &[u8; COUNT]) -> Result<usize>;
    fn write_cstr(&mut self, value: &str) -> Result<usize>;
    fn write_u8(&mut self, value: u8) -> Result<usize>;
    fn write_u16(&mut self, endian: &Endianness, value: u16) -> Result<usize>;
    fn write_u32(&mut self, endian: &Endianness, value: u32) -> Result<usize>;
    fn write_u64(&mut self, endian: &Endianness, value: u64) -> Result<usize>;
}

impl<W: Write> WriteExt for W {
    fn write_bytes<const COUNT: usize>(&mut self, value: &[u8; COUNT]) -> Result<usize> {
        Ok(self.write(value)?)
    }

    fn write_cstr(&mut self, value: &str) -> Result<usize> {
        let written = self.write(value.as_bytes())?;
        self.write_u8(0)?;
        Ok(written + 1)
    }

    fn write_u8(&mut self, value: u8) -> Result<usize> {
        Ok(self.write(&value.to_le_bytes())?)
    }

    fn write_u16(&mut self, endian: &Endianness, value: u16) -> Result<usize> {
        match endian {
            Endianness::Little => Ok(self.write(&value.to_le_bytes())?),
            Endianness::Big => Ok(self.write(&value.to_be_bytes())?),
        }
    }

    fn write_u32(&mut self, endian: &Endianness, value: u32) -> Result<usize> {
        match endian {
            Endianness::Little => Ok(self.write(&value.to_le_bytes())?),
            Endianness::Big => Ok(self.write(&value.to_be_bytes())?),
        }
    }

    fn write_u64(&mut self, endian: &Endianness, value: u64) -> Result<usize> {
        match endian {
            Endianness::Little => Ok(self.write(&value.to_le_bytes())?),
            Endianness::Big => Ok(self.write(&value.to_be_bytes())?),
        }
    }
}
