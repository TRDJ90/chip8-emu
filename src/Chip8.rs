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
            let kk = ((opcode & 0x000F) >> SHIFT0) as u8;
            
            let nnn = opcode & 0xFFF;
            //let kk = opcode & 0x000F;

            match (addr, x, y, kk) {
                (0, 0, 0, 0)        => { return; },
                (0xE0, _, _, _)     => self.clear_display(),
                (0, 0, 0xE, 0)      => self.ret(),
                (0x1, _, _, _)      => self.jump_to(nnn),
                (0x2, _, _, _)      => self.call(nnn),
                
                (0x3, _, _, _)      => self.skip_vx_equal_to_byte(x, kk),
                (0x4, _, _, _)      => self.skip_vx_not_equal_to_byte(x, kk),
                (0x5, _, _, _)      => self.skip_vx_equal_vy(x, y),
                
                (0x6, _, _, _)      => self.set_vx_to_value(x, kk),
                (0x7, _, _, _)      => self.add_value_vx(x, kk),

                (0x8, _, _, _)      => match(x, y, kk) {
                    (_, _, 0x0)     => self.put_vy_in_vx(x, y), //or_vx_vy_put_result_in_vx
                    (_, _, 0x1)     => self.or_vx_vy_put_result_in_vx(x, y),
                    (_, _, 0x2)     => self.and_vx_vy_put_result_in_vx(x, y),
                    (_, _, 0x3)     => self.xor_vx_vy_put_result_in_vx(x, y),      
                    (_, _, 0x4)     => self.add_vx_vy_put_result_in_vx(x, y),
                    (_, _, 0x5)     => self.sub_vx_vy_put_result_in_vx(x, y),
                    (_, _, 0x6)     => self.shift_vx_right(x),
                    (_, _, 0x7)     => self.sub_vy_vx_put_result_in_vx(x, y),
                    (_, _, 0x8)     => self.shift_vx_left(x),
                    _               => todo!{"unknown opcode{:04x}", opcode},
                },
                (0x9, _, _, _)      => self.skip_vx_not_equal_vy(x, y),
                

                _                   => todo!{"unknown opcode {:04x}", opcode},
            }
        }
    }
}


// Seperate Impl block for the cpu operations
impl Chip8 {
    fn clear_display(&self) {
        // should clear monitor.
    }

    fn ret(&mut self) {
        if self.stack_pointer == 0 {
            panic!("Stack underflow");
        }

        self.stack_pointer -= 1;
        self.index_memory = self.stack[self.stack_pointer] as usize;
    }

    fn jump_to(&mut self, addr: u16) {
        self.index_memory = addr as usize;
    }

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

    // Skip instruction/operation when registers and value matches or not matchtes.
    fn skip_vx_equal_to_byte(&mut self, reg_index: u8, byte: u8) {
        let vx_value = self.registers[reg_index as usize];
        if vx_value == byte {
            self.index_memory += 2;
        }
    }

    fn skip_vx_not_equal_to_byte(&mut self, reg_index: u8, byte: u8) {
        let vx_value = self.registers[reg_index as usize];
        if vx_value != byte {
            self.index_memory += 2;
        }
    }

    fn skip_vx_equal_vy(&mut self, reg_x_index: u8, reg_y_index: u8) {
        let vx_value = self.registers[reg_x_index as usize];
        let vy_value = self.registers[reg_y_index as usize];
        if vx_value == vy_value {
            self.index_memory += 2;
        }
    }

    fn skip_vx_not_equal_vy(&mut self, reg_x_index: u8, reg_y_index: u8) {
        let vx_value = self.registers[reg_x_index as usize];
        let vy_value = self.registers[reg_y_index as usize];
        if vx_value != vy_value {
            self.index_memory += 2;
        }
    }

    // Set register values
    fn set_vx_to_value(&mut self, reg_x_index: u8, byte: u8) {
        self.registers[reg_x_index as usize] = byte;
    }

    fn add_value_vx(&mut self, reg_x_index: u8, byte: u8) {
        self.registers[reg_x_index as usize] += byte;
    }

    fn put_vy_in_vx(&mut self,  reg_x_index: u8, reg_y_index: u8) {
        let vy_value = self.registers[reg_y_index as usize];
        self.registers[reg_x_index as usize] = vy_value;
    }

    // bitwise operations
    fn or_vx_vy_put_result_in_vx(&mut self, reg_x_index: u8, reg_y_index: u8) {
        let vx_value = self.registers[reg_x_index as usize];
        let vy_value = self.registers[reg_y_index as usize];
        self.registers[reg_x_index as usize] = vx_value | vy_value;
    }

    fn and_vx_vy_put_result_in_vx(&mut self, reg_x_index: u8, reg_y_index: u8) {
        let vx_value = self.registers[reg_x_index as usize];
        let vy_value = self.registers[reg_y_index as usize];
        self.registers[reg_x_index as usize] = vx_value & vy_value;
    }

    fn xor_vx_vy_put_result_in_vx(&mut self, reg_x_index: u8, reg_y_index: u8) {
        let vx_value = self.registers[reg_x_index as usize];
        let vy_value = self.registers[reg_y_index as usize];
        self.registers[reg_x_index as usize] = vx_value ^ vy_value;
    }

    // Arithmetic operations
    fn add_vx_vy_put_result_in_vx(&mut self, reg_x_index: u8, reg_y_index: u8) {
        let vx_value = self.registers[reg_x_index as usize];
        let vy_value = self.registers[reg_y_index as usize];

        let (val, overflow_detected) = vx_value.overflowing_add(vy_value);
        self.registers[reg_x_index as usize] = val;

        if overflow_detected {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn sub_vx_vy_put_result_in_vx(&mut self, reg_x_index: u8, reg_y_index: u8) {
        let vx_value = self.registers[reg_x_index as usize];
        let vy_value = self.registers[reg_y_index as usize];

        if vx_value > vy_value {
            self.registers[0xF] = 1;
            let val = vx_value - vy_value;
            self.registers[reg_x_index as usize] = val;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn shift_vx_right(&mut self, reg_x_index: u8) {
        let vx_value = self.registers[reg_x_index as usize];
        if vx_value == 1 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[reg_x_index as usize] = vx_value / 2;
    }

    fn sub_vy_vx_put_result_in_vx(&mut self, reg_x_index: u8, reg_y_index: u8) {
        let vx_value = self.registers[reg_x_index as usize];
        let vy_value = self.registers[reg_y_index as usize];

        if vy_value > vx_value {
            self.registers[0xF] = 1;
            let val = vy_value - vx_value;
            self.registers[reg_x_index as usize] = val;
        } else {
            self.registers[0xF] = 0;
        }
    }

    fn shift_vx_left(&mut self, reg_x_index: u8) {
        let vx_value = self.registers[reg_x_index as usize];
        if vx_value == 0x80 {
            self.registers[0xF] = 1;
        } else {
            self.registers[0xF] = 0;
        }
        self.registers[reg_x_index as usize] = vx_value * 2;
    }
}