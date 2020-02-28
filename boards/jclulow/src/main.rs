#![no_std]
#![no_main]

/*
 * Tinkering with a basic ARM Cortex-M3 "board" in tandem with QEMU
 * experimentation.
 */

pub mod io;

const RAM_SIZE: usize = 1024 * 1024;
const APP_MEMORY_SIZE: usize = RAM_SIZE / 2;

const NUM_PROCS: usize = 1;

const FAULT_RESPONSE: kernel::procs::FaultResponse =
    kernel::procs::FaultResponse::Panic;

static mut PROCESSES:
    [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

static mut CHIP: Option<&'static jclulow1::chip::JClulow1> = None;

#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; APP_MEMORY_SIZE] = [0; APP_MEMORY_SIZE];

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

pub struct Platform {
}

impl kernel::Platform for Platform {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            _ => f(None),
        }
    }
}

fn msg(s: &str) {
    for b in s.as_bytes() {
        jclulow1::ioarea::poop(*b);
    }
    jclulow1::ioarea::poop('\n' as u8);
}

#[no_mangle]
pub unsafe fn reset_handler() {
    msg("computer starting!");

    loop {
    }
}
