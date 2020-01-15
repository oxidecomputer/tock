//! High-level setup and interrupt mapping for the chip.

use core::hint::unreachable_unchecked;

use kernel;
use kernel::debug;
use rv32i::syscall::SysCall;
use rv32i::csr::{CSR, mcause, mie::mie, mtvec::mtvec};

use crate::interrupts;
use crate::gpio;
use crate::plic;
use crate::timer;
use crate::uart;

#[cfg(not(feature = "verilator"))]
pub const CHIP_FREQ: u32 = 50_000_000;
#[cfg(feature = "verilator")]
pub const CHIP_FREQ: u32 = 500_000;

pub struct Ibex {
    userspace_kernel_boundary: SysCall,
}

impl Ibex {
    pub unsafe fn new() -> Ibex {
        Ibex {
            userspace_kernel_boundary: SysCall::new(),
        }
    }

    pub unsafe fn enable_plic_interrupts(&self) {
        plic::disable_all();
        plic::clear_all_pending();
        plic::enable_all();
    }
}

impl kernel::Chip for Ibex {
    type MPU = ();
    type UserspaceKernelBoundary = SysCall;
    type SysTick = ();

    fn mpu(&self) -> &Self::MPU {
        &()
    }

    fn systick(&self) -> &Self::SysTick {
        &()
    }

    fn userspace_kernel_boundary(&self) -> &SysCall {
        &self.userspace_kernel_boundary
    }

    fn service_pending_interrupts(&self) {
        unsafe {
            // Timer handling, if needed
            timer::TIMER.service_interrupt();

            while let Some(interrupt) = plic::next_pending() {
                match interrupt {
                    interrupts::UART_TX_WATERMARK..interrupts::UART_RX_PARITY_ERR => {
                        uart::UART0.handle_interrupt()
                    }
                    int_pin @ interrupts::GPIO_PIN0..interrupts::GPIO_PIN31 => {
                        let pin = &gpio::PORT[(int_pin - interrupts::GPIO_PIN0) as usize];
                        pin.handle_interrupt();
                    }
                    _ => debug!("Pidx {}", interrupt),
                }

                // Mark that we are done with this interrupt and the hardware
                // can clear it.
                plic::complete(interrupt);
            }
            // Re-enable external interrupts once all entries from PLIC have
            // been processed.
            CSR.mie.modify(mie::mext::SET);
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { plic::has_pending() }
    }

    fn sleep(&self) {
        unsafe {
            rv32i::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        rv32i::support::atomic(f)
    }
}

fn handle_exception(exception: mcause::Exception, mtval: u32) {
    match exception {
        mcause::Exception::InstructionMisaligned => {
            panic!("misaligned instruction {:x}\n", mtval);
        }
        mcause::Exception::InstructionFault => {
            panic!("instruction fault {:x}\n", mtval);
        }
        mcause::Exception::IllegalInstruction => {
            panic!("illegal instruction {:x}\n", mtval);
        }
        mcause::Exception::Breakpoint => {
            debug!("breakpoint\n");
        }
        mcause::Exception::LoadMisaligned => {
            panic!("misaligned load {:x}\n", mtval);
        }
        mcause::Exception::LoadFault => {
            panic!("load fault {:x}\n", mtval);
        }
        mcause::Exception::StoreMisaligned => {
            panic!("misaligned store {:x}\n", mtval);
        }
        mcause::Exception::StoreFault => {
            panic!("store fault {:x}\n", mtval);
        }
        mcause::Exception::UserEnvCall => (),
        mcause::Exception::SupervisorEnvCall => (),
        mcause::Exception::MachineEnvCall => {
            // GENERATED BY ECALL; should never happen....
            panic!("machine mode environment call\n");
        }
        mcause::Exception::InstructionPageFault => {
            panic!("instruction page fault {:x}\n", mtval);
        }
        mcause::Exception::LoadPageFault => {
            panic!("load page fault {:x}\n", mtval);
        }
        mcause::Exception::StorePageFault => {
            panic!("store page fault {:x}\n", mtval);
        }
        mcause::Exception::Unknown => {
            panic!("exception type unknown");
        }
    }
}

unsafe fn handle_interrupt(intr: mcause::Interrupt) {
    match intr {
        mcause::Interrupt::UserSoft
        | mcause::Interrupt::UserTimer
        | mcause::Interrupt::UserExternal => {
            debug!("unexpected user-mode interrupt");
        }
        mcause::Interrupt::SupervisorExternal
        | mcause::Interrupt::SupervisorTimer
        | mcause::Interrupt::SupervisorSoft => {
            debug!("unexpected supervisor-mode interrupt");
        }

        mcause::Interrupt::MachineSoft => {
            CSR.mie.modify(mie::msoft::CLEAR);
        }
        mcause::Interrupt::MachineTimer => {
            CSR.mie.modify(mie::mtimer::CLEAR);
            // timer handling done in kernel loop
        }
        mcause::Interrupt::MachineExternal => {
            CSR.mie.modify(mie::mext::CLEAR);
        }

        mcause::Interrupt::Unknown => {
            debug!("interrupt of unknown cause");
        }
    }
}

/// Trap handler for board/chip specific code.
///
/// For the Ibex this gets called when an interrupt occurs while the chip is
/// in kernel mode. All we need to do is check which interrupt occurred and
/// disable it.
#[export_name = "_start_trap_rust"]
pub unsafe extern "C" fn start_trap_rust() {
    let cause = CSR.mcause.extract();

    match mcause::McauseHelpers::cause(&cause) {
        mcause::Trap::Interrupt(interrupt) => {
            handle_interrupt(interrupt);
        }
        mcause::Trap::Exception(exception) => {
            handle_exception(exception, CSR.mtval.get());
        }
    }
}

/// Function that gets called if an interrupt occurs while an app was running.
/// mcause is passed in, and this function should correctly handle disabling the
/// interrupt that fired so that it does not trigger again.
#[export_name = "_disable_interrupt_trap_handler"]
pub unsafe extern "C" fn disable_interrupt_trap_handler(_mcause: u32) {
    // TODO: reuse _mcause from above
    let cause = CSR.mcause.extract();

    match mcause::McauseHelpers::cause(&cause) {
        mcause::Trap::Interrupt(interrupt) => {
            handle_interrupt(interrupt);
        }
        _ => {
            panic!("unexpected non-interrupt\n");
        }
    }
}

pub unsafe fn configure_trap_handler() {
    // The Ibex CPU does not support non-vectored trap entries.
    CSR.mtvec.write(
        mtvec::trap_addr.val(_start_trap_vectored as u32 >> 2)
            + mtvec::mode::Vectored,
    )
}

#[link_section = ".riscv.trap_vectored"]
#[export_name = "_start_trap_vectored"]
#[naked]
pub extern "C" fn _start_trap_vectored() -> ! {
    unsafe {
        // According to the Ibex user manual:
        // [NMI] has interrupt ID 31, i.e., it has the highest priority of all
        // interrupts and the core jumps to the trap-handler base address (in
        // mtvec) plus 0x7C to handle the NMI.
        //
        // Below are 32 (non-compressed) jumps to cover the entire possible
        // range of vectored traps.
        asm!("
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
            j _start_trap
        "
        :
        :
        :
        : "volatile");
        unreachable_unchecked()
    }
}
