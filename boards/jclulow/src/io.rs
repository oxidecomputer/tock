use core::fmt::Write;
use core::panic::PanicInfo;
use core::str;
use cortexm3;
use kernel::debug;
use kernel::debug::IoWrite;
use kernel::hil::led;
use kernel::hil::uart::{self, Configure};

use crate::CHIP;
use crate::PROCESSES;

struct Writer {
    initialized: bool,
}

static mut WRITER: Writer = Writer { initialized: false };

impl Write for Writer {
    fn write_str(&mut self, s: &str) -> ::core::fmt::Result {
        self.write(s.as_bytes());
        Ok(())
    }
}

impl IoWrite for Writer {
    fn write(&mut self, buf: &[u8]) {
        /* XXX do nothing for now ... */

// XXX        let uart = unsafe { &mut sam4l::usart::USART0 };
// XXX        let regs_manager = &sam4l::usart::USARTRegManager::panic_new(&uart);
// XXX        if !self.initialized {
// XXX            self.initialized = true;
// XXX            uart.configure(uart::Parameters {
// XXX                baud_rate: 115200,
// XXX                width: uart::Width::Eight,
// XXX                stop_bits: uart::StopBits::One,
// XXX                parity: uart::Parity::None,
// XXX                hw_flow_control: false,
// XXX            });
// XXX            uart.enable_tx(regs_manager);
// XXX        }
// XXX        // XXX: I'd like to get this working the "right" way, but I'm not sure how
// XXX        for &c in buf {
// XXX            uart.send_byte(regs_manager, c);
// XXX            while !uart.tx_ready(regs_manager) {}
// XXX        }
    }
}

/// Panic handler.
#[cfg(not(test))]
#[no_mangle]
#[panic_handler]
pub unsafe extern "C" fn panic_fmt(pi: &PanicInfo) -> ! {
    let writer = &mut WRITER;
    debug::panic(
        &mut [], /* XXX no LEDs... */
        writer,
        pi,
        &cortexm3::support::nop,
        &PROCESSES,
        &CHIP,
    )
}
