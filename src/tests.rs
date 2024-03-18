use std::io::Cursor;

use crate::*;

#[test]
pub fn test_elf_read() {
    let mut cursor = Cursor::new(include_bytes!("../test/test_exe"));
    let bin = object::Object::read(&mut cursor).unwrap();

    assert_eq!(bin.header.e_ident.ei_magic, [0x7F, 0x45, 0x4C, 0x46]);
    assert_eq!(&bin.segments.len(), &13);
    assert_eq!(&bin.sections.len(), &38);
    assert!(&bin.shstrtab.is_some());
    assert!(&bin.strtab.is_some());
}
