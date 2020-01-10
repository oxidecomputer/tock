//! Timer driver.

use kernel::common::cells::OptionalCell;
use kernel::common::registers::{register_bitfields, register_structs, ReadWrite, WriteOnly};
use kernel::common::StaticRef;
use kernel::hil::time;

use crate::chip::CHIP_FREQ;

#[cfg(not(feature = "verilator"))]
const PRESCALE: u16 = ((CHIP_FREQ / 16_000) - 1) as u16; // 16Khz
#[cfg(feature = "verilator")]
const PRESCALE: u16 = ((CHIP_FREQ / 1_000) - 1) as u16; // 1Khz

register_structs! {
    pub TimerRegisters {
        (0x000 => ctrl: ReadWrite<u32, ctrl::Register>),

        (0x004 => _reserved),

        (0x100 => config: ReadWrite<u32, config::Register>),

        (0x104 => value_low: ReadWrite<u32>),
        (0x108 => value_high: ReadWrite<u32>),

        (0x10c => compare_low: ReadWrite<u32>),
        (0x110 => compare_high: ReadWrite<u32>),

        (0x114 => intr_enable: ReadWrite<u32, intr::Register>),
        (0x118 => intr_state: ReadWrite<u32, intr::Register>),
        (0x11c => intr_test: WriteOnly<u32, intr::Register>),
        (0x120 => @END),
    }
}

register_bitfields![u32,
    ctrl [
        enable OFFSET(0) NUMBITS(1) []
    ],
    config [
        prescale OFFSET(0) NUMBITS(12) [],
        step OFFSET(16) NUMBITS(8) []
    ],
    intr [
        timer0 OFFSET(0) NUMBITS(1) []
    ]
];

pub struct RvTimer<'a> {
    registers: StaticRef<TimerRegisters>,
    client: OptionalCell<&'a dyn time::AlarmClient>,
}

impl<'a> RvTimer<'a> {
    const fn new(base: StaticRef<TimerRegisters>) -> RvTimer<'a> {
        RvTimer {
            registers: base,
            client: OptionalCell::empty(),
        }
    }

    pub fn setup(&self) {
        // Set proper prescaler and the like
        self.registers
            .config
            .write(config::prescale.val(PRESCALE as u32) + config::step.val(1u32));
        self.registers.intr_enable.write(intr::timer0::CLEAR);
        self.registers.compare_high.set(0xffff_ffff);
        self.registers.ctrl.write(ctrl::enable::SET);
    }

    pub fn service_interrupt(&self) {
        if self.registers.intr_state.is_set(intr::timer0) {
            self.registers.intr_enable.write(intr::timer0::CLEAR);
            self.registers.intr_state.write(intr::timer0::SET);
            self.client.map(|client| {
                client.fired();
            });
        }
    }
}

impl<'a> time::Time for RvTimer<'a> {
    #[cfg(not(feature = "verilator"))]
    type Frequency = time::Freq16KHz;
    #[cfg(feature = "verilator")]
    type Frequency = time::Freq1KHz;

    fn now(&self) -> u32 {
        self.registers.value_low.get()
    }
    fn max_tics(&self) -> u32 {
        core::u32::MAX
    }
}

impl<'a> time::Alarm<'a> for RvTimer<'a> {
    fn set_client(&self, client: &'a dyn time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, tics: u32) {
        // Clear any existing interrupt state.
        self.registers.intr_state.write(intr::timer0::SET);

        // high bits in the timer go unused for now
        self.registers.value_high.set(0);

        self.registers.compare_low.set(tics);
        self.registers.compare_high.set(0);
        self.registers.intr_enable.write(intr::timer0::SET);

        rv32i::csr::CSR
            .mie
            .modify(rv32i::csr::mie::mie::mtimer::SET);
    }

    fn get_alarm(&self) -> u32 {
        self.registers.compare_low.get()
    }

    fn disable(&self) {
        self.registers.intr_enable.write(intr::timer0::CLEAR);
        self.registers.compare_high.set(0xffff_ffff);
        self.registers.intr_state.write(intr::timer0::SET);
    }

    fn is_enabled(&self) -> bool {
        self.registers.intr_enable.is_set(intr::timer0)
    }
}

const TIMER_BASE: StaticRef<TimerRegisters> =
    unsafe { StaticRef::new(0x4008_0000 as *const TimerRegisters) };

pub static mut TIMER: RvTimer = RvTimer::new(TIMER_BASE);
