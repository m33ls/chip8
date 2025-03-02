use pixels::{Error, Pixels, SurfaceTexture};
use std::{time::Duration, thread};
use winit::dpi::LogicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::KeyCode;
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;
use log::error;
use error_iter::ErrorIter;
use crate::processor::Chip8;

const WIDTH: u32 = 64;
const HEIGHT: u32 = 32;
const TICK_SPEED: u64 = 150;

mod processor;

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
    let mut my_chip8 = Chip8::initialize();
    my_chip8.load_fontset();

    let path = std::env::args().nth(1).expect("No path entered");
    let _ = my_chip8.load_program(&path);

    let mut last_frame = std::time::Instant::now();
    let last_timer = std::time::Instant::now();

    // emulation loop
    let res = event_loop.run(|event, elwt| {

        // emulate one cycle
        my_chip8.emulate_cycle();

        // lazy timing implementation
        if last_frame.elapsed() < Duration::from_secs(1 / TICK_SPEED) {
            thread::sleep(Duration::from_secs(1 / TICK_SPEED) - last_frame.elapsed());
        }
        println!("DT: {:?}", last_frame.elapsed()); 
        last_frame = std::time::Instant::now();
        
        // update timers
        if my_chip8.delay_timer > 0 {
            if last_timer.elapsed() >= Duration::from_secs(1 / 60) {
                my_chip8.delay_timer = my_chip8.delay_timer - 1;
            }
        }
        
        if my_chip8.sound_timer > 0 {
            if last_timer.elapsed() >= Duration::from_secs(1 / 60) {
                println!("BEEP");
                my_chip8.sound_timer = my_chip8.sound_timer - 1;
            }
        }

        // if the draw flag is set, draw the current frame
        if let Event::WindowEvent {
            event: WindowEvent::RedrawRequested,
            ..
        } = event
        {
            if my_chip8.draw_flag {
                my_chip8.draw(pixels.frame_mut());
                my_chip8.draw_flag = false;
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

            // Keybinds
            // +-+-+-+-+    +-+-+-+-+
            // |1|2|3|C|    |1|2|3|4|
            // +-+-+-+-+    +-+-+-+-+
            // |4|5|6|D|    |Q|W|E|R|
            // +-+-+-+-+ => +-+-+-+-+
            // |7|8|9|E|    |A|S|D|F|
            // +-+-+-+-+    +-+-+-+-+
            // |A|0|B|F|    |Z|X|C|V|
            // +-+-+-+-+    +-+-+-+-+
            //
            // Array
            // x 1 2 3 
            // q w e a
            // s d z c 
            // 4 r f v

            let keybinds = [
                KeyCode::KeyX, KeyCode::Digit1, KeyCode::Digit2, KeyCode::Digit3,
                KeyCode::KeyQ,   KeyCode::KeyW,   KeyCode::KeyE,   KeyCode::KeyA,
                KeyCode::KeyS,   KeyCode::KeyD,   KeyCode::KeyZ,   KeyCode::KeyC,
                KeyCode::Digit4,   KeyCode::KeyR,   KeyCode::KeyF,   KeyCode::KeyV
            ];

            for i in 0..keybinds.len() {
                if input.key_pressed(keybinds[i]) {my_chip8.key[i] = 1;}
                else if input.key_released(keybinds[i]) {my_chip8.key[i] = 0;}
            }
            
            // resize the window
            if let Some(size) = input.window_resized() {
                my_chip8.draw_flag = true;
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

