#[derive(Debug)]
pub struct Symbol {
    /// The name of the symbol.
    name: String,
    /// The byte offset the symbol is for.
    offset: Option<u32>,
    /// The type of symbol.
    symbol_type: SymbolType,
}

impl Symbol {
    /// Returns a new `Symbol`.
    pub fn new(name: String, symbol_type: SymbolType) -> Symbol {
        Self {
            name,
            symbol_type,
            offset: None,
        }
    }

    /// Returns a new `Symbol` for the given byte offset.
    pub fn new_with_offset(name: String, symbol_type: SymbolType, offset: u32) -> Symbol {
        Self {
            name,
            symbol_type,
            offset: Some(offset),
        }
    }
}

/// The various types of symbols that can be parsed from a program.
#[derive(Debug)]
pub enum SymbolType {
    /// Labels that are used for naming specific instructions.
    /// E.g. `test1: LOAD $0 #100`.
    Label,
    Integer,
    IrString,
}

/// A table for holding all symbols parsed from a program.
///
/// TODO: implement as HashMap.
#[derive(Debug)]
pub struct SymbolTable {
    pub symbols: Vec<Symbol>,
}

impl SymbolTable {
    /// Returns a new `SymbolTable`.
    pub fn new() -> SymbolTable {
        SymbolTable { symbols: vec![] }
    }

    /// Adds a symbol to the table.
    pub fn add_symbol(&mut self, s: Symbol) {
        self.symbols.push(s);
    }

    /// Returns the byte offset value of a symbol if found within the table.
    pub fn symbol_value(&self, s: &str) -> Option<u32> {
        for symbol in &self.symbols {
            if symbol.name == s {
                return symbol.offset;
            }
        }
        None
    }

    pub fn has_symbol(&self, s: &str) -> bool {
        for symbol in &self.symbols {
            if symbol.name == s {
                return true;
            }
        }
        false
    }

    pub fn set_symbol_offset(&mut self, s: &str, offset: u32) -> bool {
        for symbol in &mut self.symbols {
            if symbol.name == s {
                symbol.offset = Some(offset);
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VM;

    #[test]
    fn test_symbol_table() {
        let mut sym = SymbolTable::new();
        let new_symbol = Symbol::new_with_offset("test".to_string(), SymbolType::Label, 12);
        sym.add_symbol(new_symbol);
        assert_eq!(sym.symbols.len(), 1);
        let v = sym.symbol_value("test");
        assert_eq!(v.is_some(), true);
        let v = v.unwrap();
        assert_eq!(v, 12);
        let v = sym.symbol_value("does_not_exist");
        assert_eq!(v.is_some(), false);
    }
}
