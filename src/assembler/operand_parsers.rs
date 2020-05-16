use nom::digit;
use nom::types::CompleteStr;

use crate::assembler::label_parsers::label_usage;
use crate::assembler::register_parsers::register;
use crate::assembler::Token;

named!(pub operand<CompleteStr, Token>,
    alt!(
        integer_operand |
        label_usage |
        register |
        irstring
    )
);

// Parser for integer numbers, which we preface with `#` in our assembly language.
// Example: #100.
named!(pub integer_operand<CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!("#") >>
            reg_num: digit >>
            (
                Token::IntegerOperand{value: reg_num.parse::<i32>().unwrap()}
            )
        )
    )
);

// Parser for string contstants in the form of `my_string .asciiz '<string>'`.
// Strings are null-terminated (hence the MIPS `.asciiz` directive).
named!(irstring<CompleteStr, Token>,
    do_parse!(
        tag!("'") >>
        content: take_until!("'") >>
        tag!("'") >>
        (
            Token::IrString{ name: content.to_string() }
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer_operand() {
        let result = integer_operand(CompleteStr("#10"));
        assert_eq!(result.is_ok(), true);
        let (rest, value) = result.unwrap();
        assert_eq!(rest, CompleteStr(""));
        assert_eq!(value, Token::IntegerOperand { value: 10 });

        let result = integer_operand(CompleteStr("10"));
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn test_parse_string_operand() {
        let result = irstring(CompleteStr("'This is a test'"));
        assert_eq!(result.is_ok(), true);
    }
}
