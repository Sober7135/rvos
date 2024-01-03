OS_DIR = os
USER_DIR = user
OS_SOURCE_DIR = $(OS_DIR)/src
USER_SOURCE_DIR = $(USER_DIR)/src
RELEASE_TARGET_DIR = target/riscv64gc-unknown-none-elf/release
DEBUG_TARGET_DIR = target/riscv64gc-unknown-none-elf/debug
vi = nvim --noplugin
OBJCOPY = objcopy
UNAME = $(shell uname)

ifeq ($(UNAME), Darwin)
		OBJCOPY = llvm-objcopy
endif


os: $(OS_SOURCE_DIR)/* user
		cd $(OS_DIR) && cargo build --release
		$(OBJCOPY) --strip-all $(RELEASE_TARGET_DIR)/os -O binary -I elf64-little  $(RELEASE_TARGET_DIR)/os.bin
	
objdump: os
	objdump -h $(RELEASE_TARGET_DIR)/os  | $(vi) -
			
user:  $(USER_DIR)/*
		cd $(USER_DIR) && cargo build --release
		
build: os user

run: build
		qemu-system-riscv64 \
  	  	-machine virt \
  	  	-nographic \
  	  	-bios ./bootloader/rustsbi-qemu.bin \
  	  	-device loader,file=$(RELEASE_TARGET_DIR)/os.bin,addr=0x80200000

run-debug: build-debug
		qemu-system-riscv64 \
  	  	-machine virt \
  	  	-nographic \
  	  	-bios ./bootloader/rustsbi-qemu.bin \
  	  	-device loader,file=$(DEBUG_TARGET_DIR)/os.bin,addr=0x80200000


os-debug: $(OS_SOURCE_DIR)/* user-debug
		cd $(OS_DIR) && cargo build
		$(OBJCOPY) --strip-all $(DEBUG_TARGET_DIR)/os -O binary -I elf64-little  $(DEBUG_TARGET_DIR)/os.bin
	
objdump-debug: os-debug
	objdump -h $(DEBUG_TARGET_DIR)/os  | $(vi) -
			
user-debug:  $(USER_DIR)/*
		cd $(USER_DIR) && cargo build
		
build-debug: os-debug user-debug


gdbserver-debug: build-debug
		qemu-system-riscv64 \
    		-machine virt \
    		-nographic \
    		-bios ./bootloader/rustsbi-qemu.bin \
    		-device loader,file=$(DEBUG_TARGET_DIR)/os.bin,addr=0x80200000 \
    		-s -S

gdbclient-debug: build-debug
		riscv64-linux-gnu-gdb \
    		-ex 'file target/riscv64gc-unknown-none-elf/debug/os' \
    		-ex 'set arch riscv:rv64' \
    		-ex 'target remote localhost:1234'

gdbserver: build
		qemu-system-riscv64 \
    		-machine virt \
    		-nographic \
    		-bios ./bootloader/rustsbi-qemu.bin \
    		-device loader,file=$(RELEASE_TARGET_DIR)/os.bin,addr=0x80200000 \
    		-s -S

gdbclient: build
		riscv64-linux-gnu-gdb \
    		-ex 'file target/riscv64gc-unknown-none-elf/release/os' \
    		-ex 'set arch riscv:rv64' \
    		-ex 'target remote localhost:1234'


clean: 
		cargo clean
		rm -f os/src/link_app.S


.PHONY: clean gdbclient gdbclient-debug gdbserver gdbserver-debug build build-debug run run-debug