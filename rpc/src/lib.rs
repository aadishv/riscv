#![no_std]
use defmt::Format;

#[repr(C)]
#[derive(Format, Clone, Copy)]
pub enum CpuToFlpr {
    Ping,
}

#[repr(C)]
#[derive(Format, Clone, Copy)]
pub enum FlprToCpu {
    Pong = 67,
}

const _: () = assert!(size_of::<CpuToFlpr>() <= 2048);
const _: () = assert!(size_of::<FlprToCpu>() <= 2048);