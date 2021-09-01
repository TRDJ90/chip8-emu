const SHIFT0: u8 = 0;
const SHIFT4: u8 = 4;
const SHIFT8: u8 = 8;
const SHIFT12: u8 = 12;

const DEFAULT_SPRITES: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,   // 0
    0x20, 0x60, 0x20, 0x20, 0x70,   // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0,   // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0,   // 3
    0x90, 0x90, 0xF0, 0x10, 0x10,   // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0,   // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0,   // 6
    0xF0, 0x10, 0x20, 0x40, 0x40,   // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0,   // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0,   // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90,   // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0,   // B
    0xF0, 0x80, 0x80, 0x80, 0xF0,   // C
    0xE0, 0x90, 0x90, 0x90, 0xE0,   // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0,   // E
    0xF0, 0x80, 0xF0, 0x80, 0x80,   // F
];

#[derive(Debug)]
pub struct Chip8 {
    pub memory: [u8; 4096],
    pub registers: [u8; 16],
    stack: [u16; 16],
    stack_pointer: usize,
    program_counter: usize,
    index_memory: usize,
    delay_reg: u8,
    sound_ref: u8,
}

impl Chip8 {
    pub fn new() -> Chip8 {
        Chip8 { 
            memory: [0; 4096], 
            registers: [0; 16], 
            stack: [0; 16], 
            stack_pointer: 0, 
            program_counter: 0, 
            index_memory: 0, 
            delay_reg: 0, 
            sound_ref: 0
        }
    }

    pub fn load_default_sprites(&mut self) {
        // Place default sprites in memory 0x000 - 0x1FF
        self.memory[0..80].copy_from_slice(&DEFAULT_SPRITES);
    }

    fn get_opcode(&self) -> u16 {
        let index_mem = self.index_memory;
        let opcode_byte1 = self.memory[index_mem] as u16;
        let opcode_byte2 = self.memory[index_mem + 1] as u16;

        opcode_byte1 << SHIFT8 | opcode_byte2
    }

    pub fn cycle(&mut self) {
        loop {
            let opcode = self.get_opcode();
            self.index_memory += 2;

            let addr = ((opcode & 0xF000) >> SHIFT12) as u8;
            let x = ((opcode & 0x0F00) >> SHIFT8) as u8;
            let y = ((opcode & 0x00F0) >> SHIFT4) as u8;
            let n = ((opcode & 0x000F) >> SHIFT0) as u8;
            
            let nnn = opcode & 0xFFF;

            match (addr, x, y, n) {
                (0, 0, 0, 0) => { return; },
                (0, 0, 0xE, 0xE) => self.ret(),
                (0x2, _, _, _) => self.call(nnn),
                (0x8, _, _, 0x4) => self.add_xy(x, y),
                _ => todo!{"opcode {:04x}", opcode},
            }
        }
    }
}


// Seperate Impl block for the cpu operations
impl Chip8 {
    fn call(&mut self, addr: u16) {
        let sp = self.stack_pointer;
        let stack = &mut self.stack;

        if sp > stack.len() {
            panic!("Stack overflow!")
        }

        stack[sp] = self.index_memory as u16;
        self.stack_pointer += 1;
        self.index_memory = addr as usize;
    }

    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Stack underflow");
        }

        self.stack_pointer -= 1;
        self.index_memory = self.stack[self.stack_pointer] as usize;
    }

    fn add_xy(&mut self, x: u8, y: u8) {
        let arg1 = self.registers[x as usize];
        let arg2 = self.registers[y as usize];

        let (val, overflow_detected) = arg1.overflowing_add(arg2);
        self.registers[x as usize] = val;

        if overflow_detected {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }
}