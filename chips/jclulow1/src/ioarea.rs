use kernel::common::registers::ReadWrite;
use kernel::common::registers::register_bitfields;
use kernel::common::StaticRef;

#[repr(C)]
struct IOArea {
    poop: ReadWrite<u32, Poop::Register>,
}

register_bitfields![
    u32,
    Poop [
        POOP OFFSET(0) NUMBITS(8) []
    ]
];

const IO_AREA_BASE: StaticRef<IOArea> =
    unsafe { StaticRef::new(0x4000_0000 as *mut IOArea) };

pub fn poop(value: u8) {
    let ioa = IO_AREA_BASE;

    ioa.poop.write(Poop::POOP.val(value as u32));
}
