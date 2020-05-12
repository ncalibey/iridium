use nom::types::CompleteStr;

use crate::assembler::program_parsers::*;
use crate::instruction::Opcode;

pub mod directive_parsers;
pub mod instruction_parsers;
pub mod label_parsers;
pub mod opcode_parsers;
pub mod operand_parsers;
pub mod program_parsers;
pub mod register_parsers;

#[derive(Debug, PartialEq)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
    LabelDeclaration { name: String },
    LabelUsage { name: String },
    Directive { name: String },
}

/// The `AssemblerPhase` enum details which phase an `Assembler` is in. It can be only one of
/// two variants: `First` or `Second`.
#[derive(Debug)]
pub enum AssemblerPhase {
    First,
    Second,
}

/// The Assembler is a *two-pass* assembler, meaning that it takes two passes over the code
/// when assembling. The first is for passing the program string to the parser and constructing
/// a symbol table, and the second is for converting it into the bytecode that can be read by
/// the VM.
#[derive(Debug)]
pub struct Assembler {
    /// Denotes which phase the Assembler is in.
    pub phase: AssemblerPhase,
    /// The symbol table used for storing parsed labels.
    pub symbols: SymbolTable,
}

impl Assembler {
    /// Returns a new `Assembler`.
    pub fn new() -> Assembler {
        Assembler {
            phase: AssemblerPhase::First,
            symbols: SymbolTable::new(),
        }
    }

    /// Assembles the code into bytecode that is readable by the VM in two-passes.
    pub fn assemble(&mut self, raw: &str) -> Option<Vec<u8>> {
        // Pass the raw &str to the parser. Match to see if the program was parsed correctly.
        match program(CompleteStr(raw)) {
            Ok((_remainder, program)) => {
                self.process_first_phase(&program);
                Some(self.process_second_phase(&program))
            }
            Err(e) => {
                println!("There was an error assembling the code: {:?}", e);
                None
            }
        }
    }

    /// First pass over the code which extracts any labels.
    fn process_first_phase(&mut self, p: &Program) {
        self.extract_labels(p);
        self.phase = AssemblerPhase::Second;
    }

    /// Second pass over the code which converts the instructions and symbols into bytecode (`Vec<u8>`).
    fn process_second_phase(&mut self, p: &Program) -> Vec<u8> {
        let mut program = vec![];
        for i in &p.instructions {
            let mut bytes = i.to_bytes(&self.symbols);
            program.append(&mut bytes);
        }
        program
    }

    /// Extracts the labels for the program by looking for label declarations (e.g. `some_name:<opcode>...`).
    fn extract_labels(&mut self, p: &Program) {
        let mut c = 0;
        for i in &p.instructions {
            if i.is_label() {
                match i.label_name() {
                    Some(name) => {
                        let symbol = Symbol::new(name, SymbolType::Label, c);
                        self.symbols.add_symbol(symbol);
                    }
                    None => {}
                };
                c += 4;
            }
        }
    }
}

/// Represents a symbol within the parsed code.
#[derive(Debug)]
pub struct Symbol {
    /// The name of the symbol.
    name: String,
    /// The byte offset the symbol is for.
    offset: u32,
    /// The type of symbol.
    symbol_type: SymbolType,
}

impl Symbol {
    /// Returns a new `Symbol`.
    pub fn new(name: String, symbol_type: SymbolType, offset: u32) -> Symbol {
        Symbol {
            name,
            symbol_type,
            offset,
        }
    }
}

/// The various types of symbols that can be parsed from a program.
#[derive(Debug)]
pub enum SymbolType {
    /// Labels that are used for naming specific instructions.
    /// E.g. `test1: LOAD $0 #100`.
    Label,
}

#[derive(Debug)]
/// A table for holding all symbols parsed from a program.
///
/// TODO: implement as HashMap.
pub struct SymbolTable {
    symbols: Vec<Symbol>,
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
                return Some(symbol.offset);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VM;

    #[test]
    fn test_symbol_table() {
        let mut sym = SymbolTable::new();
        let new_symbol = Symbol::new("test".to_string(), SymbolType::Label, 12);
        sym.add_symbol(new_symbol);
        assert_eq!(sym.symbols.len(), 1);
        let v = sym.symbol_value("test");
        assert_eq!(v.is_some(), true);
        let v = v.unwrap();
        assert_eq!(v, 12);
        let v = sym.symbol_value("does_not_exist");
        assert_eq!(v.is_some(), false);
    }

    #[test]
    fn test_assemble_program() {
        let mut asm = Assembler::new();
        let test_string =
            "load $0 #100\nload $1 #1\nload $2 #0\ntest: inc $0\nneq $0 $2\njmpe @test\nhlt";
        let program = asm.assemble(test_string).unwrap();
        let mut vm = VM::new();
        assert_eq!(program.len(), 21);
        vm.add_bytes(program);
        assert_eq!(vm.program.len(), 21);
    }
}
