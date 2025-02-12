use rand::Rng;
use std::fs;
use std::path::Path;
use crate::{WIDTH};

// configure test cases
#[cfg(test)]
#[path = "test_opcodes.rs"]
mod test_opcodes;

// implement data types

pub struct Chip8 {
    pub opcode:      u16,                   // unsigned short opcode;
    pub memory:      [u8; 4096],            // unsigned char memory[4096];
    pub v:           [u8; 16],              // unsigned char V[16];
    pub i:           u16,                   // unsigned short I;
    pub pc:          u16,                   // unsigned short pc;
    pub gfx:         [[u8; 32]; 64],        // unsigned char gfx[64 * 32];
    pub delay_timer: u8,                    // unsigned char delay_timer;
    pub sound_timer: u8,                    // unsigned char sound_timer;
    pub stack:       [u16; 16],             // unsigned short stack[16];
    pub sp:          usize,                 // unsigned short sp;
    pub key:         [u8; 16],              // unsigned char key[16];
    pub draw_flag:   bool,
}

impl Chip8 {
    
    // create a new Chip8 instance
    pub fn initialize() -> Self {
        Self {
            opcode:      0,                // reset current opcode
            memory:      [0; 4096],        // clear memory
            v:           [0; 16],          // clear registers V0-VF
            i:           0,                // reset index register
            pc:          0x200,            // program counter starts at 0x200
            gfx:         [[0x00; 32]; 64], // clear display
            delay_timer: 0,                // reset delay timer
            sound_timer: 0,                // reset sound timer
            stack:       [0; 16],          // clear stack
            sp:          0,                // reset stack pointer
            key:         [0; 16],          // assign keys
            draw_flag:   false,            // not ready to draw
        }
    }
     
    pub fn load_fontset(&mut self) {
        let fontset: [u8; 80] = [
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

        for i in 0..80 {
            self.memory[i] = fontset[i];
        }
    }

    pub fn load_program(&mut self, path_arg: &str) -> Result<(), Box<dyn std::error::Error + 'static>> {
        // load program into memory at memory[512] (0x200)
        let path = Path::new(path_arg);
        let data: Vec<u8> = fs::read(&path)?;
        
        for i in 0..data.len() {
            self.memory[i + 512] = data[i];
            // println!("memory[{}]: {}", (i + 512), data[i]);
        }

        Ok(())
    }

    pub fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let x = i % WIDTH as usize;
            let y = i / WIDTH as usize;

            let rgba = if self.gfx[x][y] != 0 {
                [0xff, 0xff, 0xff, 0xff]
            } else {
                [0x00, 0x00, 0x00, 0xff]
            };

            pixel.copy_from_slice(&rgba);
        }
    }

    fn log(&self, call: &str) {
        println!("{:#0x}      {:04x}      {}", self.pc, self.opcode, call);
    }

    fn get_opcode(&mut self) -> u16 {
        // fetch opcode
        (self.memory[self.pc as usize] as u16) << 8 | (self.memory[self.pc as usize + 1] as u16)
    }

    pub fn emulate_cycle(&mut self) {

        self.opcode = self.get_opcode();
        
        let nibbles = (
            (self.opcode & 0xF000) >> 12 as u8,
            (self.opcode & 0x0F00) >> 8 as u8,
            (self.opcode & 0x00F0) >> 4 as u8,
            (self.opcode & 0x000F) as u8,
        );

        let x        = ((self.opcode & 0x0F00) >> 8) as usize;
        let y        = ((self.opcode & 0x00F0) >> 4) as usize;
        let n        = (self.opcode & 0x000F) as usize;
        let kk       = (self.opcode & 0x00FF) as u8;
        let nnn      = self.opcode & 0x0FFF;

        match nibbles {
            (0x00, 0x00, 0x0e, 0x00) => self.op_00e0(),
            (0x00, 0x00, 0x0e, 0x0e) => self.op_00ee(),
            (0x01, _, _, _)          => self.op_1nnn(nnn),
            (0x02, _, _, _)          => self.op_2nnn(nnn),
            (0x03, _, _, _)          => self.op_3xkk(x, kk),
            (0x04, _, _, _)          => self.op_4xkk(x, kk),
            (0x05, _, _, 0x00)       => self.op_5xy0(x, y),
            (0x06, _, _, _)          => self.op_6xkk(x, kk),
            (0x07, _, _, _)          => self.op_7xkk(x, kk),
            (0x08, _, _, 0x00)       => self.op_8xy0(x, y),
            (0x08, _, _, 0x01)       => self.op_8xy1(x, y),
            (0x08, _, _, 0x02)       => self.op_8xy2(x, y),
            (0x08, _, _, 0x03)       => self.op_8xy3(x, y),
            (0x08, _, _, 0x04)       => self.op_8xy4(x, y),
            (0x08, _, _, 0x05)       => self.op_8xy5(x, y),
            (0x08, _, _, 0x06)       => self.op_8x06(x),
            (0x08, _, _, 0x07)       => self.op_8xy7(x, y),
            (0x08, _, _, 0x0e)       => self.op_8x0e(x),
            (0x09, _, _, 0x00)       => self.op_9xy0(x, y),
            (0x0a, _, _, _)          => self.op_annn(nnn),
            (0x0b, _, _, _)          => self.op_bnnn(nnn),
            (0x0c, _, _, _)          => self.op_cxkk(x, kk),
            (0x0d, _, _, _)          => self.op_dxyn(x, y, n),
            (0x0e, _, 0x09, 0x0e)    => self.op_ex9e(x),
            (0x0e, _, 0x0a, 0x01)    => self.op_exa1(x),
            (0x0f, _, 0x00, 0x07)    => self.op_fx07(x),
            (0x0f, _, 0x00, 0x0a)    => self.op_fx0a(x),
            (0x0f, _, 0x01, 0x05)    => self.op_fx15(x),
            (0x0f, _, 0x01, 0x08)    => self.op_fx18(x),
            (0x0f, _, 0x01, 0x0e)    => self.op_fx1e(x),
            (0x0f, _, 0x02, 0x09)    => self.op_fx29(x),
            (0x0f, _, 0x03, 0x03)    => self.op_fx33(x),
            (0x0f, _, 0x05, 0x05)    => self.op_fx55(x),
            (0x0f, _, 0x06, 0x05)    => self.op_fx65(x),
            _ => println!("Unknown opcode: {:#0X}", self.opcode),
        }

    
    }

    pub fn op_00e0(&mut self) {
        // CLS
        // Clear the display.
        self.gfx = [[0x00; 32]; 64];
        self.draw_flag = true;
        self.pc += 2;
        self.log("CLS");
    }
    pub fn op_00ee(&mut self) {
        // RET
        // Return from a subroutine
        self.sp -= 1;
        self.pc = self.stack[self.sp];
        self.log("RET");
    }
    pub fn op_1nnn(&mut self, nnn: u16) {
        // JP addr
        // Jump to location nnn
        self.pc = nnn;
        self.log("JP addr");
    }
    pub fn op_2nnn(&mut self, nnn: u16) {
        // CALL addr
        // Call subroutine at nnn
        self.stack[self.sp] = self.pc + 2;
        self.sp += 1;
        self.pc = nnn;
        self.log("CALL addr");
    }
    pub fn op_3xkk(&mut self, x: usize, kk: u8) {
        // SE Vx, byte
        // Skip next instruction if Vx == kk.
        if self.v[x] == kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
        self.log("SE Vx, byte");
    }
    pub fn op_4xkk(&mut self, x: usize, kk: u8) {
        // SNE Vx, byte
        // Skip next instruction if Vx != kk.
        if self.v[x] != kk {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
        self.log("SNE Vx, byte");
    }
    pub fn op_5xy0(&mut self, x: usize, y: usize) {
        // SE Vx, Vy
        // Skip next instruction if Vx = Vy
        if self.v[x] == self.v[y] {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
        self.log("SE Vx, Vy");
    }
    pub fn op_6xkk(&mut self, x: usize, kk: u8) {
        // LD Vx, byte
        // Set Vx = kk
        self.v[x] = kk;
        self.pc += 2;
        self.log("LD Vx, byte");
    }
    pub fn op_7xkk(&mut self, x: usize, kk: u8) {
        // ADD Vx, byte
        // Set Vx = Vx + kk
        self.v[x] = (self.v[x] as u16 + kk as u16) as u8;
        self.pc += 2;
        self.log("ADD Vx, byte");
    }
    pub fn op_8xy0(&mut self, x: usize, y: usize) {
        // LD Vx, Vy
        // Set Vx = Vy
        self.v[x] = self.v[y];
        self.pc += 2;
        self.log("LD Vx, Vy");
    }
    pub fn op_8xy1(&mut self, x: usize, y: usize) {
        // OR Vx, Vy
        // Set Vx = Vx OR Vy
        self.v[x] = self.v[x] | self.v[y];
        self.pc += 2;
        self.log("OR Vx, Vy");
    }
    pub fn op_8xy2(&mut self, x: usize, y: usize) {
        // AND Vx, Vy
        // Set Vx = Vx AND Vy
        self.v[x] = self.v[x] & self.v[y];
        self.pc += 2;
        self.log("AND Vx, Vy");
    }
    pub fn op_8xy3(&mut self, x: usize, y: usize) {
        // XOR Vx, Vy
        // Set Vx = Vx XOR Vy
        self.v[x] = self.v[x] ^ self.v[y];
        self.pc += 2;
        self.log("XOR Vx, Vy");
    }
    pub fn op_8xy4(&mut self, x: usize, y: usize) {
        // ADD Vx, Vy
        // Set Vx = Vx + Vy, set VF = carry
        
        // Set Vx = Vx + Vy
        let result = self.v[x] as u16 + self.v[y] as u16;
        self.v[x] = result as u8;
        
        // Compare and set VF
        if result > 0xFF {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }

        self.pc += 2;
        self.log("ADD Vx, Vy");
    }
    pub fn op_8xy5(&mut self, x: usize, y: usize) {
        // SUB Vx, Vy
        // Set Vx = Vx - Vy, set VF = NOT borrow
        if self.v[x] > self.v[y] {
            self.v[0x0F] = 1;
        } else {
            self.v[0x0F] = 0;
        }
        self.v[x] = self.v[x].wrapping_sub(self.v[y]); 
        self.pc += 2;
        self.log("SUB Vx, Vy");
    }
    pub fn op_8x06(&mut self, x: usize) {
        // SHR Vx {, Vy}
        // Set Vx = Vx SHR 1
        self.v[0xF] = self.v[x] & 1;
        self.v[x] >>= 1;
        self.pc += 2;
        self.log("SHR Vx {, Vy}");
    }
    pub fn op_8xy7(&mut self, x: usize, y: usize) {
        // SUBN Vx, Vy
        // Set Vx = Vy - Vx, set VF = NOT borrow
        if self.v[y] > self.v[x] {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }
        self.v[x] = self.v[y] - self.v[x];
        self.pc += 2;
        self.log("SUBN Vx, Vy");
    }
    pub fn op_8x0e(&mut self, x: usize) {
        // SHL Vx {, Vy}
        // Set Vx = Vx SHL 1
        self.v[0xF] = (self.v[x] & 0x80) >> 7;
        self.v[x] <<= 1;
        self.pc += 2;
        self.log("SHL Vx {, Vy}");
    }
    pub fn op_9xy0(&mut self, x: usize, y: usize) {
        // SNE Vx, Vy
        // Skip next instruction if Vx != Vy
        if self.v[x] != self.v[y] >> 4 {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
        self.log("SNE Vx, Vy");
    }
    pub fn op_annn(&mut self, nnn: u16) {
        // LD I, addr
        // Set I = nnn
        self.i = nnn;
        self.pc += 2;
        self.log("LD I, addr")
    }
    pub fn op_bnnn(&mut self, nnn: u16) {
        // JP V0, addr
        // Jump to location nnn + V0
        self.pc = nnn + (self.v[0] as u16);
        self.log("JP V0, addr");
    }
    pub fn op_cxkk(&mut self, x: usize, kk: u8) {
        // RND Vx, byte
        // Set Vx = random byte AND kk
        self.v[x] = rand::thread_rng().gen::<u8>() & kk;
        self.pc += 2;
        self.log("RND Vx, byte");
    }
    pub fn op_dxyn(&mut self, x: usize, y: usize, n: usize) {
        // Display n-byte sprite starting at memory location I at {Vx, Vy}, set VF = collision
        //
        // The interpreter reads n bytes from memory, starting at the address storied in I. These bytes
        // are then displayed as sprites on screen at coordinates (Vx, Vy). Sprites are XORed onto the
        // existing screen. If this causes any pixels to be erased, VF is set to 1, otherwise it is set
        // to 0. If the sprite is positioned so part of it is outside the coordinates of the display,
        // it wraps around to the opposite side of the screen.
    
        self.v[0xF] = 0;

        for byte in 0..n {
            let dxyn_y = (self.v[y] as usize + byte as usize) % 32;
            for bit in 0..8 {
                let dxyn_x = (self.v[x] as usize + bit as usize) % 64;
                let color = (self.memory[(self.i as usize + byte) as usize] >> (7 - bit)) & 1;
                self.v[0xf] |= color & self.gfx[dxyn_x][dxyn_y];
                self.gfx[dxyn_x][dxyn_y] ^= color;
            }
        }

        self.draw_flag = true;
        self.pc += 2;
        self.log("DRW Vx, Vy, nibble");
    }
    pub fn op_ex9e(&mut self, x: usize) {
        // SKP Vx
        // Skip next instruction if key with the value of Vx is pressed
        if self.key[self.v[x] as usize] == 1 {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
        self.log("SKP Vx");
    }
    pub fn op_exa1(&mut self, x: usize) {
        // SKNP Vx
        // Skip next instruction if key with the value of Vx is not pressed
        if self.key[self.v[x] as usize] != 1 {
            self.pc += 4;
        } else {
            self.pc += 2;
        }
        self.log("SKNP Vx");
    }
    pub fn op_fx07(&mut self, x: usize) {
        // LD Vx, DT
        // Set Vx = delay timer value
        self.v[x] = self.delay_timer;
        self.pc += 2;
        self.log("LD Vx, DT");
    }
    pub fn op_fx0a(&mut self, x: usize) {
        // LD Vx, K
        // Wait for a key press, store the value of the key in Vx
        if self.key != [0; 16] {
            for i in 0..15 {
                if self.key[i] != 0 {
                    self.v[x] = i as u8;
                }
            }

            self.pc += 2;
            self.log("LD Vx, K");
        }
    }
    pub fn op_fx15(&mut self, x: usize) {
        // LD DT, Vx
        // Set delay timer = Vx
        self.delay_timer = self.v[x];
        self.pc += 2;
        self.log("LD DT, Vx");
    }
    pub fn op_fx18(&mut self, x: usize) {
        // LD ST, Vx
        // Set sound timer = Vx
        self.sound_timer = self.v[x];
        self.pc += 2;
        self.log("LD ST, Vx");
    }
    pub fn op_fx1e(&mut self, x: usize) {
        // ADD I, Vx
        // Set I = I + Vx
        self.i += self.v[x] as u16;
        self.pc += 2;
        self.log("ADD I, Vx");
    }
    pub fn op_fx29(&mut self, x: usize) {
        // LD F, Vx
        // Set I = location of sprite for digit Vx
        self.i = (self.v[x] as u16) * 5;
        self.pc += 2;
        self.log("LD F, Vx");
    }
    pub fn op_fx33(&mut self, x: usize) {
        // LD B, Vx
        // Store BCD representation of Vx in memory locations I, I+1, and I+2
        self.memory[self.i as usize]       =   self.v[x] / 100;
        self.memory[(self.i + 1) as usize] =  (self.v[x] / 10) % 10;
        self.memory[(self.i + 2) as usize] =  (self.v[x] % 100) % 10;
        self.pc += 2;
        self.log("LD B, Vx");
    }
    pub fn op_fx55(&mut self, x: usize) {
        // LD [I], Vx
        // Store registers V0 through Vx in memory starting at location I
        for i in 0..(x as u16) {
            self.memory[(self.i + i) as usize] = self.v[i as usize];
        }
        self.pc += 2;
        self.log("LD [I], Vx");
    }
    pub fn op_fx65(&mut self, x: usize) {
        // LD Vx, [I]
        // Read registers V0 through Vx from memory starting at location I
        for i in 0..(x as u16) {
            self.v[i as usize] = self.memory[(self.i + i) as usize];
        }
        self.pc += 2;
        self.log("LD Vx, [I]");
    }

}
