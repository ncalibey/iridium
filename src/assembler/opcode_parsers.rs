use nom::types::CompleteStr;
use nom::*;

use crate::assembler::Token;
use crate::instruction::Opcode;

named!(pub opcode_load<CompleteStr, Token>,
    do_parse!(
        opcode: alpha1 >>
        (
            {
                Token::Op{code: Opcode::from(opcode)}
            }
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_opcode_load() {
        let result = opcode_load(CompleteStr("load"));
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(token, Token::Op { code: Opcode::LOAD });
        assert_eq!(rest, CompleteStr(""));

        let result = opcode_load(CompleteStr("aold"));
        assert_eq!(result.is_ok(), false);
    }
}
