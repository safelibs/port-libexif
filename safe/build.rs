use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const SONAME: &str = "libexif.so.12";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../original/libexif/libexif.sym");
    println!("cargo:rerun-if-changed=cshim/exif-log-shim.c");
    println!("cargo:rerun-if-changed=include/libexif/exif-log.h");

    let symbols_path = Path::new("../original/libexif/libexif.sym");
    let symbols = parse_symbol_list(&fs::read_to_string(symbols_path)?)?;

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let version_script = out_dir.join("libexif.map");
    fs::write(&version_script, render_version_script(&symbols))?;

    cc::Build::new()
        .file("cshim/exif-log-shim.c")
        .include("include")
        .compile("exif-log-shim");

    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
        println!("cargo:rustc-cdylib-link-arg=-Wl,-soname,{SONAME}");
        println!(
            "cargo:rustc-cdylib-link-arg=-Wl,--version-script={}",
            version_script.display()
        );
    }

    Ok(())
}

fn parse_symbol_list(contents: &str) -> Result<Vec<String>, io::Error> {
    let mut symbols = Vec::new();

    for line in contents.lines() {
        let symbol = line.trim();
        if symbol.is_empty() {
            continue;
        }
        if symbols.iter().any(|existing| existing == symbol) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("duplicate exported symbol `{symbol}`"),
            ));
        }
        symbols.push(symbol.to_owned());
    }

    Ok(symbols)
}

fn render_version_script(symbols: &[String]) -> String {
    let mut script = String::from("{\n  global:\n");
    for symbol in symbols {
        script.push_str("    ");
        script.push_str(symbol);
        script.push_str(";\n");
    }
    script.push_str("  local:\n    *;\n};\n");
    script
}
