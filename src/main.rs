use pixels::{Error, Pixels, SurfaceTexture};
use std::{fs, time::Duration, thread};
use std::path::Path;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use log::error;
use error_iter::ErrorIter;
use rand::Rng;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;
const TICK_SPEED: u64 = 150;

// implement data types

struct Chip8 {
    opcode:      u16,                   // unsigned short opcode;
    memory:      [u8; 4096],            // unsigned char memory[4096];
    v:           [u8; 16],              // unsigned char V[16];
    i:           u16,                   // unsigned short I;
    pc:          u16,                   // unsigned short pc;
    gfx:         [[u8; 32]; 64],        // unsigned char gfx[64 * 32];
    delay_timer: u8,                    // unsigned char delay_timer;
    sound_timer: u8,                    // unsigned char sound_timer;
    stack:       [u16; 16],             // unsigned short stack[16];
    sp:          usize,                 // unsigned short sp;
    key:         [u8; 16],              // unsigned char key[16];
    draw_flag:   bool,
}

impl Chip8 {
    
    // create a new Chip8 instance
    fn initialize() -> Self {
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
     
    fn load_fontset(&mut self) {
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

    fn load_program(&mut self, path_arg: &str) -> Result<(), Box<dyn std::error::Error + 'static>> {
        // load program into memory at memory[512] (0x200)
        let path = Path::new(path_arg);
        let data: Vec<u8> = fs::read(&path)?;
        
        for i in 0..data.len() {
            self.memory[i + 512] = data[i];
            // println!("memory[{}]: {}", (i + 512), data[i]);
        }

        Ok(())
    }

    fn draw(&self, frame: &mut [u8]) {
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

    fn emulate_cycle(&mut self) {
        // fetch opcode
        self.opcode = (self.memory[self.pc as usize] as u16) << 8 | (self.memory[self.pc as usize + 1] as u16);
        
        let x        = ((self.opcode & 0x0F00) >> 8) as usize;
        let y        = ((self.opcode & 0x00F0) >> 4) as usize;
        let n        = (self.opcode & 0x000F) as usize;
        let kk       = (self.opcode & 0x00FF) as u8;
        let nnn      = self.opcode & 0x0FFF;

        // decode and execute opcode
        match self.opcode & 0xF000 {
            0x0000 => {
                match self.opcode & 0x000F {
                    0x0000 => { // 00E0: Clears the screen
                        self.gfx = [[0x00; 32]; 64];
                        self.draw_flag = true;
                        self.pc += 2;
                        self.log("CLS");
                    },
                    0x000E => { // 00EE: Returns from subroutine
                        self.sp -= 1;
                        self.pc = self.stack[self.sp];
                        self.log("RET");
                    },
                    _ => println!("Unknown opcode [0x0000]: {:#0X}", self.opcode),
                }
            },
            0x1000 => { // 1nnn: Jumps to location nnn
                self.pc = nnn;
                self.log("JP addr");
            },
            0x2000 => { // 2nnn: Calls subroutine at nnn
                self.stack[self.sp] = self.pc + 2;
                self.sp += 1;
                self.pc = nnn;
                self.log("CALL addr");
            },
            0x3000 => { // 3xkk: Skip next instruction if Vx = kk
                if self.v[x] == kk {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
                self.log("SE Vx, byte");
            },
            0x4000 => { // 4xkk: Skip next instruction if Vx != kk
                if self.v[x] != kk {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
                self.log("SNE Vx, byte");
            },
            0x5000 => { // 5xy0: Skip next instruction if Vx = Vy
                if self.v[x] == self.v[y] {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
                self.log("SE Vx, Vy");
            },
            0x6000 => { // 6xkk: Set Vx = kk
                self.v[x] = kk;
                self.pc += 2;
                self.log("LD Vx, byte");
            },
            0x7000 => { // 7xkk: Set Vx = Vx + kk
                self.v[x] = (self.v[x] as u16 + kk as u16) as u8;
                self.pc += 2;
                self.log("ADD Vx, byte");
            },
            0x8000 => {
                match self.opcode & 0x000F {
                    0x0000 => { // 8xy0: Set Vx = Vy
                        self.v[x] = self.v[y];
                        self.pc += 2;
                        self.log("LD Vx, Vy");
                    },
                    0x0001 => { // 8xy1: Set Vx = Vx OR Vy
                        self.v[x] = self.v[x] | self.v[y];
                        self.pc += 2;
                        self.log("OR Vx, Vy");
                    },
                    0x0002 => { // 8xy2: Set Vx = Vx AND Vy
                        self.v[x] = self.v[x] & self.v[y];
                        self.pc += 2;
                        self.log("AND Vx, Vy");
                    },
                    0x0003 => { // 8xy3: Set Vx = Vx XOR Vy
                        self.v[x] = self.v[x] ^ self.v[y];
                        self.pc += 2;
                        self.log("XOR Vx, Vy");
                    },
                    0x0004 => { // 8xy4: Set Vx = Vx + Vy, set VF = carry
                        if self.v[x] > 255 {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.v[x] = (self.v[x] as u16 + self.v[y] as u16) as u8;
                        self.pc += 2;
                        self.log("ADD Vx, Vy");
                    },
                    0x0005 => { // 8xy5: Set Vx = Vx - Vy, set VF = NOT borrow
                        if self.v[x] > self.v[y] {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.v[x] = self.v[x].wrapping_sub(self.v[y]); 
                        self.pc += 2;
                        self.log("SUB Vx, Vy");
                    },
                    0x0006 => { // 8xy6: Set Vx = Vx SHR 1
                        self.v[0xF] = self.v[x] & 1;
                        self.v[x] >>= 1;
                        self.pc += 2;
                        self.log("SHR Vx {, Vy}");
                    },
                    0x0007 => { // 8xy7: Set Vx = Vy - Vx, set VF = NOT borrow
                        if self.v[y] > self.v[x] {
                            self.v[0xF] = 1;
                        } else {
                            self.v[0xF] = 0;
                        }
                        self.v[x] = self.v[y] - self.v[x];
                        self.pc += 2;
                        self.log("SUBN Vx, Vy");
                    },
                    0x000E => { // 8xyE: set Vx = Vx SHL 1
                        self.v[0xF] = (self.v[x] & 0x80) >> 7;
                        self.v[x] <<= 1;
                        self.pc += 2;
                        self.log("SHL Vx {, Vy}");
                    },
                    _ => println!("Unknown opcode [0x8000]: {:#0X}", self.opcode),
                }
            },
            0x9000 => { // 9xy0: Skip next instruction if Vx != Vy
                if self.v[x] != self.v[y] >> 4 {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
                self.log("SNE Vx, Vy");
            },
            0xA000 => { // Annn: Set I = nnn
                self.i = nnn;
                self.pc += 2;
                self.log("LD I, addr");
            },
            0xB000 => { // Bnnn: Jump to location nnn + V0
                self.pc = nnn + (self.v[0] as u16);
                self.log("JP V0, addr");
            },
            0xC000 => { // Cxkk: Set Vx = random byte AND kk
                self.v[x] = rand::thread_rng().gen::<u8>() & kk;
                self.pc += 2;
                self.log("RND Vx, byte");
            },
            0xD000 => { // Dxyn: Display n-byte sprite starting at memory location I at (Vx, Vy), set VF = collision
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

            },
            0xE000 => {
                match self.opcode & 0x000F {
                    0x000E => { // Ex9E: Skip next instruction if key with the value of Vx is pressed
                        if self.key[self.v[x] as usize] == 1 {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                        self.log("SKP Vx");
                    },
                    0x0001 => { // ExA1: Skip next instruction if key with the value of Vx is not pressed
                        if self.key[self.v[x] as usize] != 1 {
                            self.pc += 4;
                        } else {
                            self.pc += 2;
                        }
                        self.log("SKNP Vx");
                    },
                    _ => println!("Unknown opcode [0xE000]: {:#0X}", self.opcode),
                }
            },
            0xF000 => {
                match self.opcode & 0x00FF {
                    0x0007 => { // Fx07: Set Vx = delay timer value
                        self.v[x] = self.delay_timer;
                        self.pc += 2;
                        self.log("LD Vx, DT");
                    },
                    0x000A => { // Fx0A: Wait for a key press, store the value of the key in Vx
                        if self.key != [0; 16] {
                            for i in 0..15 {
                                if self.key[i] != 0 {
                                    self.v[x] = i as u8;
                                }
                            }

                            self.pc += 2;
                            self.log("LD Vx, K");
                        }
                    },
                    0x0015 => { // Fx15: Set delay timer = Vx
                        self.delay_timer = self.v[x];
                        self.pc += 2;
                        self.log("LD DT, Vx");
                    },
                    0x0018 => { // Set sound timer = Vx
                        self.sound_timer = self.v[x];
                        self.pc += 2;
                        self.log("LD ST, Vx");
                    },
                    0x001E => { // Set I = I + Vx
                        self.i += self.v[x] as u16;
                        self.pc += 2;
                        self.log("ADD I, Vx");
                    },
                    0x0029 => { // Set I = location of sprite for digit Vx
                        self.i = (self.v[x] as u16) * 5;
                        self.pc += 2;
                        self.log("LD F, Vx");
                    },
                    0x0033 => { // Store BCD representation of Vx in memory location I, I+1, and I+2
                        self.memory[self.i as usize]       =   self.v[x] / 100;
                        self.memory[(self.i + 1) as usize] =  (self.v[x] / 10) % 10;
                        self.memory[(self.i + 2) as usize] =  (self.v[x] % 100) % 10;
                        self.pc += 2;
                        self.log("LD B, Vx");
                    },
                    0x0055 => { // Store registers V0 through Vx in memory starting at location I
                        for i in 0..(x as u16) {
                            self.memory[(self.i + i) as usize] = self.v[i as usize];
                        }
                        self.pc += 2;
                        self.log("LD [I], Vx");
                    },
                    0x0065 => { // Read registers V0 through Vx from memory starting at location I
                        for i in 0..(x as u16) {
                            self.v[i as usize] = self.memory[(self.i + i) as usize];
                        }
                        self.pc += 2;
                        self.log("LD Vx, [I]");
                    },
                    _ => println!("Unknown opcode [0xF000]: {:#0X}", self.opcode),
                }
            },
            _ => {
                self.pc += 2;
                println!("Unknown opcode: {:#0X}", self.opcode)
            },
        }
        
    }

}

fn main() -> Result<(), Error> {

    // set up render system
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(1024 as f64, 512 as f64);
        WindowBuilder::new()
            .with_title("chip8")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };


    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH, HEIGHT, surface_texture)?
    };

    // Initialize the Chip8 system and load the game into memory
    let mut myChip8 = Chip8::initialize();
    myChip8.load_fontset();

    let path = std::env::args().nth(1).expect("No path entered");
    let _ = myChip8.load_program(&path);

    let mut last_frame = std::time::Instant::now();
    let mut last_timer = std::time::Instant::now();

    // emulation loop
    let res = event_loop.run(|event, elwt| {

        // emulate one cycle
        myChip8.emulate_cycle();

        // lazy timing implementation
        if last_frame.elapsed() < Duration::from_secs(1 / TICK_SPEED) {
            thread::sleep(Duration::from_secs(1 / TICK_SPEED) - last_frame.elapsed());
        }
        println!("DT: {:?}", last_frame.elapsed()); 
        last_frame = std::time::Instant::now();

        // update timers
        if myChip8.delay_timer > 0 {
            if last_timer.elapsed() >= Duration::from_secs(1 / 60) {
                myChip8.delay_timer = myChip8.delay_timer - 1;
            }
        }
        
        if myChip8.sound_timer > 0 {
            if last_timer.elapsed() >= Duration::from_secs(1 / 60) {
                println!("BEEP");
                myChip8.sound_timer = myChip8.sound_timer - 1;
            }
        }

        // if the draw flag is set, draw the current frame
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            if myChip8.draw_flag {
                myChip8.draw(pixels.frame_mut());
                myChip8.draw_flag = false;
                if let Err(err) = pixels.render() {
                    log_error("pixels.render", err);
                    elwt.exit();
                    return;
        }}}

        // handle input events
        if input.update(&event) {
            // close events
            if input.key_pressed(KeyCode::Escape) || input.close_requested() {
                elwt.exit();
                return;
            }

            let keybinds = [
                KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3, KeyCode::Digit4,
                KeyCode::KeyQ,   KeyCode::KeyW,   KeyCode::KeyE,   KeyCode::KeyR,
                KeyCode::KeyA,   KeyCode::KeyS,   KeyCode::KeyD,   KeyCode::KeyF,
                KeyCode::KeyZ,   KeyCode::KeyX,   KeyCode::KeyC,   KeyCode::KeyV
            ];

            for i in 0..keybinds.len() {
                if input.key_pressed(keybinds[i]) {myChip8.key[i] = 1;}
                else if input.key_released(keybinds[i]) {myChip8.key[i] = 0;}
            }
            
            // resize the window
            if let Some(size) = input.window_resized() {
                myChip8.draw_flag = true;
                if let Err(err) = pixels.resize_surface(size.width, size.height) {
                    log_error("pixels.resize_surface", err);
                    elwt.exit();
                    return;
                }
            }

            window.request_redraw();
        }
    });
    res.map_err(|e| Error::UserDefined(Box::new(e)))
}


fn log_error<E: std::error::Error + 'static>(method_name: &str, err:E) {
    error!("{method_name}() faild: {err}");
    for source in err.sources().skip(1) {
        error!("  caused by: {source}");
    }
}

