use crate::assembler::{PIE_HEADER_LENGTH, PIE_HEADER_PREFIX};
use crate::instruction::Opcode;

pub struct VM {
    // Since we know the number of registers at compile time, we use an array instead
    // of a vector.
    /// The registers of the VM.
    pub registers: [i32; 32],
    /// Program counter that is used to track which byte is executing.
    pc: usize,
    /// Bytecode of the program.
    pub program: Vec<u8>,
    heap: Vec<u8>,
    /// The remainder of a division operation.
    remainder: u32,
    /// Contains the result of the last comparison operation.
    equal_flag: bool,
}

impl VM {
    /// Returns a new `VM` instance.
    pub fn new() -> VM {
        VM {
            registers: [0; 32],
            program: vec![],
            heap: vec![],
            pc: 65,
            remainder: 0,
            equal_flag: false,
        }
    }

    pub fn run(&mut self) {
        let mut is_done = false;
        while !is_done {
            is_done = self.execute_instruction();
        }
    }

    /// Executes one instruction. Meant to allow for more controlled execution of the VM.
    pub fn run_once(&mut self) {
        self.execute_instruction();
    }

    fn execute_instruction(&mut self) -> bool {
        // If our program counter has exceeded the length of the program itself,
        // something has gone awry.
        if self.pc >= self.program.len() {
            return true;
        }
        match self.decode_opcode() {
            Opcode::HLT => {
                println!("HLT encountered");
                return true;
            }
            Opcode::LOAD => {
                // We cast to usize so we can use it as an index into the array.
                let register = self.next_8_bits() as usize;
                let number = self.next_16_bits() as u16;
                // Our registers are i32s, so we need to cast it.
                self.registers[register] = number as i32;
            }
            Opcode::ADD => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 + register2;
            }
            Opcode::SUB => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 - register2;
            }
            Opcode::MUL => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 * register2;
            }
            Opcode::DIV => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 / register2;
                self.remainder = (register1 % register2) as u32;
            }
            Opcode::JMP => {
                let target = self.registers[self.next_8_bits() as usize];
                self.pc = target as usize;
            }
            Opcode::JMPF => {
                let value = self.registers[self.next_8_bits() as usize];
                self.pc += value as usize;
            }
            Opcode::JMPB => {
                let value = self.registers[self.next_8_bits() as usize];
                self.pc -= value as usize;
            }
            Opcode::EQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 == register2;
                self.next_8_bits();
            }
            Opcode::NEQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 != register2;
                self.next_8_bits();
            }
            Opcode::GT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 > register2;
                self.next_8_bits();
            }
            Opcode::LT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 < register2;
                self.next_8_bits();
            }
            Opcode::GTQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 >= register2;
                self.next_8_bits();
            }
            Opcode::LTQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.equal_flag = register1 <= register2;
                self.next_8_bits();
            }
            Opcode::JEQ => {
                let target = self.registers[self.next_8_bits() as usize];
                if self.equal_flag {
                    self.pc = target as usize;
                }
            }
            Opcode::JNEQ => {
                let target = self.registers[self.next_8_bits() as usize];
                if !self.equal_flag {
                    self.pc = target as usize;
                }
            }
            Opcode::ALOC => {
                let register = self.next_8_bits() as usize;
                let bytes = self.registers[register];
                let new_end = self.heap.len() as i32 + bytes;
                self.heap.resize(new_end as usize, 0);
            }
            Opcode::INC => {
                let register = self.next_8_bits() as usize;
                self.registers[register] += 1;
            }
            Opcode::DEC => {
                let register = self.next_8_bits() as usize;
                self.registers[register] -= 1;
            }
            _ => {
                println!("Unrecognized opcode found! Terminating");
                return true;
            }
        }
        false
    }

    fn decode_opcode(&mut self) -> Opcode {
        let opcode = Opcode::from(self.program[self.pc]);
        self.pc += 1;
        opcode
    }

    /// Reads the next 8 bits of the program.
    fn next_8_bits(&mut self) -> u8 {
        let result = self.program[self.pc];
        self.pc += 1;
        result
    }

    /// Reads the next 16 bits of the program.
    fn next_16_bits(&mut self) -> u16 {
        let result = ((self.program[self.pc] as u16) << 8) | (self.program[self.pc + 1] as u16);
        self.pc += 2;
        result
    }

    /// Adds a byte to the program.
    pub fn add_byte(&mut self, byte: u8) {
        self.program.push(byte);
    }

    /// Adds multiple bytes to the program.
    pub fn add_bytes(&mut self, bytes: Vec<u8>) {
        for byte in bytes {
            self.add_byte(byte);
        }
    }

    /// Processes the header of bytecode the VM wants to execute.
    fn verify_header(&self) -> bool {
        if self.program[0..4] != PIE_HEADER_PREFIX {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_vm() -> VM {
        VM::new()
    }

    fn prepend_header(mut b: Vec<u8>) -> Vec<u8> {
        let mut prepension = vec![];
        for byte in PIE_HEADER_PREFIX.iter() {
            prepension.push(byte.clone());
        }
        while prepension.len() <= PIE_HEADER_LENGTH {
            prepension.push(0);
        }
        prepension.append(&mut b);
        prepension
    }

    #[test]
    fn test_create_vm() {
        let test_vm = get_test_vm();
        assert_eq!(test_vm.registers[0], 0);
    }

    #[test]
    fn test_opcode_hlt() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![0, 0, 0, 0];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run_once();
        assert_eq!(test_vm.pc, 66);
    }

    #[test]
    fn test_opcode_igl() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![200, 0, 0, 0];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run_once();
        assert_eq!(test_vm.pc, 66);
    }

    #[test]
    fn test_load_opcode() {
        let mut test_vm = get_test_vm();
        // Remember, this is how we represent 500 using two u8s in little endian format.
        test_vm.program = vec![1, 0, 1, 244];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run_once();
        assert_eq!(test_vm.registers[0], 500);
    }

    #[test]
    fn test_add_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![2, 8, 5, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.registers[5] = 3;
        test_vm.registers[8] = 7;
        test_vm.run_once();
        assert_eq!(test_vm.registers[2], 10);
    }

    #[test]
    fn test_sub_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![3, 8, 5, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.registers[5] = 3;
        test_vm.registers[8] = 7;
        test_vm.run_once();
        assert_eq!(test_vm.registers[2], 4);
    }

    #[test]
    fn test_mul_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![4, 8, 5, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.registers[5] = 3;
        test_vm.registers[8] = 7;
        test_vm.run_once();
        assert_eq!(test_vm.registers[2], 21);
    }

    #[test]
    fn test_div_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![5, 8, 5, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.registers[5] = 3;
        test_vm.registers[8] = 7;
        test_vm.run_once();
        assert_eq!(test_vm.registers[2], 2);
        assert_eq!(test_vm.remainder, 1);
    }

    #[test]
    fn test_jmp_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![6, 0, 0, 0];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.registers[0] = 1;
        test_vm.run_once();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_jmpf_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![7, 0, 0, 0, 3, 0, 0, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.registers[0] = 2;
        test_vm.run_once();
        assert_eq!(test_vm.pc, 69);
    }

    #[test]
    fn test_jmpb_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![8, 0, 0, 0, 3, 0, 0, 2];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.registers[0] = 2;
        test_vm.run_once();
        assert_eq!(test_vm.pc, 65);
    }

    #[test]
    fn test_eq_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 10;
        test_vm.registers[1] = 10;
        test_vm.program = vec![9, 0, 1, 0, 9, 0, 1, 0];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 20;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_neq_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 10;
        test_vm.registers[1] = 11;
        test_vm.program = vec![10, 0, 1, 0, 10, 0, 1, 0];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_gt_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 11;
        test_vm.registers[1] = 10;
        test_vm.program = vec![11, 0, 1, 0, 11, 0, 1, 0];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 11;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_lt_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 9;
        test_vm.registers[1] = 10;
        test_vm.program = vec![12, 0, 1, 0, 12, 0, 1, 0];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 9;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_gtq_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 11;
        test_vm.registers[1] = 10;
        test_vm.program = vec![13, 0, 1, 0, 13, 0, 1, 0, 13, 0, 1, 0];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 11;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 12;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_ltq_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 11;
        test_vm.registers[1] = 12;
        test_vm.program = vec![14, 0, 1, 0, 14, 0, 1, 0, 14, 0, 1, 0];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 11;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, true);
        test_vm.registers[1] = 10;
        test_vm.run_once();
        assert_eq!(test_vm.equal_flag, false);
    }

    #[test]
    fn test_jeq_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 7;
        test_vm.equal_flag = true;
        test_vm.program = vec![15, 0, 0, 0, 16, 0, 0, 0, 16, 0, 0, 0];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run_once();
        assert_eq!(test_vm.pc, 7);
    }

    #[test]
    fn test_jneq_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 7;
        test_vm.equal_flag = false;
        test_vm.program = vec![16, 0, 0, 0, 17, 0, 0, 0, 17, 0, 0, 0];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run_once();
        assert_eq!(test_vm.pc, 7);
    }

    #[test]
    fn test_aloc_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 1024;
        test_vm.program = vec![17, 0, 0, 0];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run_once();
        assert_eq!(test_vm.heap.len(), 1024);
    }

    #[test]
    fn test_inc_opdcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 1;
        test_vm.program = vec![18, 0, 0, 0];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run_once();
        assert_eq!(test_vm.registers[0], 2);
    }

    #[test]
    fn test_dec_opdcode() {
        let mut test_vm = get_test_vm();
        test_vm.registers[0] = 1;
        test_vm.program = vec![19, 0, 0, 0];
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run_once();
        assert_eq!(test_vm.registers[0], 0);
    }
}
