use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use std::fs;
use std::path::Path;
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;

// implement data types

struct Chip8 {
    opcode:      u16,                // unsigned short opcode;
    memory:      [u8; 4096],         // unsigned char memory[4096];
    v:           [u8; 16],           // unsigned char V[16];
    i:           u16,                // unsigned short I;
    pc:          u16,                // unsigned short pc;
    gfx:         [[u8; 32]; 64],     // unsigned char gfx[64 * 32];
    delay_timer: u8,                 // unsigned char delay_timer;
    sound_timer: u8,                 // unsigned char sound_timer;
    stack:       [u16; 16],          // unsigned short stack[16];
    sp:          u16,                // unsigned short sp;
    key:         [u8; 16],           // unsigned char key[16];
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
            key:         [0; 16],          // assumes no key is pressed
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

    fn emulate_cycle(&mut self) {
    }

}

fn main() -> Result<(), Error> {

    // set up render system
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
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

    // initialize the chip8 system and load the game into memory
    let mut myChip8 = Chip8::initialize();
    myChip8.load_fontset();
    let _ = myChip8.load_program("/home/amelia/Downloads/ibm-logo.ch8");

    // emulation loop
    let res = event_loop.run(|event, elwt| {

        // emulate one cycle
        myChip8.emulate_cycle();
        
        // if the draw flag is set, draw the current frame
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            if myChip8.draw_flag {
                myChip8.draw(pixels.frame_mut());
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

            // resize the window
            if let Some(size) = input.window_resized() {
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

