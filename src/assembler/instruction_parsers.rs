use nom::multispace;
use nom::types::CompleteStr;

use crate::assembler::opcode_parsers::*;
use crate::assembler::operand_parsers::integer_operand;
use crate::assembler::register_parsers::register;
use crate::assembler::Token;

#[derive(Debug, PartialEq)]
pub struct AssemblerInstruction {
    opcode: Token,
    operand1: Option<Token>,
    operand2: Option<Token>,
    operand3: Option<Token>,
}

impl AssemblerInstruction {
    /// Converts assembler instructions to a vector of u8.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut results = vec![];
        match self.opcode {
            Token::Op { code } => {
                results.push(code as u8);
            }
            _ => {
                println!("Non-opcode found in opcode field");
                std::process::exit(1);
            }
        };

        for operand in vec![&self.operand1, &self.operand2, &self.operand3] {
            match operand {
                Some(t) => AssemblerInstruction::extract_operand(t, &mut results),
                None => {}
            }
        }

        results
    }

    fn extract_operand(t: &Token, results: &mut Vec<u8>) {
        match t {
            Token::Register { reg_num } => {
                results.push(*reg_num);
            }
            Token::IntegerOperand { value } => {
                let converted = *value as u16;
                let byte1 = converted;
                let byte2 = converted >> 8;
                // byte2 is pushed in first to satisfy the big endian/little endian rule.
                results.push(byte2 as u8);
                results.push(byte1 as u8);
            }
            _ => {
                println!("Opcode found in operand field");
                std::process::exit(1);
            }
        }
    }
}

// Will try to parse out any of the Instruction forms.
named!(pub instruction<CompleteStr, AssemblerInstruction>,
    do_parse!(
        ins: alt!(
            instruction_one |
            instruction_two
        ) >>
        (
            ins
        )
    )
);

// Handles instructions of the following form: <opcode> <register> <intger operand>
// Example: LOAD $0 #100
named!(instruction_one<CompleteStr, AssemblerInstruction>,
    do_parse!(
        o: opcode_load >>
        r: register >>
        i: integer_operand >>
        (
            AssemblerInstruction{
                opcode: o,
                operand1: Some(r),
                operand2: Some(i),
                operand3: None,
            }
        )
    )
);

// Handles instructions of the following form: <opcode>
// Example: HLT
named!(instruction_two<CompleteStr, AssemblerInstruction>,
    do_parse!(
        o: opcode_load >>
        opt!(multispace) >>
        (
            AssemblerInstruction{
                opcode: o,
                operand1: None,
                operand2: None,
                operand3: None,
            }
        )
    )
);

// Handles instructions of the folloing form: <opcode> <register> <register> <register>
// Example: ADD $0 $1 $2
named!(instruction_three<CompleteStr, AssemblerInstruction>,
    do_parse!(
        o: opcode_load >>
        r1: register >>
        r2: register >>
        r3: register >>
        (
            AssemblerInstruction{
                opcode: o,
                operand1: Some(r1),
                operand2: Some(r2),
                operand3: Some(r3),
            }
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::Opcode;

    #[test]
    fn test_parse_instruction_form_one() {
        let result = instruction_one(CompleteStr("load $0 #100\n"));
        assert_eq!(result.is_ok(), true);
        let (rest, assembler_instruction) = result.unwrap();
        assert_eq!(rest, CompleteStr(""));
        assert_eq!(
            assembler_instruction,
            AssemblerInstruction {
                opcode: Token::Op { code: Opcode::LOAD },
                operand1: Some(Token::Register { reg_num: 0 }),
                operand2: Some(Token::IntegerOperand { value: 100 }),
                operand3: None,
            },
        );
    }

    #[test]
    fn test_parse_instruction_form_two() {
        let result = instruction_two(CompleteStr("hlt\n"));
        assert_eq!(result.is_ok(), true);
        let (rest, assembler_instruction) = result.unwrap();
        assert_eq!(rest, CompleteStr(""));
        assert_eq!(
            assembler_instruction,
            AssemblerInstruction {
                opcode: Token::Op { code: Opcode::HLT },
                operand1: None,
                operand2: None,
                operand3: None,
            }
        );
    }

    #[test]
    fn test_parse_instruction_form_three() {
        let result = instruction_three(CompleteStr("add $0 $1 $2\n"));
        assert_eq!(result.is_ok(), true);
        let (rest, assembler_instruction) = result.unwrap();
        assert_eq!(rest, CompleteStr(""));
        assert_eq!(
            assembler_instruction,
            AssemblerInstruction {
                opcode: Token::Op { code: Opcode::ADD },
                operand1: Some(Token::Register { reg_num: 0 }),
                operand2: Some(Token::Register { reg_num: 1 }),
                operand3: Some(Token::Register { reg_num: 2 }),
            }
        )
    }
}
