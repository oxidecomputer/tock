//! Board file for LowRISC OpenTitan RISC-V development platform.
//!
//! - <https://opentitan.org/>

#![no_std]
#![no_main]
#![feature(asm)]

use capsules::virtual_alarm::{MuxAlarm, VirtualMuxAlarm};
use capsules::virtual_uart::MuxUart;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::Platform;
use kernel::{create_capability, debug, static_init};
use rv32i::csr;

mod alarm_test;

pub mod io;
//
// Actual memory for holding the active process structures. Need an empty list
// at least.
static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; 4] =
    [None, None, None, None];

// How should the kernel respond when a process faults.
const FAULT_RESPONSE: kernel::procs::FaultResponse = kernel::procs::FaultResponse::Panic;

// RAM to be shared by all application processes.
#[link_section = ".app_memory"]
static mut APP_MEMORY: [u8; 8192] = [0; 8192];

// Force the emission of the `.apps` segment in the kernel elf image
// NOTE: This will cause the kernel to overwrite any existing apps when flashed!
#[used]
#[link_section = ".app.hack"]
static APP_HACK: u8 = 0;

/// Dummy buffer that causes the linker to reserve enough space for the stack.
#[no_mangle]
#[link_section = ".stack_buffer"]
pub static mut STACK_MEMORY: [u8; 0x1000] = [0; 0x1000];

#[cfg(not(feature = "verilator"))]
const UART_BAUD: u32 = 230_400;
#[cfg(feature = "verilator")]
const UART_BAUD: u32 = 9_600;

/// A structure representing this platform that holds references to all
/// capsules for this platform. We've included an alarm and console.
struct OpenTitan {
    console: &'static capsules::console::Console<'static>,
    alarm: &'static capsules::alarm::AlarmDriver<
        'static,
        VirtualMuxAlarm<'static, ibex::timer::RvTimer<'static>>,
    >,
}

/// Mapping of integer syscalls to objects that implement syscalls.
impl Platform for OpenTitan {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.console)),
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            _ => f(None),
        }
    }
}

/// Reset Handler.
///
/// This function is called from the arch crate after some very basic RISC-V
/// setup.
#[no_mangle]
pub unsafe fn reset_handler() {
    // Basic setup of the platform.
    rv32i::init_memory();
    // Ibex-specific handler
    ibex::chip::configure_trap_handler();

    // initialize capabilities
    let process_mgmt_cap = create_capability!(capabilities::ProcessManagementCapability);
    let memory_allocation_cap = create_capability!(capabilities::MemoryAllocationCapability);

    let main_loop_cap = create_capability!(capabilities::MainLoopCapability);

    let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

    // Configure kernel debug gpios as early as possible
    kernel::debug::assign_gpios(
        Some(&ibex::gpio::PORT[7]), // First LED
        None,
        None,
    );

    let chip = static_init!(ibex::chip::Ibex, ibex::chip::Ibex::new());

    // Need to enable all interrupts for Tock Kernel
    chip.enable_plic_interrupts();
    // enable interrupts globally
    csr::CSR
        .mie
        .modify(csr::mie::mie::msoft::SET + csr::mie::mie::mtimer::SET + csr::mie::mie::mext::SET);
    csr::CSR.mstatus.modify(csr::mstatus::mstatus::mie::SET);

    // Create a shared UART channel for the console and for kernel debug.
    let uart_mux = static_init!(
        MuxUart<'static>,
        MuxUart::new(
            &ibex::uart::UART0,
            &mut capsules::virtual_uart::RX_BUF,
            UART_BAUD
        )
    );

    uart_mux.initialize();

    hil::uart::Transmit::set_transmit_client(&ibex::uart::UART0, uart_mux);
    hil::uart::Receive::set_receive_client(&ibex::uart::UART0, uart_mux);

    // Initialise the three GPIOs which are useful for debugging.
    hil::gpio::Pin::make_output(&ibex::gpio::PORT[8]);
    hil::gpio::Pin::set(&ibex::gpio::PORT[8]);

    hil::gpio::Pin::make_output(&ibex::gpio::PORT[9]);
    hil::gpio::Pin::set(&ibex::gpio::PORT[9]);

    hil::gpio::Pin::make_output(&ibex::gpio::PORT[10]);
    hil::gpio::Pin::set(&ibex::gpio::PORT[10]);

    let alarm = &ibex::timer::TIMER;
    alarm.setup();

    // Create a shared virtualization mux layer on top of a single hardware
    // alarm.
    let mux_alarm = static_init!(
        MuxAlarm<'static, ibex::timer::RvTimer>,
        MuxAlarm::new(alarm)
    );
    hil::time::Alarm::set_client(&ibex::timer::TIMER, mux_alarm);

    // Alarm
    let virtual_alarm_user = static_init!(
        VirtualMuxAlarm<'static, ibex::timer::RvTimer>,
        VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm = static_init!(
        capsules::alarm::AlarmDriver<'static, VirtualMuxAlarm<'static, ibex::timer::RvTimer>>,
        capsules::alarm::AlarmDriver::new(
            virtual_alarm_user,
            board_kernel.create_grant(&memory_allocation_cap)
        )
    );
    hil::time::Alarm::set_client(virtual_alarm_user, alarm);

    // Do alarm test
    let alarm_test_mux = static_init!(
        capsules::virtual_alarm::VirtualMuxAlarm<'static, ibex::timer::RvTimer>,
        capsules::virtual_alarm::VirtualMuxAlarm::new(mux_alarm)
    );
    let alarm_test_inst = static_init!(
        alarm_test::TestAlarm<
            'static,
            capsules::virtual_alarm::VirtualMuxAlarm<'static, ibex::timer::RvTimer>,
        >,
        alarm_test::TestAlarm::new(alarm_test_mux, Some(&ibex::gpio::PORT[11]))
    );
    hil::time::Alarm::set_client(alarm_test_mux, alarm_test_inst);
    // Wait 100ms, then toggle the LED every 200ms (for 100 iterations)
    alarm_test_inst.run(100, 200, 100);

    // Ready the pin for toggling by the test
    hil::gpio::Pin::make_output(&ibex::gpio::PORT[11]);
    hil::gpio::Pin::clear(&ibex::gpio::PORT[11]);


    // Setup the console.
    let console = components::console::ConsoleComponent::new(board_kernel, uart_mux).finalize(());
    // Create the debugger object that handles calls to `debug!()`.
    components::debug_writer::DebugWriterComponent::new(uart_mux).finalize(());

    debug!("OpenTitan initialisation complete. Entering main loop");

    extern "C" {
        /// Beginning of the ROM region containing app images.
        ///
        /// This symbol is defined in the linker script.
        static _sapps: u8;
    }

    let opentitan = OpenTitan {
        console: console,
        alarm: alarm,
    };

    kernel::procs::load_processes(
        board_kernel,
        chip,
        &_sapps as *const u8,
        &mut APP_MEMORY,
        &mut PROCESSES,
        FAULT_RESPONSE,
        &process_mgmt_cap,
    );

    // Turn off the fourth GPIO so we know we got here
    hil::gpio::Pin::clear(&ibex::gpio::PORT[10]);

    board_kernel.kernel_loop(&opentitan, chip, None, &main_loop_cap);
}
