build:
	cargo build
run: build
	qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios ./bootloader/fw_jump.bin \
	-device loader,file=target/riscv64gc-unknown-none-elf/debug/os,addr=0x80200000

gdb:
	qemu-system-riscv64 \
	-machine virt \
	-nographic \
	-bios ./bootloader/rustsbi-qemu.bin \
	-device loader,file=target/riscv64gc-unknown-none-elf/debug/os,addr=0x80200000 -S -s

.PHONY: run
.PHONY: gdb

