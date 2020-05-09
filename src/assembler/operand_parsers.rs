use nom::digit;
use nom::types::CompleteStr;

use crate::assembler::Token;

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
}
