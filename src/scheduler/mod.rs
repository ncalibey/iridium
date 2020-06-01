use crate::vm::{VMEvent, VM};
use std::thread;

#[derive(Default)]
pub struct Scheduler {
    next_pid: u32,
    max_pid: u32,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Self {
            next_pid: 0,
            max_pid: 50000,
        }
    }

    pub fn get_thread(&self, mut vm: VM) -> thread::JoinHandle<Vec<VMEvent>> {
        thread::spawn(move || vm.run())
    }
}
