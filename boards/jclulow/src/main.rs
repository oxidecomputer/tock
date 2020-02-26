#![no_std]
#![no_main]

/*
 * Tinkering with a basic ARM Cortex-M3 "board" in tandem with QEMU
 * experimentation.
 */

pub mod io;

const NUM_PROCS: usize = 1;

const FAULT_RESPONSE: kernel::procs::FaultResponse =
    kernel::procs::FaultResponse::Panic;


