use std::io::{Cursor, Read, Seek, SeekFrom, Write};

use crate::{
    object::{Class, Endianness, Header, Ident, Object},
    section::{Section, SectionHeader},
    segment::{ProgramHeader, Segment},
    symbol::Symbol,
    util::{ReadExt, Result, WriteExt},
};

impl Header {
    pub fn read(mut buf: impl Read) -> Result<Self> {
        let ident = Ident {
            ei_magic: match buf.read_bytes()? {
                [0x7F, 0x45, 0x4C, 0x46] => [0x7F, 0x45, 0x4C, 0x46],
                _ => todo!("Replace with custom error type."),
            },
            ei_class: match buf.read_u8()? {
                1 => Class::Bits32,
                2 => Class::Bits64,
                _ => todo!("Replace with custom error type."),
            },
            ei_data: match buf.read_u8()? {
                1 => Endianness::Little,
                2 => Endianness::Big,
                _ => todo!("Replace with custom error type."),
            },
            ei_version: buf.read_u8()?,
            ei_osabi: buf.read_u8()?,
            ei_abiversion: buf.read_u8()?,
            ei_pad: buf.read_bytes()?,
        };

        let header = match ident.ei_class {
            Class::Bits32 => Self {
                e_ident: ident.clone(),
                e_type: buf.read_u16(&ident.ei_data)?,
                e_machine: buf.read_u16(&ident.ei_data)?,
                e_version: buf.read_u32(&ident.ei_data)?,
                e_entry: buf.read_u32(&ident.ei_data)? as u64,
                e_phoff: buf.read_u32(&ident.ei_data)? as u64,
                e_shoff: buf.read_u32(&ident.ei_data)? as u64,
                e_flags: buf.read_u32(&ident.ei_data)?,
                e_ehsize: buf.read_u16(&ident.ei_data)?,
                e_phentsize: buf.read_u16(&ident.ei_data)?,
                e_phnum: buf.read_u16(&ident.ei_data)?,
                e_shentsize: buf.read_u16(&ident.ei_data)?,
                e_shnum: buf.read_u16(&ident.ei_data)?,
                e_shstrndx: buf.read_u16(&ident.ei_data)?,
            },
            Class::Bits64 => Self {
                e_ident: ident.clone(),
                e_type: buf.read_u16(&ident.ei_data)?,
                e_machine: buf.read_u16(&ident.ei_data)?,
                e_version: buf.read_u32(&ident.ei_data)?,
                e_entry: buf.read_u64(&ident.ei_data)?,
                e_phoff: buf.read_u64(&ident.ei_data)?,
                e_shoff: buf.read_u64(&ident.ei_data)?,
                e_flags: buf.read_u32(&ident.ei_data)?,
                e_ehsize: buf.read_u16(&ident.ei_data)?,
                e_phentsize: buf.read_u16(&ident.ei_data)?,
                e_phnum: buf.read_u16(&ident.ei_data)?,
                e_shentsize: buf.read_u16(&ident.ei_data)?,
                e_shnum: buf.read_u16(&ident.ei_data)?,
                e_shstrndx: buf.read_u16(&ident.ei_data)?,
            },
        };
        Ok(header)
    }

    /// Writes an Object header to a stream.
    ///
    /// # Example
    /// ```
    /// use crate::zehn::*;
    ///
    /// let header = object::Header::default();
    /// let mut buffer = Vec::new();
    /// header.write(&mut buffer).unwrap();
    /// assert_eq!(buffer.len(), 0x40);
    /// ```
    pub fn write(&self, mut buf: impl Write) -> Result<usize> {
        let mut written = 0;
        written += buf.write_bytes(&self.e_ident.ei_magic)?;
        written += buf.write_u8(self.e_ident.ei_class.clone() as u8)?;
        written += buf.write_u8(self.e_ident.ei_data.clone() as u8)?;
        written += buf.write_u8(self.e_ident.ei_version)?;
        written += buf.write_u8(self.e_ident.ei_osabi)?;
        written += buf.write_u8(self.e_ident.ei_abiversion)?;
        written += buf.write_bytes(&self.e_ident.ei_pad)?;

        written += buf.write_u16(&self.e_ident.ei_data, self.e_type)?;
        written += buf.write_u16(&self.e_ident.ei_data, self.e_machine)?;
        written += buf.write_u32(&self.e_ident.ei_data, self.e_version)?;
        match &self.e_ident.ei_class {
            Class::Bits32 => {
                written += buf.write_u32(&self.e_ident.ei_data, self.e_entry as u32)?;
                written += buf.write_u32(&self.e_ident.ei_data, self.e_phoff as u32)?;
                written += buf.write_u32(&self.e_ident.ei_data, self.e_shoff as u32)?;
            }
            Class::Bits64 => {
                written += buf.write_u64(&self.e_ident.ei_data, self.e_entry)?;
                written += buf.write_u64(&self.e_ident.ei_data, self.e_phoff)?;
                written += buf.write_u64(&self.e_ident.ei_data, self.e_shoff)?;
            }
        };
        written += buf.write_u32(&self.e_ident.ei_data, self.e_flags)?;
        written += buf.write_u16(&self.e_ident.ei_data, self.e_ehsize)?;
        written += buf.write_u16(&self.e_ident.ei_data, self.e_phentsize)?;
        written += buf.write_u16(&self.e_ident.ei_data, self.e_phnum)?;
        written += buf.write_u16(&self.e_ident.ei_data, self.e_shentsize)?;
        written += buf.write_u16(&self.e_ident.ei_data, self.e_shnum)?;
        written += buf.write_u16(&self.e_ident.ei_data, self.e_shstrndx)?;
        Ok(written)
    }
}

impl Object {
    pub fn read(mut input: impl Read + Seek) -> Result<Self> {
        let mut result = Object::new();
        let mut old_pos = 0;

        // Read header.
        input.seek(SeekFrom::Start(old_pos))?;
        result.header = Header::read(&mut input)?;

        // Read program headers.
        input.seek(SeekFrom::Start(result.header.e_phoff))?;
        for _ in 0..result.header.e_phnum {
            let header = ProgramHeader::read(
                &result.header.e_ident.ei_class,
                &result.header.e_ident.ei_data,
                &mut input,
            )?;
            let prog = Segment::new(header);
            result.segments.push(prog);
        }

        // Read sections.
        let mut sections = Vec::new();
        input.seek(SeekFrom::Start(result.header.e_shoff))?;
        for _ in 0..result.header.e_shnum {
            // Read section header.
            let section_header = SectionHeader::read(
                &result.header.e_ident.ei_class,
                &result.header.e_ident.ei_data,
                &mut input,
            )?;

            // Read section body.
            old_pos = input.stream_position()?;
            let mut section_body = vec![0u8; section_header.sh_size as usize];
            input.seek(SeekFrom::Start(section_header.sh_offset))?;
            input.read_exact(&mut section_body)?;

            let sect = Section {
                header: section_header,
                body: section_body,
            };
            sections.push(sect);
            input.seek(SeekFrom::Start(old_pos))?;
        }

        // Read section names.
        let shstrtab = &sections[result.header.e_shstrndx as usize];
        for sect in &sections {
            let mut body = &shstrtab.body[sect.header.sh_name as usize..];
            let name = &body.read_cstr()?;
            result.sections.insert(name.clone(), sect.clone());
        }
        result.shstrtab = Some(shstrtab.clone());

        // TODO
        // Read symbols.
        let symtab = &result.find_section(".symtab").unwrap();
        let mut cur_symtab = Cursor::new(&symtab.body);
        let mut symbols = Vec::new();
        for _ in 0..(symtab.header.sh_size / symtab.header.sh_entsize) {
            let sym = Symbol::read(
                &result.header.e_ident.ei_class,
                &result.header.e_ident.ei_data,
                &mut cur_symtab,
            )?;
            symbols.push(sym);
        }
        // Read symbol names.
        let strtab = result.find_section(".strtab").unwrap().clone();
        for sym in symbols {
            let mut body = &strtab.body[sym.sym_name as usize..];
            let name = &body.read_cstr()?;

            result.symbols.insert(name.clone(), sym.clone());
        }

        result.strtab = Some(strtab);

        input.seek(SeekFrom::Start(0))?;

        // Finalize.
        result.update()?;
        return Ok(result);
    }

    pub fn write(&mut self, mut output: impl Write + Seek) -> Result<()> {
        self.update()?;

        // Write header.
        self.header.write(&mut output)?;

        // Write ELF body.
        output.seek(SeekFrom::Start(self.header.e_phoff))?;
        // Write program headers.
        for seg in &self.segments {
            seg.header.write(
                &self.header.e_ident.ei_class,
                &self.header.e_ident.ei_data,
                &mut output,
            )?;
        }

        // Write section bodies.
        for (_, sect) in self.sections.iter() {
            output.write(&sect.body)?;
        }

        // Write section headers.
        output.seek(SeekFrom::Start(self.header.e_shoff))?;
        self.sections.iter().for_each(|(_, sect)| {
            _ = sect
                .header
                .write(
                    &self.header.e_ident.ei_class,
                    &self.header.e_ident.ei_data,
                    &mut output,
                )
                .unwrap();
        });

        return Ok(());
    }
}
