
use crate::Chip8;

#[test]
fn test_initialize() {
    let mut myChip8 = Chip8::initialize();
    assert_eq!(myChip8.pc, 0x200);
    assert_eq!(myChip8.sp, 0);
    assert_eq!(myChip8.stack, [0; 16]);

    myChip8.load_fontset();
    assert_eq!(myChip8.memory[0], 0xF0);
    assert_eq!(myChip8.memory[1], 0x90);
}

#[test]
fn test_fontset() {
    let mut myChip8 = Chip8::initialize();
    myChip8.load_fontset();

    let fontset_test: [u8; 80] = [
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
        assert_eq!(myChip8.memory[i], fontset_test[i]);
    }
}


