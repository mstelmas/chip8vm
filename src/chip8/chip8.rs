use super::{Cpu, Interconnect, Display, Keypad};
use super::mem_map;

use std::process;

pub struct Chip8 {
    cpu: Cpu,
    interconnect: Interconnect
}

impl Chip8 {
    pub fn new(display: Display, keypad: Keypad) -> Self {
        let mut interconnect = Interconnect::new(display, keypad);
        Chip8::load_fonts(&mut interconnect);

        Chip8 {
            cpu: Cpu::new(),
            interconnect
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
        loop {
            self.step();
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
}
