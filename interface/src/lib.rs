#![no_std]

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BoardSize {
    // The number of tiles wide
    pub width: u32,
    pub height: u32,
}
