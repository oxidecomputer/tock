//! Core low-level operations.

use crate::csr::{mstatus::mstatus, CSR};
use core::ops::FnOnce;

#[inline(always)]
/// NOP instruction
pub fn nop() {
    unsafe {
        asm!("nop" :::: "volatile");
    }
}

#[inline(always)]
/// WFI instruction
pub unsafe fn wfi() {
    asm!("wfi" :::: "volatile");
}

pub unsafe fn atomic<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    let cached_mstatus = CSR.mstatus.extract();
    if cached_mstatus.is_set(mstatus::mie) {
        CSR.mstatus
            .modify_no_read(cached_mstatus, mstatus::mie::CLEAR);
    }
    let res = f();
    if cached_mstatus.is_set(mstatus::mie) {
        CSR.mstatus
            .modify_no_read(cached_mstatus, mstatus::mie::SET);
    }
    res
}

#[cfg(target_os = "none")]
#[lang = "eh_personality"]
pub extern "C" fn eh_personality() {}
