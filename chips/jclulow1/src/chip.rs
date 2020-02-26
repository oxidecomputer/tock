//! Interrupt mapping and DMA channel setup.

use crate::deferred_call_tasks::Task;
use crate::nvic;

use core::fmt::Write;
use cortexm3;
use kernel::common::deferred_call;
use kernel::Chip;

pub struct JClulow1 {
    mpu: cortexm3::mpu::MPU,
    userspace_kernel_boundary: cortexm3::syscall::SysCall,
    systick: cortexm3::systick::SysTick,
}

impl JClulow1{
    pub unsafe fn new() -> JClulow1{
        JClulow1{
            mpu: cortexm3::mpu::MPU::new(),
            userspace_kernel_boundary: cortexm3::syscall::SysCall::new(),
            systick: cortexm3::systick::SysTick::new(),
        }
    }
}

impl Chip for JClulow1{
    type MPU = cortexm3::mpu::MPU;
    type UserspaceKernelBoundary = cortexm3::syscall::SysCall;
    type SysTick = cortexm3::systick::SysTick;

    fn service_pending_interrupts(&self) {
        unsafe {
            loop {
                /*if let Some(task) = deferred_call::DeferredCall::next_pending() {
                    match task {
                    }
                } else*/ if let Some(interrupt) = cortexm3::nvic::next_pending() {
                    match interrupt {
                        _ => {
                            panic!("unhandled interrupt {}", interrupt);
                        }
                    }
                    let n = cortexm3::nvic::Nvic::new(interrupt);
                    n.clear_pending();
                    n.enable();
                } else {
                    break;
                }
            }
        }
    }

    fn has_pending_interrupts(&self) -> bool {
        unsafe { cortexm3::nvic::has_pending() || deferred_call::has_tasks() }
    }

    fn mpu(&self) -> &cortexm3::mpu::MPU {
        &self.mpu
    }

    fn systick(&self) -> &cortexm3::systick::SysTick {
        &self.systick
    }

    fn userspace_kernel_boundary(&self) -> &cortexm3::syscall::SysCall {
        &self.userspace_kernel_boundary
    }

    fn sleep(&self) {
        // if pm::deep_sleep_ready() {
        //     unsafe {
        //         cortexm3::scb::set_sleepdeep();
        //     }
        // } else {
             unsafe {
                 cortexm3::scb::unset_sleepdeep();
             }
        // }

        unsafe {
            cortexm3::support::wfi();
        }
    }

    unsafe fn atomic<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        cortexm3::support::atomic(f)
    }

    unsafe fn print_state(&self, writer: &mut dyn Write) {
        cortexm3::print_cortexm3_state(writer);
    }
}
