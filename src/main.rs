use std::{fs};
use macroquad::{shapes, window};
use macroquad::color::WHITE;
use macroquad::window::Conf;

const MEMORY_OFFSET: u16 = 0x200;

const KEYMAP: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

struct Chip8 {
    memory: [u8; 4096],
    stack: [u16; 16],
    display: [[u32; 32]; 64],
    registers: [u8; 16],
    index_register: u16,
    delay_timer: u8,
    sound_timer: u8,
    sp: u8,
    pc: u16,
    opcode: u16,
    keypad: [bool; 16],
}

impl Chip8 {
    fn default() -> Self {
        Chip8 {
            memory: [0; 4096],
            stack: [0; 16],
            display: [[0; 32]; 64],
            registers: [0; 16],
            index_register: 0,
            delay_timer: 0,
            sound_timer: 0,
            sp: 0,
            pc: MEMORY_OFFSET,
            opcode: 0,
            keypad: [false; 16],
        }
    }

    fn init_keymap(&mut self) {
        self.memory[0x050..0x0A0].copy_from_slice(&KEYMAP);
    }

    fn load_rom(&mut self, rom: &str) {
        let byte_data = fs::read(rom).expect("Failed to read rom");
        self.memory[0x200..0x200 + byte_data.len()].copy_from_slice(&byte_data);
    }

    fn rand_byte() -> u8 {
        rand::random::<u8>()
    }

    fn fetch(&mut self) -> u16 {
        let op = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[self.pc as usize + 1] as u16);
        self.pc += 2;
        op
    }

    fn cls(&mut self) {
        println!("cls()");
        self.display = [[0; 32]; 64];
    }
    
    fn jmp(&mut self, addr: u16) {
        println!("jmp({:X})", addr);
        self.pc = addr;
    }
    
    fn call(&mut self, addr: u16) {
        println!("Called call()")
    }
    
    fn call_sub(&mut self, addr: u16) {
        println!("call_sub()");
        self.pc = addr;
        self.stack[self.sp as usize] = self.pc;
        self.sp += 1;
    }
    
    fn ret(&mut self) {
        println!("ret()");
        self.pc = self.stack[self.sp as usize];
        self.sp -= 1;
    }
    
    fn set_reg(&mut self, reg: u8, val: u8) {
        println!("set_reg({:X}, {:X})", reg, val);
        self.registers[reg as usize] = val;
    }
    
    fn add_reg(&mut self, reg: u8, val: u8) {
        println!("add_reg({:X}, {:X})", reg, val);
        self.registers[reg as usize] += val;
    }
    
    fn set_index(&mut self, val: u16) {
        println!("set_index({:X})", val);
        self.index_register = val;
        println!("Data at index: {:X}", self.memory[self.index_register as usize]);
    }
    
    fn eq(&mut self, reg: u8, val: u8) {
        println!("eq({:X}, {:X})", reg, val);
        if self.registers[reg as usize] == val {
            self.pc += 2;
        }
    }

    fn ne(&mut self, reg: u8, val: u8) {
        println!("ne({:X}, {:X})", reg, val);
        if self.registers[reg as usize] != val {
            self.pc += 2;
        }
    }

    fn reg_eq(&mut self, vx: u8, vy: u8) {
        println!("reg_eq({:X}, {:X})", vx, vy);
        if self.registers[vx as usize] == self.registers[vy as usize] {
            self.pc += 2;
        }
    }

    fn reg_cp(&mut self, vx: u8, vy: u8) {
        println!("reg_cp({:X}, {:X})", vx, vy);
        self.registers[vx as usize] = self.registers[vy as usize];
    }

    fn reg_or(&mut self, vx: u8, vy: u8) {
        println!("reg_or({:X}, {:X})", vx, vy);
        self.registers[vx as usize] |= self.registers[vy as usize];
    }

    fn reg_and(&mut self, vx: u8, vy: u8) {
        println!("reg_and({:X}, {:X})", vx, vy);
        self.registers[vx as usize] &= self.registers[vy as usize];
    }

    fn vxor(&mut self, vx: u8, vy: u8) {
        println!("vxor({:X}, {:X})", vx, vy);
        self.registers[vx as usize] ^= self.registers[vy as usize];
    }

    fn reg_add(&mut self, vx: u8, vy: u8) {
        println!("reg_add({:X}, {:X})", vx, vy);
        let sum = (self.registers[vx as usize] + self.registers[vy as usize]) as u16;
        if sum > 255 {
            self.registers[0xF] = 1;
        }

        self.registers[vx as usize] = (sum & 0xFF) as u8;
    }

    fn reg_sub(&mut self, vx: u8, vy: u8) {
        println!("reg_sub({:X}, {:X})", vx, vy);
        if self.registers[vx as usize] > self.registers[vy as usize] {
            self.registers[0xF] = 1;
        }

        self.registers[vx as usize] -= self.registers[vy as usize];
    }

    fn reg_shr(&mut self, vx: u8, vy: u8) {
        println!("reg_shr({:X}, {:X})", vx, vy);
        let lsb = self.registers[vx as usize] & 0b1;
        self.registers[vx as usize] >>= 1;
        self.registers[0xF] = lsb;
    }

    fn reg_sub_inv(&mut self, vx: u8, vy: u8) {
        println!("reg_sub_inv({:X}, {:X})", vx, vy);
        if self.registers[vx as usize] < self.registers[vy as usize] {
            self.registers[0xF] = 1;
        }
        self.registers[vx as usize] = self.registers[vy as usize] - self.registers[vx as usize];
    }
    
    fn reg_shl(&mut self, vx: u8, vy: u8) {
        println!("reg_shl({:X}, {:X})", vx, vy);
        let msb = self.registers[vx as usize] & 0b10000000;
        self.registers[vx as usize] <<= 1;
        self.registers[0xF] = msb;
    }

    fn jne(&mut self, vx: u8, vy: u8) {
        if self.registers[vx as usize] != self.registers[vy as usize] {
            self.pc += 2;
        }
    }

    fn jmp_v0(&mut self, val: u16) {
        self.pc = val + self.registers[0] as u16;
    }

    fn vx_rand(&mut self, vx: u8, val: u8) {
        self.registers[vx as usize] = Chip8::rand_byte() & val;
    }

    fn keydown(&mut self, vx: u8) {
        if self.keypad[self.registers[vx as usize] as usize] {
            self.pc += 2;
        }
    }

    fn key_not_down(&mut self, vx: u8) {
        if !self.keypad[self.registers[vx as usize] as usize] {
            self.pc += 2;
        }
    }

    fn get_delay(&mut self, vx: u8) {
        self.registers[vx as usize] = self.delay_timer;
    }

    fn set_delay(&mut self, vx: u8) {
        self.delay_timer = self.registers[vx as usize];
    }

    fn set_sound(&mut self, vx: u8) {
        self.sound_timer = self.registers[vx as usize];
    }

    fn add_to_i(&mut self, vx: u8) {
        self.index_register += self.registers[vx as usize] as u16;
    }

    fn set_sprite(&mut self, vx: u8) {
        self.index_register = 0x50 + self.registers[vx as usize] as u16;
    }

    fn fill_mem(&mut self, vx: u8) {
        for i in 0x0..=vx {
            // self.registers[i as usize] = self.memory[self.index_register as usize + i as usize];
            self.memory[self.index_register as usize + i as usize] = self.registers[i as usize];
        }
    }
    
    fn fill_reg(&mut self, vx: u8) {
        for i in 0x0..=vx {
            self.registers[i as usize] = self.memory[self.index_register as usize + i as usize];
        }
    }
    
    fn draw(&mut self, x: u8, y: u8, height: u8) {
        println!("draw({}, {}, {})", x, y, height);


        let x_pos = self.registers[x as usize] as usize;
        let y_pos = self.registers[y as usize] as usize;

        println!("Pos: {}, {}", x_pos, y_pos);

        self.registers[0xF] = 0;
        
        for row in 0..height as usize {
            let row_bits = self.memory[self.index_register as usize + row];
            for col in 0..8 {
                let pixel = row_bits >> (7 - col) & 0x1;
                let screen_pixel = &mut self.display[(x_pos + col) % 64][(y_pos + row) % 32];

                if pixel == 0xF {
                    if pixel == *screen_pixel as u8 {
                        self.registers[0xF] = 1;
                    }
                }
                
                *screen_pixel ^= pixel as u32;
            }
        }
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: String::from("Chip8"),
        window_width: 640,
        window_height: 320,
        high_dpi: false,
        fullscreen: false,
        sample_count: 0,
        window_resizable: false,
        icon: None,
        platform: Default::default(),
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut chip8 = Chip8::default();

    chip8.init_keymap();
    chip8.load_rom("trip8.ch8");
    
    loop {
        let op = chip8.fetch();
        
        println!("Instr: {:X}", (op & 0xF000) >> 12);
        println!("& 0xFFF: {:X}", op & 0x0FFF);

        let vx = ((op & 0x0F00) >> 8) as u8;
        let vy = ((op & 0x00F0) >> 4) as u8;

        match (op & 0xF000) >> 12 {
            0x0 => match op & 0x0FFF {
                0x0E0 => chip8.cls(),
                0x0EE => chip8.ret(),
                _ => chip8.call(op & 0x0FFF),
            },
            0x1 => chip8.jmp(op & 0x0FFF),
            0x2 => chip8.call_sub(op & 0x0FFF),
            0x3 => chip8.eq(vx, (op & 0x00FF) as u8),
            0x4 => chip8.ne(vx, (op & 0x00FF) as u8),
            0x5 => chip8.reg_eq(vx, vy),
            0x6 => chip8.set_reg(vx, (op & 0x00FF) as u8),
            0x7 => chip8.add_reg(vx, (op & 0x00FF) as u8),
            0x8 => match op & 0x000F {
                0x0 => chip8.reg_cp(vx, vy),
                0x1 => chip8.reg_or(vx, vy),
                0x2 => chip8.reg_and(vx, vy),
                0x3 => chip8.vxor(vx, vy),
                0x4 => chip8.reg_add(vx, vy),
                0x5 => chip8.reg_sub(vx, vy),
                0x6 => chip8.reg_shr(vx, vy),
                0x7 => chip8.reg_sub_inv(vx, vy),
                0xE => chip8.reg_shl(vx, vy),
                _ => panic!("Unknown opcode {:X}", op),
            },
            0x9 => chip8.jne(vx, vy),
            0xA => chip8.set_index(op & 0x0FFF),
            0xB => chip8.jmp_v0(op & 0x0FFF),
            0xC => chip8.vx_rand(vx, (op & 0x00FF) as u8),
            0xD => chip8.draw(vx, vy, (op & 0x000F) as u8),
            0xE => match op & 0x00FF {
                0x9E => chip8.keydown(vx),
                0xA1 => chip8.key_not_down(vx),
                _ => panic!("Invalid opcode {:X}", op),
            },
            0xF => match op & 0x00FF {
                0x07 => chip8.get_delay(vx),
                0x15 => chip8.set_delay(vx),
                0x18 => chip8.set_sound(vx),
                0x1E => chip8.add_to_i(vx),
                0x29 => chip8.set_sprite(vx),
                0x55 => chip8.fill_mem(vx),
                0x65 => chip8.fill_reg(vx),
                _ => panic!("Invalid opcode {:X}", op),
            }
            _ => panic!("Unknown opcode: {:#X}", op),
        }


        for row in 0..64 {
            for col in 0..32 {
                if(chip8.display[row][col] != 0) {
                    shapes::draw_rectangle(window::screen_width() / 64.0 * row as f32, window::screen_height() / 32. * col as f32 , window::screen_width() / 64., window::screen_height() / 32., WHITE);
                }
            }
        }

        window::next_frame().await;
    }
}
