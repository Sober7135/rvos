MODE = release

OS_DIR = os
OS_SOURCE_DIR = $(OS_DIR)/src
TARGET_DIR = target/riscv64gc-unknown-none-elf/$(MODE)
vi = nvim --noplugin


os: $(OS_SOURCE_DIR)/*
		cd $(OS_DIR) && cargo build --$(MODE)
		rust-objcopy --strip-all $(TARGET_DIR)/os -O binary  $(TARGET_DIR)/os.bin	
	
objdump: os
			rust-objdump -h $(TARGET_DIR)/os  | $(vi) -

clean: 
		cargo clean

qemu: os
		qemu-system-riscv64 \
  	  	-machine virt \
  	  	-nographic \
  	  	-bios ./bootloader/rustsbi-qemu.bin \
  	  	-device loader,file=$(TARGET_DIR)/os.bin,addr=0x80200000

debug: os
		qemu-system-riscv64 \
    		-machine virt \
    		-nographic \
    		-bios ./bootloader/rustsbi-qemu.bin \
    		-device loader,file=$(TARGET_DIR)/os.bin,addr=0x80200000 \
    		-s -S
gdb:
		riscv64-unknown-elf-gdb \
    		-ex 'file target/riscv64gc-unknown-none-elf/release/os' \
    		-ex 'set arch riscv:rv64' \
    		-ex 'target remote localhost:1234'