# Aug 23, 2026
Problem: OS does not switch between different processes and blocks input

## Diagnosis
### test 1
- User process 1 is reached, and it's uart_shell is launched.
- No input from keyboard is printed using uart_driver.
- User process 2 is not reached. 

### test 2 
- breaking at process 1's uart_shell `uart_driver.read_char()` and at `input[i]
  = input_c` allows me to type in input, and then after pressing continue, my
  characters are printed. However after all my input is printed and I continue,
  it hangs at `uart_driver.read_char()` and no matter what I type, it remains
  hanged there.

It's almost like, when input enters the UART buffer before the contents of it
are read, it works, but otherwise it doesnt.

### test 3 - increasing procstack size from 512 bytes to 4096 bytes. Ruling out stack corruption.
- process switching succeeded. Input printing works.
- around the 7-8th process switch, an "ebreak" event occurred. More granular
  error reporting is required.
- identified event type: scause 0x1 - instruction access fault. 

### test 4 - increasing procstack size from 4096 bytes to 8192 bytes.
- No instruction access fault this time.


Theory: stack corruption is causing errors. Possibility:
processes are messing with eachother's stacks.

I want to see, as the processes switch, which stack each process is using.

### test 5 
stack pointers at each process switch before the loop around:
process switch 0: 0x80205358
process switch 1: 0x80207358
process switch 2: 0x80209358
process switch 3: 0x8020b358
process switch 4: 0x8020d358
process switch 5: 0x8020f358
process switch 6: 0x80211358
process switch 7: 0x80213358
process switch 8: 0x80215358

so after the last process switch, and we switch back to process 1, i see this:

x0/zero  0000000000000000 x1/ra    0000000080200d28 x2/sp    0000000080214298
x3/gp    0000000000000000

sp: 0x80214298 -> that's in process 8's stack.

after process 1 round 2:

x0/zero  0000000000000000 x1/ra    0000000080200d28 x2/sp    0000000080206298
x3/gp    0000000000000000


wait no, after we switch back to process 0 we see this: x0/zero
0000000000000000 x1/ra    0000000080200d28 x2/sp    0000000080204298 x3/gp
0000000000000000


### test 6
reverted back to 4096 procstack size. It works now?


### test 7
going back to 512 breaks the OS just like before. cannot type in input, no process switching.


### test 8 
switching back to 4096 size causes the instruction access fault

### test 9
going back to 8192 size fixes it, then going back to 4096 breaks it the same way.


when the load access fault happens, these are the regs:

CPU#0
 V      =   0
 pc       0000000080200162
 mhartid  0000000000000000
 mstatus  8000000a000060a0
 hstatus  0000000200000000
 vsstatus 0000000a00000000
 mip      0000000000000020
 mie      00000000000000a8
 mideleg  0000000000001666
 hideleg  0000000000000000
 medeleg  0000000000f4b509
 hedeleg  0000000000000000
 mtvec    00000000800004f8
 stvec    0000000080200020
 vstvec   0000000000000000
 mepc     0000000080200020
 sepc     0000000000000000
 vsepc    0000000000000000
 mcause   0000000000000001
 scause   0000000000000001
 vscause  0000000000000000
 mtval    0000000000000000
 stval    0000000000000000
 htval    0000000000000000
 mtval2   0000000000000000
 mscratch 0000000080045000
 sscratch 0000000080203148
 satp     0000000000000000
 x0/zero  0000000000000000 x1/ra    000000008020134c x2/sp    00000000802031b8 x3/gp    0000000000000001
 x4/tp    0000000010000000 x5/t0    0000000080203148 x6/t1    0000000000000000 x7/t2    0000000000000000
 x8/s0    00000000802031d8 x9/s1    0000000000000001 x10/a0   00000000802031f0 x11/a1   0000000000000043
 x12/a2   0000000000000000 x13/a3   00000000802020e0 x14/a4   0000000000000001 x15/a5   0000000110000000
 x16/a6   0000000080203218 x17/a7   0000000080200f78 x18/s2   0000000080202040 x19/s3   0000000010000000
 x20/s4   0000000000000001 x21/s5   0000000010000000 x22/s6   0000000010000005 x23/s7   0000000000000001
 x24/s8   0000000080203258 x25/s9   0000000000000001 x26/s10  0000000010000005 x27/s11  0000000010000005
 x28/t3   0000000000000001 x29/t4   00000001802032a0 x30/t5   0000000080201a1e x31/t6   0000000080201a1e
 fcsr     0000000000000000
 f0/ft0   ffffffff00000000 f1/ft1   ffffffff00000000 f2/ft2   ffffffff00000000 f3/ft3   ffffffff00000000
 f4/ft4   ffffffff00000000 f5/ft5   ffffffff00000000 f6/ft6   ffffffff00000000 f7/ft7   ffffffff00000000
 f8/fs0   ffffffff00000000 f9/fs1   ffffffff00000000 f10/fa0  ffffffff00000000 f11/fa1  ffffffff00000000
 f12/fa2  ffffffff00000000 f13/fa3  ffffffff00000000 f14/fa4  ffffffff00000000 f15/fa5  ffffffff00000000
 f16/fa6  ffffffff00000000 f17/fa7  ffffffff00000000 f18/fs2  ffffffff00000000 f19/fs3  ffffffff00000000
 f20/fs4  ffffffff00000000 f21/fs5  ffffffff00000000 f22/fs6  ffffffff00000000 f23/fs7  ffffffff00000000
 f24/fs8  ffffffff00000000 f25/fs9  ffffffff00000000 f26/fs10 ffffffff00000000 f27/fs11 ffffffff00000000
 f28/ft8  ffffffff00000000 f29/ft9  ffffffff00000000 f30/ft10 ffffffff00000000 f31/ft11 ffffffff00000000

objdump of the PC:

 (standard input)-80200154: 1101         	addi	sp, sp, -0x20
(standard input)-80200156: ec06         	sd	ra, 0x18(sp)
(standard input)-80200158: e822         	sd	s0, 0x10(sp)
(standard input)-8020015a: 1000         	addi	s0, sp, 0x20
(standard input)-8020015c: fea43423     	sd	a0, -0x18(s0)
(standard input):80200160: a009         	j	0x80200162 <__rustc::rust_begin_unwind+0xe>
(standard input):80200162: a001         	j	0x80200162 <__rustc::rust_begin_unwind+0xe>
(standard input)-
(standard input)-0000000080200164 <riscv_os::uart_shell>:
(standard input)-80200164: 7141         	addi	sp, sp, -0x1f0
(standard input)-80200166: f786         	sd	ra, 0x1e8(sp)
(standard input)-80200168: f3a2         	sd	s0, 0x1e0(sp)

### test 9 
ra             0x802000c8	0x802000c8 <skip_umode_only+8>
sp             0x80203228	0x80203228 <riscv_os::PROC_ENTRIES+2192>
gp             0x1	0x1
tp             0x10000000	0x10000000
t0             0x80203148	2149593416
t1             0x0	0
t2             0x0	0
fp             0x80203298	0x80203298 <riscv_os::PROC_ENTRIES+2304>
s1             0x1	1
a0             0x10000000	268435456
a1             0x10000000	268435456
a2             0x1	1
a3             0x110000005	4563402757
a4             0x1	1
a5             0x110000000	4563402752
a6             0x80203218	2149593624
a7             0x80200f78	2149584760
s2             0x80202040	2149589056
s3             0x10000000	268435456
s4             0x1	1
s5             0x10000000	268435456
s6             0x10000005	268435461
s7             0x1	1
s8             0x80203258	2149593688
s9             0x1	1
s10            0x10000005	268435461
s11            0x10000005	268435461
t3             0x1	1
t4             0x1802032a0	6444561056
t5             0x80201a1e	2149587486
t6             0x80201a1e	2149587486
pc             0x80200aa4	0x80200aa4 <riscv_os::trap_handler+8>


this was the regs right before the ebreak

this was when the panic handler's loop was reached:

(gdb) bt
#0  riscv_os::panic (_info=0x802031f0 <riscv_os::PROC_ENTRIES+2136>)
    at src/main.rs:391
#1  0x000000008020134c in core::panicking::panic_nounwind_fmt::runtime ()
    at library/core/src/panicking.rs:122
#2  core::panicking::panic_nounwind_fmt ()
    at library/core/src/intrinsics/mod.rs:2450
#3  0x000000008020136c in core::panicking::panic_null_pointer_dereference ()
    at library/core/src/panicking.rs:302
#4  0x0000000080200bfa in riscv_os::trap_handler () at src/main.rs:352
#5  0x00000000802000c8 in skip_umode_only ()
Backtrace stopped: frame did not save the PC

info reg

ra             0x8020134c	0x8020134c <core::panicking::panic_null_pointer_dereference>
sp             0x802031b8	0x802031b8 <riscv_os::PROC_ENTRIES+2080>
gp             0x1	0x1
tp             0x10000000	0x10000000
t0             0x80203148	2149593416
t1             0x0	0
t2             0x0	0
fp             0x802031d8	0x802031d8 <riscv_os::PROC_ENTRIES+2112>
s1             0x1	1
a0             0x802031f0	2149593584
a1             0x43	67
a2             0x0	0
a3             0x802020e0	2149589216
a4             0x1	1
a5             0x110000000	4563402752
a6             0x80203218	2149593624
a7             0x80200f78	2149584760
s2             0x80202040	2149589056
s3             0x10000000	268435456
s4             0x1	1
s5             0x10000000	268435456
s6             0x10000005	268435461
s7             0x1	1
s8             0x80203258	2149593688
s9             0x1	1
s10            0x10000005	268435461
s11            0x10000005	268435461
t3             0x1	1
t4             0x1802032a0	6444561056
t5             0x80201a1e	2149587486
t6             0x80201a1e	2149587486
pc             0x80200160	0x80200160 <riscv_os::panic+12>


Solution: I had a 4096-byte array being allocated per user process in the uart
driver, consuming the entirety of the user stack.
