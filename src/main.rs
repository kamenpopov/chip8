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
        // println!("Data at jmp ({:X})", self.memory[self.pc as usize]);
    }
    
    fn call(&mut self, addr: u16) {
        println!("Called call()")
    }
    
    fn call_sub(&mut self, addr: u16) {
        println!("call_sub()")
    }
    
    fn ret(&mut self) {
        println!("ret()")
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
    
    fn draw(&mut self, x: u8, y: u8, height: u8) {
        println!("draw({}, {}, {})", x, y, height);


        let x_pos: usize = self.registers[x as usize] as usize;
        let y_pos: usize = self.registers[y as usize] as usize;

        println!("Pos: {}, {}", x_pos, y_pos);

        self.registers[0xF] = 0;
        
        for row in 0..height as usize {
            let row_bits = self.memory[self.index_register as usize + row];
            for col in 0..8 {
                let pixel = row_bits >> (7 - col) & 0x1;
                let screen_pixel = &mut self.display[x_pos + col][y_pos + row];

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
    chip8.load_rom("ibm_logo.ch8");
    
    loop {
        let op = chip8.fetch();
        
        println!("Instr: {:X}", (op & 0xF000) >> 12);
        println!("& 0xFFF: {:X}", op & 0x0FFF);

        match (op & 0xF000) >> 12 {
            0x0 => match op & 0x0FFF {
                0x0E0 => chip8.cls(),
                0x0EE => chip8.ret(),
                _ => chip8.call(op & 0x0FFF),
            },
            0x1 => chip8.jmp(op & 0x0FFF),
            0x2 => chip8.call_sub(op & 0x0FFF),
            0x3 => chip8.eq(((op & 0x0F00) >> 8) as u8, (op & 0x00FF) as u8),
            0x6 => chip8.set_reg(((op & 0x0F00) >> 8) as u8, (op & 0x00FF) as u8),
            0x7 => chip8.add_reg(((op & 0x0F00) >> 8) as u8, (op & 0x00FF) as u8),
            0xA => chip8.set_index(op & 0x0FFF),
            0xD => chip8.draw(((op & 0x0F00) >> 8) as u8, ((op & 0x00F0) >> 4) as u8, (op & 0x000F) as u8),
            _ => println!("Unknown opcode: {:#X}", op),
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
