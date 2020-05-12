use crate::assembler::{program_parsers::program, SymbolTable};
use crate::vm::VM;
use nom::types::CompleteStr;
use std;
use std::io;
use std::{fs::File, io::Read, io::Write, num::ParseIntError, path::Path};

/// The core structure of the Assembler REPL.
pub struct REPL {
    command_buffer: Vec<String>,
    /// The VM the REPL uses to execute code.
    vm: VM,
}

impl REPL {
    /// Returns a new assembly REPL.
    pub fn new() -> REPL {
        REPL {
            vm: VM::new(),
            command_buffer: vec![],
        }
    }

    pub fn run(&mut self) {
        println!("Welcome to Iridium! Let's be productive!");
        loop {
            let mut buffer = String::new();
            let stdin = io::stdin();

            print!(">>> ");
            io::stdout().flush().expect("Unable to flush stdout");

            stdin
                .read_line(&mut buffer)
                .expect("Unable to read line from user");
            let buffer = buffer.trim();

            // Store a copy of the command into the buffer.
            self.command_buffer.push(buffer.to_string());

            match buffer {
                ".quit" => {
                    println!("Farewell! Have a great day!");
                    std::process::exit(0);
                }
                ".history" => {
                    for command in &self.command_buffer {
                        println!("{}", command);
                    }
                }
                ".program" => {
                    println!("Listing instructions currently in VM's program vector:");
                    for instruction in &self.vm.program {
                        println!("{}", instruction);
                    }
                    println!("End of Program Listing");
                }
                ".registers" => {
                    println!("Listing registers and all contents:");
                    println!("{:#?}", self.vm.registers);
                    println!("End of Register Listing");
                }
                ".clear_program" => {
                    self.vm.program = vec![];
                    println!("Program has been cleared!");
                }
                ".load_file" => {
                    print!("Please enter the path to the file you wish to load: ");
                    io::stdout().flush().expect("Unable to flush stdout");
                    let mut tmp = String::new();
                    stdin
                        .read_line(&mut tmp)
                        .expect("Unable to read line from user");
                    let tmp = tmp.trim();
                    let filename = Path::new(&tmp);
                    let mut f = File::open(Path::new(&filename)).expect("File not found");
                    let mut contents = String::new();
                    f.read_to_string(&mut contents)
                        .expect("There was an error freading from the file");
                    let program = match program(CompleteStr(&contents)) {
                        Ok((_remainder, program)) => program,
                        Err(e) => {
                            println!("Unable to parse input: {:?}", e);
                            continue;
                        }
                    };
                    // TODO fix
                    let symbol_table = SymbolTable::new();
                    self.vm.program.append(&mut program.to_bytes(&symbol_table));
                }
                _ => {
                    let parsed_program = program(CompleteStr(buffer));
                    if !parsed_program.is_ok() {
                        println!("Unable to parse input");
                        continue;
                    }
                    let (_, result) = parsed_program.unwrap();
                    // TODO fix
                    let symbol_table = SymbolTable::new();
                    let bytecode = result.to_bytes(&symbol_table);

                    for byte in bytecode {
                        self.vm.add_byte(byte);
                    }
                    self.vm.run_once();
                }
            }
        }
    }

    /// Accepts a hexadecimal string *without* a leading `0x` and returns a `Vec<u8>`.
    /// Example for a LOAD command: `01 01 03 E8`.
    #[allow(dead_code)]
    fn parse_hex(&mut self, i: &str) -> Result<Vec<u8>, ParseIntError> {
        let split = i.split(" ").collect::<Vec<&str>>();
        let mut results: Vec<u8> = vec![];
        for hex_string in split {
            let byte = u8::from_str_radix(&hex_string, 16);
            match byte {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(results)
    }
}
