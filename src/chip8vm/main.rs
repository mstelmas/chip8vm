#[macro_use] extern crate log;
#[macro_use] extern crate bincode;
#[macro_use] extern crate serde_derive;
extern crate env_logger;
extern crate sdl2;
extern crate rand;
extern crate serde;
extern crate serde_json;

use std::env;
use std::fs;
use std::io::prelude::*;

mod chip8;

fn main() {
    let rom_path = env::args().nth(1).expect("Provide rom location!");

    env_logger::init();

    let code = fs::read(&rom_path).expect("Could not read file");

    info!("Starting Chip8 emulation for ROM at: {:#}", rom_path);

    let sdl_context = sdl2::init().unwrap();

    let display = chip8::display::Display::new(&sdl_context);
    let keypad = chip8::keypad::Keypad::new(&sdl_context);
    let mut chip8 = chip8::Chip8::new(display, keypad);
    chip8.load_rom(&code);
    chip8.run();
}
