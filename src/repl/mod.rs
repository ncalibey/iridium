use crate::assembler::Assembler;
use crate::assembler::{program_parsers::program, symbols::SymbolTable};
use crate::scheduler::Scheduler;
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
    asm: Assembler,
    scheduler: Scheduler,
}

impl REPL {
    /// Returns a new assembly REPL.
    pub fn new() -> REPL {
        REPL {
            vm: VM::new(),
            command_buffer: vec![],
            asm: Assembler::new(),
            scheduler: Scheduler::new(),
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
                    let contents = self.get_data_from_load();
                    if let Some(contents) = contents {
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
                    } else {
                        continue;
                    }
                }
                ".spawn" => {
                    let contents = self.get_data_from_load();
                    if let Some(contents) = contents {
                        match self.asm.assemble(&contents) {
                            Ok(mut assembled_program) => {
                                println!("Sending assembled program to VM");
                                self.vm.program.append(&mut assembled_program);
                                println!("{:#?}", self.vm.program);
                                self.scheduler.get_thread(self.vm.clone());
                            }
                            Err(errors) => {
                                for error in errors {
                                    println!("Unable to parse input: {}", error);
                                }
                                continue;
                            }
                        }
                    } else {
                        continue;
                    }
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

    fn get_data_from_load(&mut self) -> Option<String> {
        let stdin = io::stdin();
        println!("Please enter the path to the file you wish to load: ");
        io::stdout().flush().expect("Unable to flush stdout");
        let mut tmp = String::new();

        stdin
            .read_line(&mut tmp)
            .expect("Unable to read line from user");
        println!("Attempting to load program from file...");

        let tmp = tmp.trim();
        let filename = Path::new(&tmp);
        let mut f = match File::open(&filename) {
            Ok(f) => f,
            Err(e) => {
                println!("There was an error opening that file: {:?}", e);
                return None;
            }
        };
        let mut contents = String::new();
        match f.read_to_string(&mut contents) {
            Ok(_bytes_read) => Some(contents),
            Err(e) => {
                println!("there was an error reading that file: {:?}", e);
                None
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
