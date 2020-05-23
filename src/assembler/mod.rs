use nom::types::CompleteStr;

use crate::assembler::assembler_errors::AssemblerError;
use crate::assembler::instruction_parsers::AssemblerInstruction;
use crate::assembler::program_parsers::*;
use crate::assembler::symbols::{Symbol, SymbolTable, SymbolType};
use crate::instruction::Opcode;

pub mod assembler_errors;
pub mod directive_parsers;
pub mod instruction_parsers;
pub mod label_parsers;
pub mod opcode_parsers;
pub mod operand_parsers;
pub mod program_parsers;
pub mod register_parsers;
pub mod symbols;

#[derive(Debug, PartialEq)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
    LabelDeclaration { name: String },
    LabelUsage { name: String },
    Directive { name: String },
    IrString { name: String },
}

pub const PIE_HEADER_PREFIX: [u8; 4] = [45, 50, 49, 45];
pub const PIE_HEADER_LENGTH: usize = 64;

/// The `AssemblerPhase` enum details which phase an `Assembler` is in. It can be only one of
/// two variants: `First` or `Second`.
#[derive(Debug, PartialEq)]
pub enum AssemblerPhase {
    First,
    Second,
}

impl Default for AssemblerPhase {
    fn default() -> Self {
        AssemblerPhase::First
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssemblerSection {
    Data { starting_instruction: Option<u32> },
    Code { starting_instruction: Option<u32> },
    Unknown,
}

impl<'a> From<&'a str> for AssemblerSection {
    fn from(name: &str) -> AssemblerSection {
        match name {
            "data" => AssemblerSection::Data {
                starting_instruction: None,
            },
            "code" => AssemblerSection::Code {
                starting_instruction: None,
            },
            _ => AssemblerSection::Unknown,
        }
    }
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
    /// The read-only data section that is used for storing constants.
    pub ro: Vec<u8>,
    /// The compiled bycode generated from the assembly instructions.
    pub bytecode: Vec<u8>,
    /// The current offset of the read-only section.
    ro_offset: u32,
    /// A list of all sections seen in the code.
    sections: Vec<AssemblerSection>,
    /// The current section of the Assembler.
    current_section: Option<AssemblerSection>,
    /// The current instruction of the Assembler.
    current_instruction: u32,
    /// Errors encountered when assembling the code. These are presented to the user
    /// at the end of assembly.
    errors: Vec<AssemblerError>,
}

impl Assembler {
    /// Returns a new `Assembler`.
    pub fn new() -> Assembler {
        Assembler {
            phase: AssemblerPhase::First,
            symbols: SymbolTable::new(),
            ro: vec![],
            bytecode: vec![],
            ro_offset: 0,
            sections: vec![],
            current_section: None,
            current_instruction: 0,
            errors: vec![],
        }
    }

    /// Assembles the code into bytecode that is readable by the VM in two-passes.
    pub fn assemble(&mut self, raw: &str) -> Result<Vec<u8>, Vec<AssemblerError>> {
        // Pass the raw &str to the parser. Match to see if the program was parsed correctly.
        match program(CompleteStr(raw)) {
            Ok((_remainder, program)) => {
                // First we grab the header for later.
                let mut assembled_program = self.write_pie_header();
                // First pass.
                self.process_first_phase(&program);

                // Check for errors. If there are any, return and don't do the second pass.
                if !self.errors.is_empty() {
                    return Err(self.errors.clone());
                }

                // Ensure we have at least one data section and one code section.
                if self.sections.len() != 2 {
                    println!("Did not find at least two sections.");
                    self.errors.push(AssemblerError::InsufficientSections);
                    return Err(self.errors.clone());
                }

                // Second pass.
                let mut body = self.process_second_phase(&program);
                // Merge the header with the body vector.
                assembled_program.append(&mut body);
                Ok(assembled_program)
            }
            Err(e) => {
                println!("There was an error assembling the code: {:?}", e);
                Err(vec![AssemblerError::ParseError {
                    error: e.to_string(),
                }])
            }
        }
    }

    /// First pass over the code which extracts any label declarations and directives and puts them
    /// into segments.
    fn process_first_phase(&mut self, p: &Program) {
        // We iterate over all the instructions even though we are hunting for label declarations.
        for i in &p.instructions {
            if i.is_label() {
                if self.current_section.is_some() {
                    // If we've already hit a segment header (e.g., `.code`), then we're all good to
                    // process the label.
                    self.process_label_declaration(&i);
                } else {
                    // If we haven't hit a segment yet, then we have an error since we have a label
                    // outside of a segment header.
                    self.errors.push(AssemblerError::NoSegmentDeclarationFound {
                        instruction: self.current_instruction,
                    });
                }
            }

            if i.is_directive() {
                self.process_directive(i);
            }
            self.current_instruction += 1;
        }
        self.phase = AssemblerPhase::Second;
    }

    /// Second pass over the code which converts the instructions and symbols into bytecode (`Vec<u8>`).
    fn process_second_phase(&mut self, p: &Program) -> Vec<u8> {
        // Restart the counting of instructions.
        self.current_instruction = 0;
        // We put the bytecode up for execution in a separate Vec so we can do some
        // post-processing before merging it with the header and read-only sections.
        let mut program = vec![];
        // Same as first-phase, but now we care about opcodes and directives.
        for i in &p.instructions {
            if i.is_opcode() {
                // Opcodes know how to properly transform themselves into 32-bits, so we can just
                // call `to_bytes` and append it to our program.
                let mut bytes = i.to_bytes(&self.symbols);
                program.append(&mut bytes);
            }
            if i.is_directive() {
                // We are looking for different types of directives than gathered on the first pass.
                // As such, we let the directive check which phase we're in and decide what to do.
                self.process_directive(i);
            }
            self.current_instruction += 1;
        }
        program
    }

    /// Processes label declarations such as `hello: .asciiz 'Hello'`.
    fn process_label_declaration(&mut self, i: &AssemblerInstruction) {
        // Check if the label is None or String.
        let name = match i.get_label_name() {
            Some(name) => name,
            None => {
                self.errors
                    .push(AssemblerError::StringConstantDeclaredWithoutLabel {
                        instruction: self.current_instruction,
                    });
                return;
            }
        };

        // Check if label is already in use (i.e. has an entry in the symbol table).
        if self.symbols.has_symbol(&name) {
            self.errors.push(AssemblerError::SymbolAlreadyDeclared);
            return;
        }

        let symbol = Symbol::new(name, SymbolType::Label);
        self.symbols.add_symbol(symbol);
    }

    /// Processes directives such as `.code`.
    fn process_directive(&mut self, i: &AssemblerInstruction) {
        let directive_name = match i.get_directive_name() {
            Some(name) => name,
            None => {
                println!("Directive has an invalid name: {:?}", i);
                return;
            }
        };
        // Now check for any operands.
        if i.has_operands() {
            // If yes, determine the directive.
            match directive_name.as_ref() {
                "asciiz" => {
                    self.handle_asciiz(i);
                }
                _ => {
                    self.errors.push(AssemblerError::UnknownDirectiveFound {
                        directive: directive_name.clone(),
                    });
                    return;
                }
            }
        } else {
            // If not, then it's a section header.
            self.process_section_header(&directive_name);
        }
    }

    /// Handles a declaration of a section header (e.g. `.code`).
    fn process_section_header(&mut self, header_name: &str) {
        let new_section: AssemblerSection = header_name.into();
        // Only specific names are allowed.
        if new_section == AssemblerSection::Unknown {
            println!("Found a section header that is unknown: {:#?}", header_name);
            return;
        }
        self.sections.push(new_section.clone());
        self.current_section = Some(new_section);
    }

    /// Handles a declaration of a null-terminated string (e.g. `hello: .asciiz 'Hello!'`)
    fn handle_asciiz(&mut self, i: &AssemblerInstruction) {
        // Being a constant declaration, this is only meaningful in the first pass.
        if self.phase != AssemblerPhase::First {
            return;
        }

        // Operand1 will have the entire string we need to read into RO memory.
        match i.get_string_constant() {
            Some(s) => {
                match i.get_label_name() {
                    Some(name) => {
                        self.symbols.set_symbol_offset(&name, self.ro_offset);
                    }
                    None => {
                        // This would be someting typing: .asciiz 'Hello!'
                        println!("Found a string constant with no associated label!");
                        return;
                    }
                };
                // We'll read the string into the read-only section byte-by-byte.
                for byte in s.as_bytes() {
                    self.ro.push(*byte);
                    self.ro_offset += 1;
                }
                // This is the null termination bit we are using to indicate a string has ended.
                self.ro.push(0);
                self.ro_offset += 1;
            }
            None => {
                // This just means someone typed `.asciiz` for some reason.
                println!("String constant following an .asciiz was empty");
            }
        };
    }

    /// Extracts the labels for the program by looking for label declarations (e.g. `some_name:<opcode>...`).
    fn extract_labels(&mut self, p: &Program) {
        let mut c = 0;
        for i in &p.instructions {
            if i.is_label() {
                match i.get_label_name() {
                    Some(name) => {
                        let symbol = Symbol::new_with_offset(name, SymbolType::Label, c);
                        self.symbols.add_symbol(symbol);
                    }
                    None => {}
                };
                c += 4;
            }
        }
    }

    /// Writes the PIE header which is 4 bytes long. The remaining 60 bytes are padded with 0s
    /// so they can be used later on.
    fn write_pie_header(&self) -> Vec<u8> {
        let mut header = vec![];
        for byte in PIE_HEADER_PREFIX.iter() {
            header.push(byte.clone());
        }
        while header.len() < PIE_HEADER_LENGTH {
            header.push(0 as u8);
        }
        header
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::VM;

    #[test]
    fn test_assemble_program() {
        let mut asm = Assembler::new();
        let test_string =
            ".data\n.code\nload $0 #100\nload $1 #1\nload $2 #0\ntest: inc $0\nneq $0 $2\njmpe @test\nhlt";
        let program = asm.assemble(test_string).unwrap();
        let mut vm = VM::new();
        assert_eq!(program.len(), 92);
        vm.add_bytes(program);
        assert_eq!(vm.program.len(), 92);
    }

    #[test]
    fn test_symbol_table() {
        let mut sym = SymbolTable::new();
        let new_symbol = Symbol::new_with_offset(String::from("test"), SymbolType::Label, 12);
        sym.add_symbol(new_symbol);
        assert_eq!(sym.symbols.len(), 1);
        let v = sym.symbol_value("test");
        assert_eq!(true, v.is_some());
        let v = v.unwrap();
        assert_eq!(v, 12);
        let v = sym.symbol_value("does_not_exist");
        assert_eq!(v.is_some(), false);
    }

    #[test]
    fn test_ro_data() {
        let mut asm = Assembler::new();
        let test_string = ".data\ntest: .asciiz 'This is a test'\n.code\n";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), true);
    }

    #[test]
    fn test_bad_ro_data() {
        let mut asm = Assembler::new();
        let test_string = ".code\ntest: .asciiz 'This is a test'\n.wrong\n";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), false);
    }

    #[test]
    fn test_first_phase_no_segment() {
        let mut asm = Assembler::new();
        let test_string = "hello: .asciiz 'Fail'";
        let result = program(CompleteStr(test_string));
        assert_eq!(result.is_ok(), true);
        let (_, p) = result.unwrap();
        asm.process_first_phase(&p);
        assert_eq!(asm.errors.len(), 1);
    }

    #[test]
    fn test_first_phase_inside_segment() {
        let mut asm = Assembler::new();
        let test_string = ".data\ntest: .asciiz 'Hello'";
        let result = program(CompleteStr(test_string));
        assert_eq!(result.is_ok(), true);
        let (_, p) = result.unwrap();
        asm.process_first_phase(&p);
        assert_eq!(asm.errors.len(), 0);
    }
}
