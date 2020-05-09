use nom::types::CompleteStr;

/// Opcode encapsulates the various operation codes.
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Opcode {
    HLT,
    LOAD,
    ADD,
    SUB,
    MUL,
    DIV,
    JMP,
    JMPF,
    JMPB,
    EQ,
    NEQ,
    GT,
    LT,
    GTQ,
    LTQ,
    JEQ,
    JNEQ,
    IGL,
}

impl From<u8> for Opcode {
    fn from(v: u8) -> Self {
        match v {
            0 => Opcode::HLT,
            1 => Opcode::LOAD,
            2 => Opcode::ADD,
            3 => Opcode::SUB,
            4 => Opcode::MUL,
            5 => Opcode::DIV,
            6 => Opcode::JMP,
            7 => Opcode::JMPF,
            8 => Opcode::JMPB,
            9 => Opcode::EQ,
            10 => Opcode::NEQ,
            11 => Opcode::GT,
            12 => Opcode::LT,
            13 => Opcode::GTQ,
            14 => Opcode::LTQ,
            15 => Opcode::JEQ,
            16 => Opcode::JNEQ,
            _ => Opcode::IGL,
        }
    }
}

impl<'a> From<CompleteStr<'a>> for Opcode {
    fn from(v: CompleteStr<'a>) -> Self {
        let lower = v.to_lowercase();
        match CompleteStr(&lower) {
            CompleteStr("hlt") => Opcode::HLT,
            CompleteStr("load") => Opcode::LOAD,
            CompleteStr("add") => Opcode::ADD,
            CompleteStr("sub") => Opcode::SUB,
            CompleteStr("mul") => Opcode::MUL,
            CompleteStr("div") => Opcode::DIV,
            CompleteStr("jmp") => Opcode::JMP,
            CompleteStr("jmpf") => Opcode::JMPF,
            CompleteStr("jmpb") => Opcode::JMPB,
            CompleteStr("eq") => Opcode::EQ,
            CompleteStr("neq") => Opcode::NEQ,
            CompleteStr("gt") => Opcode::GT,
            CompleteStr("lt") => Opcode::LT,
            CompleteStr("gtq") => Opcode::GTQ,
            CompleteStr("ltq") => Opcode::LTQ,
            CompleteStr("jeq") => Opcode::JEQ,
            CompleteStr("jneq") => Opcode::JNEQ,
            _ => Opcode::IGL,
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Instruction {
    opcode: Opcode,
}

impl Instruction {
    pub fn new(opcode: Opcode) -> Instruction {
        Instruction { opcode }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_hlt() {
        let opcode = Opcode::HLT;
        assert_eq!(opcode, Opcode::HLT);
    }

    #[test]
    fn test_create_igl() {
        let opcode = Opcode::IGL;
        assert_eq!(opcode, Opcode::IGL);
    }
}
