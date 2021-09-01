mod chip8;

use crate::chip8::Chip8;

fn main() {
    let mut emulator = Chip8::new();
    //emulator.load_default_sprites();

    emulator.registers[0] = 5;
    emulator.registers[1] = 10;

    emulator.memory[0x000] = 0x21; emulator.memory[0x001] = 0x00;
    emulator.memory[0x002] = 0x21; emulator.memory[0x003] = 0x00;

    emulator.memory[0x100] = 0x80; emulator.memory[0x101] = 0x14;
    emulator.memory[0x102] = 0x80; emulator.memory[0x103] = 0x14;
    emulator.memory[0x104] = 0x00; emulator.memory[0x105] = 0xEE;

    emulator.cycle();

    assert_eq!(emulator.registers[0], 45);

    println!("5 + (10 * 2) + (10 * 2) = {}", emulator.registers[0]);
}
