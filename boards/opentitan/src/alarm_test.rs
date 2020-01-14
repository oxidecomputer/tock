use core::cell::Cell;
use kernel::hil::gpio::Pin;
use kernel::hil::time::{Alarm, AlarmClient, Frequency};

pub struct TestAlarm<'a, A: Alarm<'a>> {
    alarm: &'a A,
    count: Cell<u32>,
    interval: Cell<u32>,
    toggle_pin: Option<&'a dyn Pin>,
}

fn ms_to_tick<'a, A: Alarm<'a>>(ms: u32) -> u32 {
    let freq = <A::Frequency>::frequency();
    let ftick = (freq as u64) * (ms as u64);

    (ftick / 1000) as u32
}

impl<'a, A: Alarm<'a>> TestAlarm<'a, A> {
    pub fn new(alarm: &'a A, pin: Option<&'a dyn Pin>) -> TestAlarm<'a, A> {
        TestAlarm {
            alarm: alarm,
            count: Cell::new(0),
            interval: Cell::new(0),
            toggle_pin: pin,
        }
    }

    pub fn run(&self, initial_ms: u32, interval_ms: u32, count: u32) {
        self.count.set(count);
        self.interval.set(ms_to_tick::<A>(interval_ms));
        self.set_next_alarm(ms_to_tick::<A>(initial_ms));
    }

    fn set_next_alarm(&self, tick: u32) {
        self.alarm.set_alarm(tick);
    }
}

impl<'a, A: Alarm<'a>> AlarmClient for TestAlarm<'a, A> {
    fn fired(&self) {
        self.toggle_pin.map(|pin| Pin::toggle(pin));
        let c = self.count.get();
        if c > 0 {
            self.count.set(c - 1);
            let now = self.alarm.now();
            let expire = now.wrapping_add(self.interval.get());
            self.set_next_alarm(expire);
        }
    }
}
