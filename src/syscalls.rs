#![no_std]
#![no_main]

pub fn raw_malloc(bytes: usize) -> usize {
    unsafe {
        let mut return_value: usize = 0x0;
        core::arch::asm!(
            "ecall",
            in("a2") 0x1,
            in("a3") bytes,
            out("a0") return_value
        );
        return_value
    }
}
