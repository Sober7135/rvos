MODE = release

OS_DIR = os
USER_DIR = user
OS_SOURCE_DIR = $(OS_DIR)/src
USER_SOURCE_DIR = $(USER_DIR)/src
TARGET_DIR = target/riscv64gc-unknown-none-elf/$(MODE)
vi = nvim --noplugin

os: $(OS_SOURCE_DIR)/* user
		cd $(OS_DIR) && cargo build --$(MODE)
		objcopy --strip-all $(TARGET_DIR)/os -O binary -I elf64-little  $(TARGET_DIR)/os.bin	
	
objdump: os
	objdump -h $(TARGET_DIR)/os  | $(vi) -
			
user:  $(USER_DIR)/*
		cd $(USER_DIR) && ./build.py 
		
build: os user

run: build
		qemu-system-riscv64 \
  	  	-machine virt \
  	  	-nographic \
  	  	-bios ./bootloader/rustsbi-qemu.bin \
  	  	-device loader,file=$(TARGET_DIR)/os.bin,addr=0x80200000

gdbserver: build
		qemu-system-riscv64 \
    		-machine virt \
    		-nographic \
    		-bios ./bootloader/rustsbi-qemu.bin \
    		-device loader,file=$(TARGET_DIR)/os.bin,addr=0x80200000 \
    		-s -S

gdbclient: build
		riscv64-elf-gdb \
    		-ex 'file target/riscv64gc-unknown-none-elf/release/os' \
    		-ex 'set arch riscv:rv64' \
    		-ex 'target remote localhost:1234'

clean: 
		cargo clean
		rm -f os/src/link_app.S

