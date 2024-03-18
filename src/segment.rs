use std::io::Read;
use std::io::Write;

use crate::object::Class;
use crate::object::Endianness;
use crate::util::ReadExt;
use crate::util::Result;
use crate::util::WriteExt;

pub mod ptype {
    /// Program header table entry unused.
    pub const PT_NULL: u32 = 0x00000000;
    /// Loadable segment.
    pub const PT_LOAD: u32 = 0x00000001;
    /// Dynamic linking information.
    pub const PT_DYNAMIC: u32 = 0x00000002;
    /// Interpreter information.
    pub const PT_INTERP: u32 = 0x00000003;
    /// Auxiliary information.
    pub const PT_NOTE: u32 = 0x00000004;
    /// Reserved.
    pub const PT_SHLIB: u32 = 0x00000005;
    /// Segment containing program header table itself.
    pub const PT_PHDR: u32 = 0x00000006;
    /// Thread-Local Storage template.
    pub const PT_TLS: u32 = 0x00000007;
    /// Reserved inclusive range. Operating system specific.
    pub const PT_LOOS: u32 = 0x60000000;
    pub const PT_HIOS: u32 = 0x6FFFFFFF;
    /// Reserved inclusive range. Processor specific.
    pub const PT_LOPROC: u32 = 0x70000000;
    pub const PT_HIPROC: u32 = 0x7FFFFFFF;
}

#[derive(Debug, Clone)]
pub struct ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

impl ProgramHeader {
    pub fn read(class: &Class, endian: &Endianness, mut buf: impl Read) -> Result<Self> {
        let prog = match class {
            Class::Bits32 => Self {
                p_type: buf.read_u32(endian)?,
                p_offset: buf.read_u32(endian)? as u64,
                p_vaddr: buf.read_u32(endian)? as u64,
                p_paddr: buf.read_u32(endian)? as u64,
                p_filesz: buf.read_u32(endian)? as u64,
                p_memsz: buf.read_u32(endian)? as u64,
                p_flags: buf.read_u32(endian)?,
                p_align: buf.read_u32(endian)? as u64,
            },
            Class::Bits64 => Self {
                p_type: buf.read_u32(endian)?,
                p_flags: buf.read_u32(endian)?,
                p_offset: buf.read_u64(endian)?,
                p_vaddr: buf.read_u64(endian)?,
                p_paddr: buf.read_u64(endian)?,
                p_filesz: buf.read_u64(endian)?,
                p_memsz: buf.read_u64(endian)?,
                p_align: buf.read_u64(endian)?,
            },
        };
        Ok(prog)
    }

    pub fn write(&self, class: &Class, endian: &Endianness, mut buf: impl Write) -> Result<usize> {
        let mut written = 0;
        match class {
            Class::Bits32 => {
                written += buf.write_u32(endian, self.p_type)?;
                written += buf.write_u32(endian, self.p_offset as u32)?;
                written += buf.write_u32(endian, self.p_vaddr as u32)?;
                written += buf.write_u32(endian, self.p_paddr as u32)?;
                written += buf.write_u32(endian, self.p_filesz as u32)?;
                written += buf.write_u32(endian, self.p_memsz as u32)?;
                written += buf.write_u32(endian, self.p_flags)?;
                written += buf.write_u32(endian, self.p_align as u32)?;
            }
            Class::Bits64 => {
                written += buf.write_u32(endian, self.p_type)?;
                written += buf.write_u32(endian, self.p_flags)?;
                written += buf.write_u64(endian, self.p_offset)?;
                written += buf.write_u64(endian, self.p_vaddr)?;
                written += buf.write_u64(endian, self.p_paddr)?;
                written += buf.write_u64(endian, self.p_filesz)?;
                written += buf.write_u64(endian, self.p_memsz)?;
                written += buf.write_u64(endian, self.p_align)?;
            }
        }
        Ok(written)
    }
}

#[derive(Debug, Clone)]
pub struct Segment {
    pub header: ProgramHeader,
}

impl Segment {
    pub fn new(header: ProgramHeader) -> Self {
        Self { header: header }
    }
}
