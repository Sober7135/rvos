TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
DISASM_TMP := target/$(TARGET)/$(MODE)/asm
APPS := user/src/bin/*
OBJCOPY := rust-objcopy
GDB := gdb
vi = nvim --noplugin

kernel: $(KERNEL_BIN)

$(KERNEL_BIN): $(KERNEL_ELF)
		@$(OBJCOPY) --strip-all $(KERNEL_ELF) -O binary -I elf64-little  $(KERNEL_BIN)	

$(KERNEL_ELF): os/ user
		@cd os && cargo build --$(MODE)
	
build: kernel

run: build
		@qemu-system-riscv64 \
  	  	-machine virt \
  	  	-nographic \
  	  	-bios ./bootloader/rustsbi-qemu.bin \
  	  	-device loader,file=$(KERNEL_BIN),addr=0x80200000

gdbserver: build
		@qemu-system-riscv64 \
    		-machine virt \
    		-nographic \
    		-bios ./bootloader/rustsbi-qemu.bin \
    		-device loader,file=$(KERNEL_BIN),addr=0x80200000 \
    		-s -S

gdbclient: build env
		@$(GDB) \
    		-ex 'file target/riscv64gc-unknown-none-elf/release/os' \
    		-ex 'set arch riscv:rv64' \
    		-ex 'target remote localhost:1234'

clean: 
		@cargo clean
		@rm -f os/src/link_app.S

clippy:
		@cargo clippy

.PHONY: user kernel