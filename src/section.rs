use std::io::{Read, Write};

use crate::object::{Class, Endianness};
use crate::util::ReadExt;
use crate::util::Result;
use crate::util::WriteExt;

pub mod shtype {
    /// Section header table entry (unused)
    pub const SHT_NULL: u32 = 0x0;
    /// Program data
    pub const SHT_PROGBITS: u32 = 0x1;
    /// Symbol table
    pub const SHT_SYMTAB: u32 = 0x2;
    /// String table
    pub const SHT_STRTAB: u32 = 0x3;
    /// Relocation entries with addends
    pub const SHT_RELA: u32 = 0x4;
    /// Symbol hash table
    pub const SHT_HASH: u32 = 0x5;
    /// Dynamic linking information
    pub const SHT_DYNAMIC: u32 = 0x6;
    /// Notes
    pub const SHT_NOTE: u32 = 0x7;
    /// Program space with no data (bss)
    pub const SHT_NOBITS: u32 = 0x8;
    /// Relocation entries, no addends
    pub const SHT_REL: u32 = 0x9;
    /// Reserved
    pub const SHT_SHLIB: u32 = 0x0A;
    /// Dynamic linker symbol table
    pub const SHT_DYNSYM: u32 = 0x0B;
    /// Array of constructors
    pub const SHT_INIT_ARRAY: u32 = 0x0E;
    /// Array of destructors
    pub const SHT_FINI_ARRAY: u32 = 0x0F;
    /// Array of pre-constructors
    pub const SHT_PREINIT_ARRAY: u32 = 0x10;
    /// Section group
    pub const SHT_GROUP: u32 = 0x11;
    /// Extended section indices
    pub const SHT_SYMTAB_SHNDX: u32 = 0x12;
    /// Number of defined types.
    pub const SHT_NUM: u32 = 0x13;
    /// Start OS-specific.
    pub const SHT_LOOS: u32 = 0x60000000;
}

#[derive(Debug, Clone)]
pub struct SectionHeader {
    /// An offset to a string in the .shstrtab section that represents the name of this section.
    pub sh_name: u32,
    /// Identifies the type of this header.
    pub sh_type: u32,
    /// Identifies the attributes of the section.
    pub sh_flags: u64,
    /// Virtual address of the section in memory, for sections that are loaded.
    pub sh_addr: u64,
    /// Offset of the section in the file image.
    pub sh_offset: u64,
    /// Size in bytes of the section in the file image. May be 0.
    pub sh_size: u64,
    /// Contains the section index of an associated section. This field is used for several purposes, depending on the type of section.
    pub sh_link: u32,
    /// Contains extra information about the section. This field is used for several purposes, depending on the type of section.
    pub sh_info: u32,
    /// Contains the required alignment of the section. This field must be a power of two.
    pub sh_addralign: u64,
    /// Contains the size, in bytes, of each entry, for sections that contain fixed-size entries. Otherwise, this field contains zero.
    pub sh_entsize: u64,
}

impl SectionHeader {
    pub fn read(class: &Class, endian: &Endianness, mut buf: impl Read) -> Result<Self> {
        let prog = match class {
            Class::Bits32 => Self {
                sh_name: buf.read_u32(endian)?,
                sh_type: buf.read_u32(endian)?,
                sh_flags: buf.read_u32(endian)? as u64,
                sh_addr: buf.read_u32(endian)? as u64,
                sh_offset: buf.read_u32(endian)? as u64,
                sh_size: buf.read_u32(endian)? as u64,
                sh_link: buf.read_u32(endian)?,
                sh_info: buf.read_u32(endian)?,
                sh_addralign: buf.read_u32(endian)? as u64,
                sh_entsize: buf.read_u32(endian)? as u64,
            },
            Class::Bits64 => Self {
                sh_name: buf.read_u32(endian)?,
                sh_type: buf.read_u32(endian)?,
                sh_flags: buf.read_u64(endian)?,
                sh_addr: buf.read_u64(endian)?,
                sh_offset: buf.read_u64(endian)?,
                sh_size: buf.read_u64(endian)?,
                sh_link: buf.read_u32(endian)?,
                sh_info: buf.read_u32(endian)?,
                sh_addralign: buf.read_u64(endian)?,
                sh_entsize: buf.read_u64(endian)?,
            },
        };
        Ok(prog)
    }

    pub fn write(&self, class: &Class, endian: &Endianness, mut buf: impl Write) -> Result<usize> {
        let mut written = 0;
        match class {
            Class::Bits32 => {
                written += buf.write_u32(endian, self.sh_name)?;
                written += buf.write_u32(endian, self.sh_type)?;
                written += buf.write_u32(endian, self.sh_flags as u32)?;
                written += buf.write_u32(endian, self.sh_addr as u32)?;
                written += buf.write_u32(endian, self.sh_offset as u32)?;
                written += buf.write_u32(endian, self.sh_size as u32)?;
                written += buf.write_u32(endian, self.sh_link)?;
                written += buf.write_u32(endian, self.sh_info)?;
                written += buf.write_u32(endian, self.sh_addralign as u32)?;
                written += buf.write_u32(endian, self.sh_entsize as u32)?;
            }
            Class::Bits64 => {
                written += buf.write_u32(endian, self.sh_name)?;
                written += buf.write_u32(endian, self.sh_type)?;
                written += buf.write_u64(endian, self.sh_flags)?;
                written += buf.write_u64(endian, self.sh_addr)?;
                written += buf.write_u64(endian, self.sh_offset)?;
                written += buf.write_u64(endian, self.sh_size)?;
                written += buf.write_u32(endian, self.sh_link)?;
                written += buf.write_u32(endian, self.sh_info)?;
                written += buf.write_u64(endian, self.sh_addralign)?;
                written += buf.write_u64(endian, self.sh_entsize)?;
            }
        }
        Ok(written)
    }
}

#[derive(Debug, Clone)]
pub struct Section {
    pub header: SectionHeader,
    pub body: Vec<u8>,
}

impl Section {
    pub fn new(header: SectionHeader) -> Self {
        Self {
            header: header,
            body: vec![],
        }
    }
}
