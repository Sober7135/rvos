MODE = release

OS_DIR = os
USER_DIR = user
OS_SOURCE_DIR = $(OS_DIR)/src
USER_SOURCE_DIR = $(USER_DIR)/src
TARGET_DIR = target/riscv64gc-unknown-none-elf/$(MODE)
vi = nvim --noplugin

define strip_bin
		rust-objcopy --strip-all $(TARGET_DIR)/$(1) -O binary  $(TARGET_DIR)/$(1).bin	
endef

os: $(OS_SOURCE_DIR)/* user
		cd $(OS_DIR) && cargo build --$(MODE)
		# rust-objcopy --strip-all $(TARGET_DIR)/os -O binary  $(TARGET_DIR)/os.bin	
		$(call strip_bin,os)
	
objdump: os
			rust-objdump -h $(TARGET_DIR)/os  | $(vi) -
			
user:  $(USER_DIR)/*
		cd $(USER_DIR) && cargo build --$(MODE)
		$(call strip_bin,00hello_world)
		$(call strip_bin,01store_fault)
		$(call strip_bin,02power)
		$(call strip_bin,03priv_inst)
		$(call strip_bin,04priv_csr)

qemu: os user
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

gdb: os user
		riscv64-unknown-elf-gdb \
    		-ex 'file target/riscv64gc-unknown-none-elf/release/os' \
    		-ex 'set arch riscv:rv64' \
    		-ex 'target remote localhost:1234'

clean: 
		cargo clean
		rm -f os/src/link_app.S

