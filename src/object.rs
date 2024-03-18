use indexmap::IndexMap;

use crate::{
    section::Section,
    segment::Segment,
    symbol::Symbol,
    util::{align_to, Result, WriteExt},
};

#[derive(Debug, Clone, Default)]
pub enum Class {
    Bits32 = 1,
    #[default]
    Bits64 = 2,
}

#[derive(Debug, Clone, Default)]
pub enum Endianness {
    #[default]
    Little = 1,
    Big = 2,
}

#[derive(Debug, Clone, Default)]
pub struct Ident {
    pub ei_magic: [u8; 4],
    pub ei_class: Class,
    pub ei_data: Endianness,
    pub ei_version: u8,
    pub ei_osabi: u8,
    pub ei_abiversion: u8,
    pub ei_pad: [u8; 7],
}

#[derive(Debug, Clone, Default)]
pub struct Header {
    pub e_ident: Ident,
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

/// A simplified ELF representation.
#[derive(Debug, Clone)]
pub struct Object {
    pub header: Header,
    pub segments: Vec<Segment>,
    pub sections: IndexMap<String, Section>,
    pub symbols: IndexMap<String, Symbol>,
    pub shstrtab: Option<Section>,
    pub strtab: Option<Section>,
}

impl Object {
    pub fn new() -> Self {
        Self {
            header: Header::default(),
            segments: Vec::new(),
            sections: IndexMap::new(),
            symbols: IndexMap::new(),
            shstrtab: None,
            strtab: None,
        }
    }

    pub fn get_sections(&self, sh_type: u32) -> Vec<&Section> {
        self.sections
            .iter()
            .map(|(_, x)| x)
            .filter(|x| x.header.sh_type == sh_type)
            .collect()
    }

    pub fn get_sections_mut(&mut self, sh_type: u32) -> Vec<&mut Section> {
        self.sections
            .iter_mut()
            .map(|(_, x)| x)
            .filter(|x| x.header.sh_type == sh_type)
            .collect()
    }

    /// Attempts to find a section by name.
    pub fn find_section(&self, name: &str) -> Option<&Section> {
        self.sections.get(name)
    }

    /// Attempts to find a section by name and returns its index.
    pub fn find_section_idx(&self, name: &str) -> Option<u16> {
        if let Some(x) = self.sections.get_index_of(name) {
            return Some(x as u16);
        }
        None
    }

    /// Attempts to find a section by name and gives a mutable reference to it.
    pub fn find_section_mut(&mut self, name: &str) -> Option<&mut Section> {
        self.sections.get_mut(name)
    }

    /// Gets a copy of all symbols contained in this object.
    pub fn get_symbols(&self) -> Vec<&Symbol> {
        self.symbols.iter().map(|(_, x)| x).collect()
    }

    /// Gets mutable references to all symbols contained in this object.
    pub fn get_symbols_mut(&mut self) -> Vec<&mut Symbol> {
        self.symbols.iter_mut().map(|(_, x)| x).collect()
    }

    /// Resolves internal references and offsets.
    pub(crate) fn update(&mut self) -> Result<()> {
        // Update program header sizes + offsets.
        let old_phnum = self.header.e_phnum;
        self.header.e_phnum = self.segments.len() as u16;
        // Update symbol table.
        {
            let mut symtab_data = Vec::new();
            for symbol in self.get_symbols() {
                symbol.write(
                    &self.header.e_ident.ei_class,
                    &self.header.e_ident.ei_data,
                    &mut symtab_data,
                )?;
            }
            let symtab = self.find_section_mut(".symtab").unwrap();
            symtab.header.sh_size = symtab_data.len() as u64;
            symtab.body = symtab_data;
        }

        // Update symbol string table.
        {
            let mut strtab_data = vec![0u8];
            let mut strtab_pos = 1; // Leave room for null strings.
            for (name, symbol) in &mut self.symbols {
                symbol.sym_name = strtab_pos as u32; // Update the name offset.
                strtab_pos += strtab_data.write_cstr(name)?;
            }
            let strtab = self.find_section_mut(".strtab").unwrap();
            strtab.header.sh_size = strtab_data.len() as u64;
            strtab.body = strtab_data;
        }

        // Update section string table.
        {
            let mut shstr_data = vec![0u8];
            let mut shstr_pos = 1; // Leave room for null strings.
            for (name, section) in &mut self.sections {
                shstr_data.write_cstr(name)?;
                section.header.sh_name = shstr_pos as u32;
                shstr_pos += name.len() + 1;
            }
            // TODO
            self.header.e_shstrndx = self.find_section_idx(".shstrtab").unwrap() as u16;
            let shstrtab = self.find_section_mut(".shstrtab").unwrap();
            shstrtab.body = shstr_data;
            shstrtab.header.sh_size = shstr_pos as u64;
        }

        // Update section sizes + offsets.
        // Get the amount of total sections.
        self.header.e_shnum = self.sections.len() as u16;
        let mut section_pos = self.header.e_ehsize as u64
            + (self.header.e_phentsize as u64 * self.header.e_phnum as u64);
        for (_, section) in &mut self.sections {
            // Align.
            section_pos = align_to(&section_pos, &section.header.sh_addralign);
            section.header.sh_offset = section_pos;
            // Add the size of this section to the current cursor.
            section_pos += section.body.len() as u64;
        }

        // Start of the symbol table is after all sections.
        section_pos = align_to(&section_pos, &16);
        self.header.e_shoff = section_pos;

        // Byte size of new program header elements.
        let new_phsize = self.header.e_phentsize * (self.header.e_phnum - old_phnum);

        self.segments
            .iter_mut()
            .for_each(|x| match x.header.p_type {
                // Update the (first) PHDR element in the program header table.
                6 => {
                    x.header.p_filesz = self.header.e_phnum as u64 * self.header.e_phentsize as u64;
                    x.header.p_memsz = self.header.e_phnum as u64 * self.header.e_phentsize as u64;
                }
                // Update the INTERP element in the program header table.
                3 => x.header.p_offset += new_phsize as u64,
                _ => (),
            });

        Ok(())
    }
}
