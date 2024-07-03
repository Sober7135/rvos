fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let target_path = TARGET_PATH_WITHOUT_MODE.to_string() + profile.as_str();
    println!("cargo:rerun-if-changed=../usr/src/");
    println!("cargo:rerun-if-changed={}", target_path);
}

static TARGET_PATH_WITHOUT_MODE: &str = "./target/riscv64gc-unknown-none-elf/";
