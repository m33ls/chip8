
// implement data types

struct chip8 {
    opcode:      u16,                // unsigned short opcode;
    memory:      [u8; 4096],         // unsigned char memory[4096];
    V:           [u8; 16],           // unsigned char V[16];
    I:           u16,                // unsigned short I;
    pc:          u16,                // unsigned short pc;
    gfx:         [[u8; 64]; 32],     // unsigned char gfx[64 * 32];
    delay_timer: u8,                 // unsigned char delay_timer;
    sound_timer: u8,                 // unsigned char sound_timer;
    stack:       [u16; 16],          // unsigned short stack[16];
    sp:          u16,                // unsigned short sp;
    key:         [u8; 16],           // unsigned char key[16];
}

fn setupGraphics() {

}

fn main() {
    // Set up render system and register input callbacks
    setupGraphics();
}

