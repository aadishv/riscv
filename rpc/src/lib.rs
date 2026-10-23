#![no_std]

#[repr(C)]
pub enum CpuToFlpr {
    Ping,
}

#[repr(C)]
pub enum FlprToCpu {
    Pong,
}

const _: () = assert!(size_of::<CpuToFlpr>() <= 2048);
const _: () = assert!(size_of::<FlprToCpu>() <= 2048);