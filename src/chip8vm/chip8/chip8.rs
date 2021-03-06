use super::{Cpu, Interconnect, Display, Keypad, RemoteDbg, DbgMessage};
use super::remote_dbg::{Chip8Snapshots};
use super::mem_map;

use std::process;
use std::thread;
use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::prelude::*;
use std::io::{BufReader, BufWriter};
use std::sync::mpsc;

#[derive(PartialEq)]
enum VmState {
    CREATED,
    RUNNING,
    STOPPED
}

pub struct Chip8 {
    cpu: Cpu,
    interconnect: Interconnect,

    state: VmState
}

impl Chip8 {
    pub fn new(display: Display, keypad: Keypad) -> Self {
        let mut interconnect = Interconnect::new(display, keypad);
        Chip8::load_fonts(&mut interconnect);

        Chip8 {
            cpu: Cpu::new(),
            interconnect,
            state: VmState::CREATED
        }
    }

    fn load_fonts(interconnect: &mut Interconnect) {
        // TODO: extract somewhere else
        let font_set: Vec<u8> = vec![
            0xF0, 0x90, 0x90, 0x90, 0xF0, /* 0 */
            0x20, 0x60, 0x20, 0x20, 0x70, /* 1 */
            0xF0, 0x10, 0xF0, 0x80, 0xF0, /* 2 */
            0xF0, 0x10, 0xF0, 0x10, 0xF0, /* 3 */
            0x90, 0x90, 0xF0, 0x10, 0x10, /* 4 */
            0xF0, 0x80, 0xF0, 0x10, 0xF0, /* 5 */
            0xF0, 0x80, 0xF0, 0x90, 0xF0, /* 6 */
            0xF0, 0x10, 0x20, 0x40, 0x40, /* 7 */
            0xF0, 0x90, 0xF0, 0x90, 0xF0, /* 8 */
            0xF0, 0x90, 0xF0, 0x10, 0xF0, /* 9 */
            0xF0, 0x90, 0xF0, 0x90, 0x90, /* A */
            0xE0, 0x90, 0xE0, 0x90, 0xE0, /* B */
            0xF0, 0x80, 0x80, 0x80, 0xF0, /* C */
            0xE0, 0x90, 0x90, 0x90, 0xE0, /* D */
            0xF0, 0x80, 0xF0, 0x80, 0xF0, /* E */
            0xF0, 0x80, 0xF0, 0x80, 0x80, /* F */
        ];

        interconnect.write_memory(mem_map::FONTS_LOCATION, &font_set);
    }

    pub fn load_rom(&mut self, rom: &Vec<u8>) {
        self.interconnect.write_memory(mem_map::ROM_LOCATION, rom);
    }

    pub fn run(&mut self) {
        assert!(self.state == VmState::CREATED);

        let (sender, receiver) = mpsc::channel();
        let (sender2, receiver2) = mpsc::channel();

        RemoteDbg::init(sender, receiver2);

        self.state = VmState::RUNNING;

        loop {
            match receiver.try_recv() {
                Ok(message) => self.handle_dbg_message(message, &sender2),
                Err(_) => {}
            }

            if self.state == VmState::RUNNING {
                self.step();
            }
        }
    }

    fn step(&mut self) {
        match self.interconnect.keypad().poll() {
            Err(_) => process::exit(0),
            Ok(keypad_state) => {
                self.interconnect.keypad().update_state(keypad_state);
                self.cpu.execute_cycle(&mut self.interconnect);
            }
        }
    }

    // TODO: Extract debugging stuff
    fn handle_dbg_message(&mut self, message: DbgMessage, sender: &mpsc::Sender<Chip8Snapshots>) {
        debug!("Handling debugger message: {:?}", message);

        match message {
            DbgMessage::START => self.state = VmState::RUNNING,
            DbgMessage::STOP => self.state = VmState::STOPPED,
            DbgMessage::RESTART => self.cpu = Cpu::new(),
            DbgMessage::CPU => sender.send(Chip8Snapshots::CPU(self.cpu.snapshot())).unwrap(),
            DbgMessage::MEM(addr, size) => {
                let mut v = vec![];

                for i in 0..size {
                    v.push(self.interconnect.read_byte(addr + i as u16));
                }

                sender.send(Chip8Snapshots::MEM(v));
            },
            DbgMessage::STEP => {
                assert!(self.state == VmState::STOPPED);
                self.step();
            }
        }
    }
}
