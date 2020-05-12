use nom::types::CompleteStr;

use crate::assembler::label_parsers::label_declaration;
use crate::assembler::opcode_parsers::*;
use crate::assembler::operand_parsers::*;
use crate::assembler::Token;

#[derive(Debug, PartialEq)]
pub struct AssemblerInstruction {
    pub opcode: Option<Token>,
    pub label: Option<Token>,
    pub directive: Option<Token>,
    pub operand1: Option<Token>,
    pub operand2: Option<Token>,
    pub operand3: Option<Token>,
}

impl AssemblerInstruction {
    /// Converts assembler instructions to a vector of u8.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut results = vec![];
        match self.opcode {
            Some(Token::Op { code }) => {
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

        while results.len() < 4 {
            results.push(0);
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

named!(instruction_combined<CompleteStr, AssemblerInstruction>,
    do_parse!(
        l: opt!(label_declaration) >>
        o: opcode >>
        o1: opt!(operand) >>
        o2: opt!(operand) >>
        o3: opt!(operand) >>
        (
            AssemblerInstruction{
                opcode: Some(o),
                label: l,
                directive: None,
                operand1: o1,
                operand2: o2,
                operand3: o3,
            }
        )
    )
);

// Will try to parse out any of the Instruction forms.
named!(pub instruction<CompleteStr, AssemblerInstruction>,
    do_parse!(
        ins: alt!(
            instruction_combined
        ) >>
        (
            ins
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::Opcode;

    #[test]
    fn test_parse_instruction_form_one() {
        let result = instruction_combined(CompleteStr("load $0 #100\n"));
        assert_eq!(result.is_ok(), true);
        let (rest, assembler_instruction) = result.unwrap();
        assert_eq!(rest, CompleteStr(""));
        assert_eq!(
            assembler_instruction,
            AssemblerInstruction {
                opcode: Some(Token::Op { code: Opcode::LOAD }),
                label: None,
                directive: None,
                operand1: Some(Token::Register { reg_num: 0 }),
                operand2: Some(Token::IntegerOperand { value: 100 }),
                operand3: None,
            },
        );
    }

    #[test]
    fn test_parse_instruction_form_two() {
        let result = instruction_combined(CompleteStr("hlt"));
        assert_eq!(result.is_ok(), true);
        let (rest, assembler_instruction) = result.unwrap();
        assert_eq!(rest, CompleteStr(""));
        assert_eq!(
            assembler_instruction,
            AssemblerInstruction {
                opcode: Some(Token::Op { code: Opcode::HLT }),
                label: None,
                directive: None,
                operand1: None,
                operand2: None,
                operand3: None,
            }
        );
    }

    #[test]
    fn test_parse_instruction_form_three() {
        let result = instruction_combined(CompleteStr("add $0 $1 $2\n"));
        assert_eq!(result.is_ok(), true);
        let (rest, assembler_instruction) = result.unwrap();
        assert_eq!(rest, CompleteStr(""));
        assert_eq!(
            assembler_instruction,
            AssemblerInstruction {
                opcode: Some(Token::Op { code: Opcode::ADD }),
                label: None,
                directive: None,
                operand1: Some(Token::Register { reg_num: 0 }),
                operand2: Some(Token::Register { reg_num: 1 }),
                operand3: Some(Token::Register { reg_num: 2 }),
            }
        )
    }
}
