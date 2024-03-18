use crate::object::{Class, Endianness, Object};
use crate::util::ReadExt;
use crate::util::Result;
use crate::util::WriteExt;
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub sym_name: u32,
    pub sym_info: u8,
    pub sym_other: u8,
    pub sym_shndx: u16,
    pub sym_value: u64,
    pub sym_size: u64,
}

impl Symbol {
    pub fn get_name(&self, obj: &Object) -> Result<String> {
        let strtab = obj
            .find_section(".strtab")
            .expect("Unable to get the name for a symbol: Section \".strtab\" was not present!");
        let mut name = &strtab.body[self.sym_name as usize..];
        Ok(name.read_cstr()?)
    }

    pub fn read(class: &Class, endian: &Endianness, mut buf: impl Read) -> Result<Self> {
        let prog = match class {
            Class::Bits32 => Self {
                sym_name: buf.read_u32(endian)?,
                sym_info: buf.read_u8()?,
                sym_other: buf.read_u8()?,
                sym_shndx: buf.read_u16(endian)?,
                sym_value: buf.read_u32(endian)? as u64,
                sym_size: buf.read_u32(endian)? as u64,
            },
            Class::Bits64 => Self {
                sym_name: buf.read_u32(endian)?,
                sym_info: buf.read_u8()?,
                sym_other: buf.read_u8()?,
                sym_shndx: buf.read_u16(endian)?,
                sym_value: buf.read_u64(endian)?,
                sym_size: buf.read_u64(endian)?,
            },
        };
        Ok(prog)
    }
    pub fn write(&self, class: &Class, endian: &Endianness, mut buf: impl Write) -> Result<usize> {
        let mut written = 0;
        match class {
            Class::Bits32 => {
                written += buf.write_u32(endian, self.sym_name)?;
                written += buf.write_u8(self.sym_info)?;
                written += buf.write_u8(self.sym_other)?;
                written += buf.write_u16(endian, self.sym_shndx)?;
                written += buf.write_u32(endian, self.sym_value as u32)?;
                written += buf.write_u32(endian, self.sym_size as u32)?;
            }
            Class::Bits64 => {
                written += buf.write_u32(endian, self.sym_name)?;
                written += buf.write_u8(self.sym_info)?;
                written += buf.write_u8(self.sym_other)?;
                written += buf.write_u16(endian, self.sym_shndx)?;
                written += buf.write_u64(endian, self.sym_value)?;
                written += buf.write_u64(endian, self.sym_size)?;
            }
        }
        Ok(written)
    }
}
