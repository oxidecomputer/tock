//! Test that an Alarm implementation is working. Depends on a working
//! UART and debug! macro.
//!
//! Author: Philip Levis <plevis@google.com>
//! Last Modified: 1/10/2020
use core::cell::Cell;
use kernel::debug;
use kernel::hil::time::{Alarm, AlarmClient, Ticks};

pub struct TestAlarm<'a, A: Alarm<'a>> {
    alarm: &'a A,
    ms: Cell<u32>,
}

impl<A: Alarm<'a>> TestAlarm<'a, A> {
    pub fn new(alarm: &'a A) -> TestAlarm<'a, A> {
        TestAlarm {
            alarm: alarm,
            ms: Cell::new(0),
        }
    }

    pub fn run(&self) {
        debug!("Starting alarms.");
        self.ms.set(10000);
        self.set_next_alarm(10000);
    }

    fn set_next_alarm(&self, ms: u32) {
        self.ms.set(ms);
        let now = self.alarm.now();
        let t = now.wrapping_add(A::ticks_from_ms(ms));
        debug!("Setting alarm to {}", t.into_u32());
        self.alarm.set_alarm(t);
    }
}

impl<A: Alarm<'a>> AlarmClient for TestAlarm<'a, A> {
    fn fired(&self) {
        // Generate a new interval that's irregular
        let new_ms: u32 = 10 + ((self.ms.get() + 137) % 757);
        self.set_next_alarm(new_ms);
    }
}
