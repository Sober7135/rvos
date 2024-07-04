use std::{
    fs::{self, File},
    io::{Result, Write},
};

fn main() {
    println!("cargo:rerun-if-changed=../usr/src/");
    println!("cargo:rerun-if-changed={}", TARGET_PATH);
    insert_app().unwrap();
}

static TARGET_PATH: &str = "./target/riscv64gc-unknown-none-elf/release";

fn insert_app() -> Result<()> {
    let mut link_file = File::create("src/link_app.S").unwrap();
    let mut apps: Vec<_> = fs::read_dir("../user/src/bin")
        .unwrap()
        .map(|dir_entry| {
            let mut name_with_ext = dir_entry.unwrap().file_name().into_string().unwrap();
            name_with_ext.drain(name_with_ext.find(".rs").unwrap()..name_with_ext.len());
            name_with_ext
        })
        .collect();
    apps.sort();

    writeln!(
        link_file,
        r#"
    .align 3
    .section .data
    .global _num_app
_num_app:
    .quad {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        let start = format!("app_{}_start", i);
        writeln!(link_file, r#"    .quad {}"#, start)?;
    }

    writeln!(link_file, r#"    .quad app_{}_end"#, apps.len() - 1)?;

    for (i, app) in apps.iter().enumerate() {
        let start = format!("app_{}_start", i);
        let end = format!("app_{}_end", i);
        writeln!(
            link_file,
            r#"
    .section .data # maybe unnecessary
    .global {}
    .global {}
{}:
    .incbin "{}/{}.bin"
{}:"#,
            start, end, start, TARGET_PATH, app, end
        )?
    }
    Ok(())
}
