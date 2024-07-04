#!/usr/bin/env python
#! ASSUME the current pwd is "./user"
import os
from sys import platform

base_address = 0x8040_0000
step = 0x2_0000
linker_file = "src/linker.ld"
target_dir = "../target/riscv64gc-unknown-none-elf/release/"
objcopy = "objcopy"
cargo_command = "cargo build --bin {} --release"
objcopy_bin = "rust-objcopy"

objcopy_command = "{} --strip-all {}/{} -O binary -I elf64-little {}/{}.bin"

apps = os.listdir("src/bin")
apps.sort()
apps_without_ext = []

for app_id, app in enumerate(apps):
    app_without_ext = app[: app.find(".")]
    apps_without_ext.append(app_without_ext)
    lines = []
    # read and replace
    with open(linker_file, "r") as f:
        for line in f.readlines():
            line = line.replace(
                hex(base_address + (app_id - 1) * step),
                hex(base_address + app_id * step),
            )
            lines.append(line)
    # override the file
    with open(linker_file, "w+") as f:
        f.writelines(lines)
    # run command
    command = cargo_command.format(app_without_ext)
    print(command)
    os.system(command)

# recover
lines = []
with open(linker_file, "r") as f:
    for line in f.readlines():
        line = line.replace(
            hex(base_address + (len(apps) - 1) * step),
            hex(base_address),
        )
        lines.append(line)
with open(linker_file, "w+") as f:
    f.writelines(lines)

# strip bin
for app_without_ext in apps_without_ext:
    command = objcopy_command.format(
        objcopy_bin, target_dir, app_without_ext, target_dir, app_without_ext
    )
    print(command)
    os.system(command)
