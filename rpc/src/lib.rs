#![no_std]

const SHARED_MEMORY_SIZE = 4096;

#[repr(C)]
enum CpuToFlpr {
    Ping,
}

#[repr(C)]
enum FlprToCpu {
    Pong,
}

