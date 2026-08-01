#![no_std]
#![no_main]

use core::ptr;
use core::mem;

core::arch::global_asm!(include_str!("entry.s"));

static logo: &str = "\x1B[0;36m  █████╗  ███╗   ██╗███╗   ██╗ █████╗      ██████╗ ███████╗\n ██╔══██╗ ████╗  ██║████╗  ██║██╔══██╗    ██╔═══██╗██╔════╝\n ███████║ ██╔██╗ ██║██╔██╗ ██║███████║    ██║   ██║███████╗\n ██╔══██║ ██║╚██╗██║██║╚██╗██║██╔══██║    ██║   ██║╚════██║\n ██║  ██║ ██║ ╚████║██║ ╚████║██║  ██║    ╚██████╔╝███████║\n ╚═╝  ╚═╝ ╚═╝  ╚═══╝╚═╝  ╚═══╝╚═╝  ╚═╝     ╚═════╝ ╚══════╝\x1B[0m\n";



unsafe extern "C" {
    static mut start_bss: u8;
    static mut end_bss: u8;
    static mut b_psp: u8;
    static mut t_psp: u8;
    static mut t_sp: usize;
    fn trap_vector();
}

fn bss_clear() {
    let start_bss_ptr = core::ptr::addr_of_mut!(start_bss);
    let end_bss_ptr = core::ptr::addr_of_mut!(end_bss);
    let count = (end_bss_ptr as usize) - (start_bss_ptr as usize);
    unsafe { core::ptr::write_bytes(start_bss_ptr, 0, count); }
}

static UART_ADDR: usize = 0x1000_0000;


static mut CURRENT_PROCESS: usize = 0;

#[repr(C)]
#[derive(Clone, Copy)]
struct TrapFrame {
    regs: [usize; 32],
    kernel_sp: usize,
    sepc: usize,
}
impl TrapFrame {
    const fn new() -> TrapFrame {
        TrapFrame { regs: [0; 32], kernel_sp: 0x0, sepc: 0x0 }
    }
}

static mut TRAP_FRAME: TrapFrame = TrapFrame {
    regs: [0; 32],
    kernel_sp: 0x0,
    sepc: 0x0,
};


#[repr(C)]
#[derive(Clone, Copy)]
struct ProcessEntry {
    proc_id: u8,
    trapframe: TrapFrame,
}
impl ProcessEntry {
    const fn new() -> ProcessEntry {
        ProcessEntry { proc_id: 0, trapframe: TrapFrame::new() }
    }
}

static PROCSTACK_SIZE: usize = 512;
static PROC_NUM: usize = 9;
static mut PROCSTACK: [[u8; PROCSTACK_SIZE]; PROC_NUM] = [[0; PROCSTACK_SIZE]; PROC_NUM];
static mut PROC_ENTRIES: [ProcessEntry; PROC_NUM] = [ProcessEntry::new(); PROC_NUM];




fn user_process_1() -> ! {
    let uart_driver = UartDriver::new(UART_ADDR);
    uart_driver.write_str("\x1B[2J\x1B[H");
    uart_driver.write_char(b'\n');
    uart_driver.write_str("Hello from process 0\n\n");
    uart_shell();
}
fn user_process_2() -> ! {
    let uart_driver = UartDriver::new(UART_ADDR);
    uart_driver.write_str("\x1B[2J\x1B[H");
    uart_driver.write_char(b'\n');
    uart_driver.write_str("Hello from process 1\n\n");
    uart_shell();
}
fn user_process_3() -> ! {
    let uart_driver = UartDriver::new(UART_ADDR);
    uart_driver.write_str("\x1B[2J\x1B[H");
    uart_driver.write_char(b'\n');
    uart_driver.write_str("Hello from process 2\n\n");
    uart_shell();
}


fn launch_user_process() {
    let mut sstatus: usize;
    unsafe {
        core::arch::asm!("csrw sepc, {0}", in(reg) user_process_1 as usize); 
        core::arch::asm!("csrr {0}, sstatus", out(reg) sstatus);
        sstatus = sstatus & !(1 << 8);
        core::arch::asm!("csrw sstatus, {0}", in(reg) sstatus);
        core::arch::asm!("la sp, t_psp");
        core::arch::asm!("sret");
    }
}


fn scheduler() {
    // the scheduler is ran at the trap handler 
    // And it needs to load that next process's trapframe address into sscratch 
    // And then it needs to change sstatus to return to U-mode
    // And then return and let the trap handler return too
    
    let uart_driver: UartDriver = UartDriver::new(UART_ADDR);
    // uart_driver.write_str("\nScheduler running.\n");
    let mut sstatus: usize;
    let mut sscratch: usize;
    unsafe {
        let current_process = CURRENT_PROCESS;
         
        sscratch = core::ptr::addr_of_mut!(PROC_ENTRIES[current_process].trapframe) as usize;
        core::arch::asm!("csrw sscratch, {0}", in(reg) sscratch);
        core::arch::asm!("csrr {0}, sstatus", out(reg) sstatus);
        sstatus = sstatus & !(1 << 8);
        core::arch::asm!("csrw sstatus, {0}", in(reg) sstatus);
        // uart_driver.write_str("\nScheduler finished.\n");

        // uart_shell();
        CURRENT_PROCESS = (CURRENT_PROCESS + 1) % PROC_NUM;
    }
}


fn init_processes() {
    for i in 0..PROC_NUM {
        unsafe {
            let current_proc_stack: usize = core::ptr::addr_of_mut!(PROCSTACK[i]) as usize;
            PROC_ENTRIES[i].trapframe.regs[2] = current_proc_stack + PROCSTACK_SIZE;
            PROC_ENTRIES[i].trapframe.sepc = if i == 0 {
                user_process_1 as usize
            } else if i == 1 {
                user_process_2 as usize
            } else {
                user_process_3 as usize
            };
            PROC_ENTRIES[i].trapframe.kernel_sp = core::ptr::addr_of_mut!(t_sp) as usize;
        }
    }
}


struct MemoryNode {
    memory: [u8; 4096],
    previous_node: usize,
    next_node: usize,
}

fn init_freelist() {
    // We want to be able to return a pointer to a memory address that is zeroed out and can be used
    // for whatever.
    //
    // One approach is to separate a big chunk of memory into standard chunks of a fixed size each.
    // Like 4096 bytes each.
    // So if a user wants 4097 bytes, they get 2 chunks. We round up. 
    // But aren't we wasting memory?
    // I suppose we could tell the next node after "hey, you can use 4095 bytes from the previous
    // node"

}

fn init() {
    // clearing bss section
    bss_clear();
    // setting up trap vector
    unsafe { core::arch::asm!("csrw stvec, {0}", in(reg) trap_vector as usize); }
    unsafe { TRAP_FRAME.kernel_sp = core::ptr::addr_of_mut!(t_sp) as usize; }
    let trapframe_ptr = core::ptr::addr_of_mut!(TRAP_FRAME) as usize;

    unsafe { core::arch::asm!("csrw sscratch, {0}", in(reg) trapframe_ptr); }

    // enabling interrupts
    let mut sstatus: usize;
    unsafe { core::arch::asm!("csrr {0}, sstatus", out(reg) sstatus); }
    sstatus |= 0x2;
    unsafe { core::arch::asm!("csrw sstatus, {0}", in(reg) sstatus); }
    // setting up user process pool
    init_processes();
    // enabling timer interrupts
    let mut sie: usize;
    unsafe { core::arch::asm!("csrr {0}, sie", out(reg) sie); }
    sie |= 0x20;
    unsafe { core::arch::asm!("csrw sie, {0}", in(reg) sie); }

}


fn intro(uart_driver: &UartDriver) {
    uart_driver.write_str("\x1B[2J\x1B[H");
    uart_driver.write_str(logo);
    uart_driver.write_str("\n");
    uart_driver.write_str("\x1B[0;34mVersion 1.0\x1B[0m\n");

    uart_driver.write_str("\n");
    uart_driver.write_str("\x1B[0;32m===========================================================\x1B[0m\n");
    uart_driver.write_str("\n");

    uart_driver.write_str("\x1B[0;34mWelcome :3\x1B[0m\n");
    uart_driver.write_str("\n");
}


fn uart_shell() -> ! {
    let uart_driver = UartDriver::new(UART_ADDR);
    intro(&uart_driver);

    uart_driver.write_str("\x1B[0;36m>> \x1B[0m");
    loop { 
        let input_c = uart_driver.read_char();
        uart_driver.write_char(input_c);
        if input_c == 0x0D {
            uart_driver.write_str("\n\x1B[0;36m>> \x1B[0m");
        }
    }
}

#[unsafe(no_mangle)]
extern "C" fn kmain() -> ! {
    init();
    loop { }
}



#[repr(C)]
struct UartRegisters {
    thr: u8,
    _unused: [u8; 4],
    lsr: u8,
}


struct UartDriver {
    registers: *mut UartRegisters,
}

impl UartDriver {
    fn new(addr: usize) -> UartDriver {
        return UartDriver { 
            registers: addr as *mut UartRegisters
        };
    }
    fn write_char(&self, c: u8) -> u8 {
        unsafe {
            loop {
                let status = core::ptr::read_volatile(core::ptr::addr_of!((*self.registers).lsr));
                if status & 0x20 != 0 {
                    break;
                }
            }
            core::ptr::write_volatile(
                core::ptr::addr_of_mut!((*self.registers).thr),
                c,
            );
        }
        0
    }
    fn write_str(&self, str: &str) -> u8 {
        for c in str.as_bytes() {
            self.write_char(*c);
        }
        0
    }

     fn read_char(&self) -> u8 {
         unsafe {
             loop {
                 let status = core::ptr::read_volatile(core::ptr::addr_of!((*self.registers).lsr));
                 if status & 0x01 != 0 {
                     break;
                 }
             }
             core::ptr::read_volatile(core::ptr::addr_of!((*self.registers).thr))
         }
        
     }
}


#[unsafe(no_mangle)]
extern "C" fn trap_handler() {
    let cause: usize;
    let mut sepc: usize;
    let uart = UartDriver::new(UART_ADDR);

    unsafe { 
        core::arch::asm!("csrr {0}, scause", out(reg) cause); 
        core::arch::asm!("csrr {0}, sepc", out(reg) sepc);
    }
    let CAUSE_IS_EBREAK = cause >> 63 == 0b0;
    let IS_TIMER_INTERRUPT = cause & !(1 << 63) == 0x5;


    // uart.write_str("Trap encountered!\n");

    if CAUSE_IS_EBREAK {
        uart.write_str("ebreak detected!\n");


        let instruction = unsafe { *(sepc as *const u16) };
        let len = if instruction & 0x3 == 0b11 {
            4
        } else {
            2
        };

        sepc += len;
        unsafe { core::arch::asm!("csrw sepc, {0}", in(reg) sepc); }
    } else {
        // uart.write_str("\nInterrupt detected\n");
        uart.write_str("\nInterrupt detected\n");


        if IS_TIMER_INTERRUPT {
            let mut now: usize;
            unsafe { core::arch::asm!("csrr {0}, time", out(reg) now); }
            now += 10000000;


            unsafe { 
                core::arch::asm!(
                    "ecall",
                    in("a0") now,
                    in ("a6") 0,
                    in ("a7") 0x54494D45
                );
                scheduler();
                return
            }
        }
    }

}




#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {

    }
}
