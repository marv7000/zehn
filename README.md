[![crates.io](https://img.shields.io/crates/v/zehn.svg)](https://crates.io/crates/zehn)
[![Documentation](https://docs.rs/zehn/badge.svg)](https://docs.rs/zehn)

> *Note:* This library is not stable yet and a work in progress.

# zehn
A library for reading and writing ELF binaries

## Usage
```rs
use std::io::File;
use zehn::object::*;

let mut file = File::read("my_executable").unwrap();
let obj = Object::read(&mut file);
for (symbol_name, symbol) in obj.symbols {
    println!("Name: {}, Size: {}", symbol_name, symbol.sym_size);
}
```

## Roadmap
### v0.1
- [X] ELF Parsing
- [X] ELF Writing

### v0.2
- [ ] Adding sections
- [ ] Adding segments
