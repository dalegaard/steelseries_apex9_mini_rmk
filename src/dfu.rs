use core::arch::asm;
use cortex_m::{interrupt::free, peripheral::SCB};

const BOOTLOADER_FLAG_MAGIC: u64 = 0x424f4f544c4f4144;

const BOOTLOADER_BASE_ADDR: u32 = 0x1FFF_0000;

#[unsafe(link_section = ".uninit")]
static mut BOOTLOADER_FLAGS: DfuEntry = DfuEntry { enter: 0 };

#[repr(align(32))]
pub struct DfuEntry {
    enter: u64,
}

#[inline(never)]
pub fn enter() -> ! {
    free(|_| unsafe {
        BOOTLOADER_FLAGS.enter = BOOTLOADER_FLAG_MAGIC;
        SCB::sys_reset();
    });
    panic!();
}

#[inline(never)]
pub fn check_enter() {
    unsafe {
        if BOOTLOADER_FLAGS.enter == BOOTLOADER_FLAG_MAGIC {
            BOOTLOADER_FLAGS.enter = 0;

            defmt::info!("Entering bootloader");
            asm!(
                "ldrd {sp}, {code}, [{bl}]",
                "mov sp, {sp}",
                "bx {code}",
                bl = in(reg) BOOTLOADER_BASE_ADDR,
                sp = out(reg) _,
                code = out(reg) _
            );
        }
    }
}
