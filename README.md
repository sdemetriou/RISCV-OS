# How to Run

## Toolchain setup
You will need qemu-system-riscv64 via `rustup`

```
cargo build
qemu-system-riscv64 -machine virt -nographic -serial mon:stdio -bios default -kernel target/riscv64gc-unknown-none-elf/debug/riscv-os
```
