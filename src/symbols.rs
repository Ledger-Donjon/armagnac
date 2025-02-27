use std::fs;

use object::{Object, ObjectSymbol, SymbolKind};

#[derive(Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub offset: u64,
    pub size: u64,
}

pub trait SymbolResolver {
    fn resolve(&self, address: u64) -> Option<Symbol>;
}

impl Symbol {
    /// Returns `true` if given address is in the symbol range.
    pub fn contains(&self, address: u64) -> bool {
        address >= self.offset && address < self.offset + self.size
    }
}

pub struct BasicSymbolResolver {
    symbols: Vec<Symbol>,
}

impl BasicSymbolResolver {
    pub fn new() -> Self {
        Self {
            symbols: Vec::new(),
        }
    }

    pub fn add_symbols_from_file(&mut self, path: &str) {
        let bin_data = fs::read(path).unwrap();
        let obj_file = object::File::parse(&*bin_data).unwrap();
        self.add_symbols(&obj_file);
    }

    pub fn add_symbols(&mut self, file: &object::File) {
        self.symbols.extend(file.symbols().filter_map(|s| {
            if s.kind() == SymbolKind::Text {
                if let Ok(name) = s.name() {
                    Some(Symbol {
                        name: name.into(),
                        offset: s.address() & !1,
                        size: s.size(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }));
    }

    pub fn add_symbol(&mut self, name: &str, offset: u64, size: u64) {
        self.symbols.push(Symbol {
            name: name.into(),
            offset,
            size,
        });
    }

    pub fn resolve_name(&self, address: u64) -> Option<String> {
        self.symbols
            .iter()
            .find(|s| s.contains(address))
            .map(|s| s.name.clone())
    }
}

impl SymbolResolver for BasicSymbolResolver {
    fn resolve(&self, address: u64) -> Option<Symbol> {
        self.symbols.iter().find(|s| s.contains(address)).cloned()
    }
}
