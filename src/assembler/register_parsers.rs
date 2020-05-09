use nom::digit;
use nom::types::CompleteStr;

use crate::assembler::Token;

// `register` accepts a `CompleteStr` and returns one with a `Token`, or an `Error`.
named!(pub register <CompleteStr, Token>,
    // The `ws!` macro consumes any whitespace on either side of the register.
    ws!(
        // `do_parse!` allows us to chain parsers.
        do_parse!(
            // Use `tag!` too look for `$` and then pass the result.
            tag!("$") >>
            // `digit` stores the result of `tag!` into `reg_num` and parses the number.
            reg_num: digit >>
            (
                Token::Register{
                    reg_num: reg_num.parse::<u8>().unwrap()
                }
            )
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_register() {
        let result = register(CompleteStr("$0"));
        assert_eq!(result.is_ok(), true);
        let result = register(CompleteStr("0"));
        assert_eq!(result.is_ok(), false);
        let result = register(CompleteStr("$a"));
        assert_eq!(result.is_ok(), false);
    }
}
