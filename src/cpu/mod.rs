use rand::prelude::*;

pub mod cpu {
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
        sound_reg: u8,
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
                sound_reg: 0
            }
        }

        pub fn load_default_sprites(&mut self) {
            // Place default sprites in memory 0x000 - 0x1FF
            self.memory[0..80].copy_from_slice(&DEFAULT_SPRITES);
        }

        fn get_opcode(&self) -> u16 {
            let index_mem = self.program_counter;
            let opcode_byte1 = self.memory[index_mem] as u16;
            let opcode_byte2 = self.memory[index_mem + 1] as u16;

            opcode_byte1 << SHIFT8 | opcode_byte2
        }

        pub fn cycle(&mut self) {
            loop {
                let opcode = self.get_opcode();
                self.program_counter += 2;

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
                    (0xA, _, _, _)      => self.put_addr_in_index_register(nnn),
                    (0xB, _, _, _)      => self.jump_to_addr_with_v0_offset(nnn),
                    
                    (0xC, _, _, _)      => self.store_random_value_in_vx(x, kk),
                    (0xD, _, _, _)      => todo!{"Need to integrate monitor"},
                    
                    (0xE, _, _, _)      => todo!{"Need to integrate keyboard"},
                    (0xf, _, _, _)      => match(x, y, kk) {
                        (_, _ , 0x7)    => self.put_delay_timer_in_reg_x(x),
                        (_, _ , 0xA)    => self.wait_on_key_press(x, kk),
                        (_, _ , 0x15)   => self.set_vx_as_delay_timer(x),
                        (_, _ , 0x18)   => self.set_vx_as_sound_timer(x),
                        (_, _ , 0x1E)   => self.set_add_vx_to_index_memory(x),
                        (_, _ , 0x29)   => todo!{"unknown opcode{:04x}", opcode},
                        (_, _ , 0x33)   => todo!{"unknown opcode{:04x}", opcode},
                        (_, _ , 0x55)   => self.store_reg_0_to_x_in_mem_at_i(x),
                        (_, _ , 0x65)   => self.read_memory_into_registers(x),
                        _               => todo!{"unknown opcode{:04x}", opcode},
                    }
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
            self.program_counter = self.stack[self.stack_pointer] as usize;
        }

        fn jump_to(&mut self, addr: u16) {
            self.program_counter = addr as usize;
        }

        fn call(&mut self, addr: u16) {
            let sp = self.stack_pointer;
            let stack = &mut self.stack;

            if sp > stack.len() {
                panic!("Stack overflow!")
            }

            stack[sp] = self.program_counter as u16;
            self.stack_pointer += 1;
            self.program_counter = addr as usize;
        }

        // Skip instruction/operation when registers and value matches or not matchtes.
        fn skip_vx_equal_to_byte(&mut self, reg_index: u8, byte: u8) {
            let vx_value = self.registers[reg_index as usize];
            if vx_value == byte {
                self.program_counter += 2;
            }
        }

        fn skip_vx_not_equal_to_byte(&mut self, reg_index: u8, byte: u8) {
            let vx_value = self.registers[reg_index as usize];
            if vx_value != byte {
                self.program_counter += 2;
            }
        }

        fn skip_vx_equal_vy(&mut self, reg_x_index: u8, reg_y_index: u8) {
            let vx_value = self.registers[reg_x_index as usize];
            let vy_value = self.registers[reg_y_index as usize];
            if vx_value == vy_value {
                self.program_counter += 2;
            }
        }

        fn skip_vx_not_equal_vy(&mut self, reg_x_index: u8, reg_y_index: u8) {
            let vx_value = self.registers[reg_x_index as usize];
            let vy_value = self.registers[reg_y_index as usize];
            if vx_value != vy_value {
                self.program_counter += 2;
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

        fn put_addr_in_index_register(&mut self, addr: u16) {
            self.index_memory = addr as usize;
        }

        fn jump_to_addr_with_v0_offset(&mut self, addr: u16) {
            let v0_value = self.registers[0] as u16;
            self.program_counter = (addr + v0_value) as usize;
        }

        fn store_random_value_in_vx(&mut self, value: u8, reg_x_index: u8) {
            let rng = rand::random::<u8>();
            self.registers[reg_x_index as usize] = rng & value;
        }

        fn put_delay_timer_in_reg_x(&mut self, reg_x_index: u8) {
            self.registers[reg_x_index as usize] = self.delay_reg;
        }

        fn wait_on_key_press(&mut self, reg_x_index: u8, value: u8) {
            todo!("integrate keyboard");
            //self.registers[reg_x_index as usize] = 1336; // wait on keypress.
        }

        fn set_vx_as_delay_timer(&mut self, reg_x_index: u8) {
            self.delay_reg = self.registers[reg_x_index as usize];
        }

        fn set_vx_as_sound_timer(&mut self, reg_x_index: u8) {
            self.sound_reg = self.registers[reg_x_index as usize];
        }

        fn set_add_vx_to_index_memory(&mut self, reg_x_index: u8) {
            self.index_memory += self.registers[reg_x_index as usize] as usize; 
        }

        fn store_reg_0_to_x_in_mem_at_i(&mut self, reg_x_index: u8) {
            let mut reg_value;

            for index in 0..=reg_x_index {
                reg_value = self.registers[index as usize];
                self.memory[self.index_memory + index as usize] = reg_value;
            }
        }

        fn read_memory_into_registers(&mut self, reg_x_index: u8) {
            let mut mem_value;

            for index in 0..=reg_x_index {
                mem_value = self.memory[self.index_memory + index as usize];
                self.registers[index as usize] = mem_value;
            }
        }
    }
}