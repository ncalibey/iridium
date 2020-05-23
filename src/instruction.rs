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
    ALOC,
    INC,
    DEC,
    PRTS,
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
            17 => Opcode::ALOC,
            18 => Opcode::INC,
            19 => Opcode::DEC,
            20 => Opcode::PRTS,
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
            CompleteStr("prts") => Opcode::PRTS,
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
    fn test_create_load() {
        let opcode = Opcode::LOAD;
        assert_eq!(opcode, Opcode::LOAD);
    }

    #[test]
    fn test_create_add() {
        let opcode = Opcode::ADD;
        assert_eq!(opcode, Opcode::ADD);
    }

    #[test]
    fn test_create_sub() {
        let opcode = Opcode::SUB;
        assert_eq!(opcode, Opcode::SUB);
    }

    #[test]
    fn test_create_mul() {
        let opcode = Opcode::MUL;
        assert_eq!(opcode, Opcode::MUL);
    }

    #[test]
    fn test_create_div() {
        let opcode = Opcode::DIV;
        assert_eq!(opcode, Opcode::DIV);
    }

    #[test]
    fn test_create_jmp() {
        let opcode = Opcode::JMP;
        assert_eq!(opcode, Opcode::JMP);
    }

    #[test]
    fn test_create_jmpf() {
        let opcode = Opcode::JMPF;
        assert_eq!(opcode, Opcode::JMPF);
    }

    #[test]
    fn test_create_jmpb() {
        let opcode = Opcode::JMPB;
        assert_eq!(opcode, Opcode::JMPB);
    }

    #[test]
    fn test_create_eq() {
        let opcode = Opcode::EQ;
        assert_eq!(opcode, Opcode::EQ);
    }

    #[test]
    fn test_create_neq() {
        let opcode = Opcode::NEQ;
        assert_eq!(opcode, Opcode::NEQ);
    }

    #[test]
    fn test_create_gt() {
        let opcode = Opcode::GT;
        assert_eq!(opcode, Opcode::GT);
    }

    #[test]
    fn test_create_lt() {
        let opcode = Opcode::LT;
        assert_eq!(opcode, Opcode::LT);
    }

    #[test]
    fn test_create_gtq() {
        let opcode = Opcode::GTQ;
        assert_eq!(opcode, Opcode::GTQ);
    }

    #[test]
    fn test_create_ltq() {
        let opcode = Opcode::LTQ;
        assert_eq!(opcode, Opcode::LTQ);
    }

    #[test]
    fn test_create_jeq() {
        let opcode = Opcode::JEQ;
        assert_eq!(opcode, Opcode::JEQ);
    }

    #[test]
    fn test_create_igl() {
        let opcode = Opcode::IGL;
        assert_eq!(opcode, Opcode::IGL);
    }

    #[test]
    fn test_create_prts() {
        let opcode = Opcode::PRTS;
        assert_eq!(opcode, Opcode::PRTS);
    }

    #[test]
    fn test_create_jneq() {
        let opcode = Opcode::JNEQ;
        assert_eq!(opcode, Opcode::JNEQ);
    }

    #[test]
    fn test_str_to_opcode() {
        // Check lowercase.
        let opcode = Opcode::from(CompleteStr("load"));
        assert_eq!(opcode, Opcode::LOAD);
        // Check uppercase.
        let opcode = Opcode::from(CompleteStr("LOAD"));
        assert_eq!(opcode, Opcode::LOAD);
        let opcode = Opcode::from(CompleteStr("illegal"));
        assert_eq!(opcode, Opcode::IGL);
    }
}
