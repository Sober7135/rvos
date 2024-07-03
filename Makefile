TARGET := riscv64gc-unknown-none-elf
MODE := release
KERNEL_ELF := target/$(TARGET)/$(MODE)/os
KERNEL_BIN := $(KERNEL_ELF).bin
FS_IMG := target/$(TARGET)/$(MODE)/fs.img
DISASM_DIR := disasm
OBJCOPY := llvm-objcopy
GDB := gdb
vi = nvim --noplugin

fs-img: user
		@rm -rf $(FS_IMG)
		@cargo run --$(MODE) --package=fs-fuse --target=x86_64-unknown-linux-gnu -- -s user/src/bin -t target/$(TARGET)/$(MODE)

kernel: $(KERNEL_BIN)

$(KERNEL_BIN): $(KERNEL_ELF)
		@$(OBJCOPY) --strip-all $(KERNEL_ELF) -O binary -I elf64-little  $(KERNEL_BIN)	

$(KERNEL_ELF): os/ user
		@cd os && cargo build --$(MODE)
	
user: user/
		@cd user && cargo build --release --target=$(TARGET)
		
build: kernel user fs-img

run: build
		@qemu-system-riscv64 \
  	  	-machine virt \
  	  	-nographic \
  	  	-bios ./bootloader/rustsbi-qemu.bin \
  	  	-device loader,file=$(KERNEL_BIN),addr=0x80200000 \
				-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
        -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0

disasm: build
		@mkdir -p $(DISASM_DIR)
		@rust-objdump -S $(KERNEL_ELF) 1> $(DISASM_DIR)/os.asm 2> /dev/null

gdbserver: build
		@qemu-system-riscv64 \
    		-machine virt \
    		-nographic \
    		-bios ./bootloader/rustsbi-qemu.bin \
    		-device loader,file=$(KERNEL_BIN),addr=0x80200000 \
				-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
        -device virtio-blk-device,drive=x0,bus=virtio-mmio-bus.0 \
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

.PHONY: user kernel fs-img build 