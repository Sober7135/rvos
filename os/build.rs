use std::{
    fs::{self, File},
    io::{Result, Write},
};

fn main() {
    let profile = std::env::var("PROFILE").unwrap();
    let target_path = TARGET_PATH_WITHOUT_MODE.to_string() + profile.as_str();
    println!("cargo:rerun-if-changed=../usr/src/");
    println!("cargo:rerun-if-changed={}", target_path);
    insert_app(target_path).unwrap();
}

static TARGET_PATH_WITHOUT_MODE: &str = "./target/riscv64gc-unknown-none-elf/";

fn insert_app(target_path: String) -> Result<()> {
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
    .global _num_apps
_num_apps:
    .quad {}"#,
        apps.len()
    )?;

    for i in 0..apps.len() {
        let start = format!("app_{}_start", i);
        writeln!(link_file, r#"    .quad {}"#, start)?;
    }

    writeln!(link_file, r#"    .quad app_{}_end"#, apps.len() - 1)?;
    writeln!(
        link_file,
        r#"    
    .global _app_names
_app_names:"#
    )?;

    for app in apps.iter() {
        writeln!(link_file, r#"    .string "{}""#, app)?;
    }

    for (i, app) in apps.iter().enumerate() {
        let start = format!("app_{}_start", i);
        let end = format!("app_{}_end", i);
        writeln!(
            link_file,
            r#"
    .section .data # maybe unnecessary
    .global {}
    .global {}
    .align 3
{}:
    .incbin "{}/{}"
{}:"#,
            start, end, start, target_path, app, end
        )?
    }
    Ok(())
}
