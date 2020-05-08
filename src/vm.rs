use crate::instruction::Opcode;

pub struct VM {
    // Since we know the number of registers at compile time, we use an array instead
    // of a vector.
    /// The registers of the VM.
    registers: [i32; 32],
    /// Program counter that is used to track which byte is executing.
    pc: usize,
    /// Bytecode of the program.
    program: Vec<u8>,
    /// The remainder of a division operation.
    remainder: u32,
}

impl VM {
    /// Returns a new `VM` instance.
    pub fn new() -> VM {
        VM {
            registers: [0; 32],
            program: vec![],
            pc: 0,
            remainder: 0,
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
            return false;
        }
        match self.decode_opcode() {
            Opcode::HLT => {
                println!("HLT encountered");
                return false;
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
            _ => {
                println!("Unrecognized opcode found! Terminating");
                return false;
            }
        }
        true
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_vm() -> VM {
        VM::new()
    }

    #[test]
    fn test_create_vm() {
        let test_vm = get_test_vm();
        assert_eq!(test_vm.registers[0], 0);
    }

    #[test]
    fn test_opcode_hlt() {
        let mut test_vm = get_test_vm();
        let test_bytes = vec![0, 0, 0, 0];
        test_vm.program = test_bytes;
        test_vm.run_once();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_opcode_igl() {
        let mut test_vm = get_test_vm();
        let test_bytes = vec![200, 0, 0, 0];
        test_vm.program = test_bytes;
        test_vm.run_once();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_load_opcode() {
        let mut test_vm = get_test_vm();
        // Remember, this is how we represent 500 using two u8s in little endian format.
        test_vm.program = vec![1, 0, 1, 244];
        test_vm.run_once();
        assert_eq!(test_vm.registers[0], 500);
    }

    #[test]
    fn test_add_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![2, 8, 5, 2];
        test_vm.registers[5] = 3;
        test_vm.registers[8] = 7;
        test_vm.run_once();
        assert_eq!(test_vm.registers[2], 10);
    }

    #[test]
    fn test_sub_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![3, 8, 5, 2];
        test_vm.registers[5] = 3;
        test_vm.registers[8] = 7;
        test_vm.run_once();
        assert_eq!(test_vm.registers[2], 4);
    }

    #[test]
    fn test_mul_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![4, 8, 5, 2];
        test_vm.registers[5] = 3;
        test_vm.registers[8] = 7;
        test_vm.run_once();
        assert_eq!(test_vm.registers[2], 21);
    }

    #[test]
    fn test_div_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![5, 8, 5, 2];
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
        test_vm.registers[0] = 1;
        test_vm.run_once();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_jmpf_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![7, 0, 0, 0, 3, 0, 0, 2];
        test_vm.registers[0] = 2;
        test_vm.run_once();
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_jmpb_opcode() {
        let mut test_vm = get_test_vm();
        test_vm.program = vec![8, 0, 0, 0, 3, 0, 0, 2];
        test_vm.registers[0] = 2;
        test_vm.run_once();
        assert_eq!(test_vm.pc, 0);
    }
}
