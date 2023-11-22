#!/usr/bin/env python
#! ASSUME the current pwd is "./user"
import os

base_address = 0x8040_0000
linker_file = "src/linker.ld"
target_dir = "../target/riscv64gc-unknown-none-elf/release/"
objcopy = "objcopy"
cargo_command = "cargo build --bin {} --release"
objcopy_command = "objcopy --strip-all {}/{} -O binary -I elf64-little {}/{}.bin"

apps = os.listdir("src/bin")
apps.sort()
apps_without_ext = []

for app_id, app in enumerate(apps):
    app_without_ext = app[: app.find(".")]
    apps_without_ext.append(app_without_ext)
    lines = []
    # read and replace
    # run command
    command = cargo_command.format(app_without_ext)
    print(command)
    os.system(command)

# strip bin
for app_without_ext in apps_without_ext:
    command = objcopy_command.format(
        target_dir, app_without_ext, target_dir, app_without_ext
    )
    print(command)
    os.system(command)
