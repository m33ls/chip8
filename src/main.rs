use error_iter::ErrorIter as _;
use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

// implement data types

struct Chip8 {
    opcode:      u16,                // unsigned short opcode;
    memory:      [u8; 4096],         // unsigned char memory[4096];
    v:           [u8; 16],           // unsigned char V[16];
    i:           u16,                // unsigned short I;
    pc:          u16,                // unsigned short pc;
    gfx:         [[u8; 64]; 32],     // unsigned char gfx[64 * 32];
    delay_timer: u8,                 // unsigned char delay_timer;
    sound_timer: u8,                 // unsigned char sound_timer;
    stack:       [u16; 16],          // unsigned short stack[16];
    sp:          u16,                // unsigned short sp;
    key:         [u8; 16],           // unsigned char key[16];
}

fn setup_graphics() -> Result<(), Error> {
    const WIDTH: u32 = 64;
    const HEIGHT: u32 = 32;

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

    let res = event_loop.run(|event, elwt| {
        // draw the current frame
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            draw(pixels.frame_mut());
            if let Err(err) = pixels.render() {
                log_error("pixels.render", err);
                elwt.exit();
                return;
            }
        }

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

fn draw(frame: &mut [u8]) {
    println!("drawing frame");
}

fn main() { 
    // Set up render system and register input callbacks
    setup_graphics();
}

