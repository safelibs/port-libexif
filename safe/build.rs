use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

const SONAME: &str = "libexif.so.12";
const DEFAULT_GETTEXT_PACKAGE: &str = "libexif-12";
const DEFAULT_LOCALEDIR: &str = "/usr/share/locale";

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../original/libexif/libexif.sym");
    println!("cargo:rerun-if-changed=../original/libexif/exif-tag.c");
    println!("cargo:rerun-if-changed=include/libexif/exif-tag.h");
    println!("cargo:rerun-if-changed=include/libexif/exif-ifd.h");
    println!("cargo:rerun-if-changed=include/libexif/exif-data-type.h");
    println!("cargo:rerun-if-changed=tests/support/config.h");
    println!("cargo:rerun-if-changed=tests/support/libexif/i18n.h");
    println!("cargo:rerun-if-changed=cshim/exif-log-shim.c");
    println!("cargo:rerun-if-changed=include/libexif/exif-log.h");
    println!("cargo:rerun-if-env-changed=LIBEXIF_GETTEXT_PACKAGE");
    println!("cargo:rerun-if-env-changed=LIBEXIF_LOCALEDIR");
    let symbols_path = Path::new("../original/libexif/libexif.sym");
    let symbols = parse_symbol_list(&fs::read_to_string(symbols_path)?)?;

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let version_script = out_dir.join("libexif.map");
    fs::write(&version_script, render_version_script(&symbols))?;
    fs::write(out_dir.join("tag_table_data.rs"), render_tag_table_data()?)?;

    println!(
        "cargo:rustc-env=LIBEXIF_GETTEXT_PACKAGE={}",
        env::var("LIBEXIF_GETTEXT_PACKAGE").unwrap_or_else(|_| DEFAULT_GETTEXT_PACKAGE.to_owned())
    );
    println!(
        "cargo:rustc-env=LIBEXIF_LOCALEDIR={}",
        env::var("LIBEXIF_LOCALEDIR").unwrap_or_else(|_| DEFAULT_LOCALEDIR.to_owned())
    );

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

fn render_tag_table_data() -> Result<String, Box<dyn Error>> {
    let preprocessed = preprocess_tag_source()?;
    let tag_values = parse_tag_values(&preprocessed)?;
    let entries = parse_tag_table_entries(&preprocessed, &tag_values)?;

    let mut output = String::new();
    output.push_str(&format!(
        "pub(crate) static TAG_TABLE: [TagEntry; {}] = [\n",
        entries.len()
    ));

    for entry in entries {
        output.push_str("    TagEntry {\n");
        output.push_str(&format!("        tag: {},\n", entry.tag));
        output.push_str(&format!("        name: {},\n", render_message(&entry.name)));
        output.push_str(&format!(
            "        title: {},\n",
            render_message(&entry.title)
        ));
        output.push_str(&format!(
            "        description: {},\n",
            render_message(&entry.description)
        ));
        output.push_str(&format!(
            "        support_levels: {},\n",
            entry.support_levels
        ));
        output.push_str("    },\n");
    }

    output.push_str("];\n");
    Ok(output)
}

fn preprocess_tag_source() -> Result<String, Box<dyn Error>> {
    let compiler = env::var("CC").unwrap_or_else(|_| String::from("cc"));
    let output = Command::new(compiler)
        .arg("-E")
        .arg("-P")
        .arg("-I")
        .arg("tests/support")
        .arg("-I")
        .arg("include")
        .arg("-I")
        .arg("../original")
        .arg("../original/libexif/exif-tag.c")
        .output()?;

    if !output.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "failed to preprocess exif-tag.c\nstdout:\n{}\nstderr:\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        )
        .into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

fn parse_tag_values(preprocessed: &str) -> Result<HashMap<String, u32>, io::Error> {
    let enum_end = preprocessed.find("} ExifTag;").ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "failed to locate ExifTag enum end",
        )
    })?;
    let enum_start = preprocessed[..enum_end]
        .rfind("typedef enum {")
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "failed to locate ExifTag enum")
        })?
        + "typedef enum {".len();
    let enum_body = &preprocessed[enum_start..enum_end];

    let mut values = HashMap::new();
    let mut next_value = 0u32;
    for item in enum_body.split(',') {
        let item = item.trim();
        if item.is_empty() {
            continue;
        }

        let (name, value) = item
            .split_once('=')
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "malformed ExifTag enum"))?;
        let value = parse_u32(value.trim())?;
        values.insert(name.trim().to_owned(), value);
        next_value = value.wrapping_add(1);
    }

    let _ = next_value;
    Ok(values)
}

fn parse_tag_table_entries(
    preprocessed: &str,
    tag_values: &HashMap<String, u32>,
) -> Result<Vec<ParsedTagEntry>, io::Error> {
    let table_start = preprocessed.find("} ExifTagTable[] = {").ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "failed to locate ExifTagTable")
    })? + "} ExifTagTable[] = {".len();
    let body = &preprocessed[table_start..];
    let raw_entries = split_entries(body)?;

    raw_entries
        .iter()
        .map(|entry| parse_tag_table_entry(entry, tag_values))
        .collect()
}

fn split_entries(body: &str) -> Result<Vec<String>, io::Error> {
    let mut entries = Vec::new();
    let mut index = 0usize;
    let bytes = body.as_bytes();

    while index < bytes.len() {
        while index < bytes.len() && matches!(bytes[index], b' ' | b'\t' | b'\r' | b'\n' | b',') {
            index += 1;
        }
        if index >= bytes.len() {
            break;
        }
        if bytes[index] == b'}' {
            break;
        }
        if bytes[index] != b'{' {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unexpected tag-table token near byte {index}"),
            ));
        }

        let end = find_matching_brace(body, index)?;
        entries.push(body[index..=end].to_owned());
        index = end + 1;
    }

    Ok(entries)
}

fn find_matching_brace(text: &str, start: usize) -> Result<usize, io::Error> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut index = start;
    let mut in_string = false;
    let mut escape = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escape {
                escape = false;
            } else if byte == b'\\' {
                escape = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else if byte == b'"' {
            in_string = true;
        } else if byte == b'{' {
            depth += 1;
        } else if byte == b'}' {
            depth -= 1;
            if depth == 0 {
                return Ok(index);
            }
        }
        index += 1;
    }

    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "unterminated brace expression in tag table",
    ))
}

fn parse_tag_table_entry(
    entry: &str,
    tag_values: &HashMap<String, u32>,
) -> Result<ParsedTagEntry, io::Error> {
    let inner = entry
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "malformed tag-table entry"))?;
    let fields = split_top_level(inner)?;
    if fields.len() != 5 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected 5 tag-table fields, found {}", fields.len()),
        ));
    }

    Ok(ParsedTagEntry {
        tag: render_tag_token(fields[0].trim(), tag_values)?,
        name: parse_c_string_expr(fields[1].trim())?,
        title: parse_c_string_expr(fields[2].trim())?,
        description: parse_c_string_expr(fields[3].trim())?,
        support_levels: normalize_support_levels(fields[4].trim()),
    })
}

fn split_top_level(text: &str) -> Result<Vec<&str>, io::Error> {
    let bytes = text.as_bytes();
    let mut fields = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    let mut index = 0usize;
    let mut in_string = false;
    let mut escape = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if in_string {
            if escape {
                escape = false;
            } else if byte == b'\\' {
                escape = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else {
            match byte {
                b'"' => in_string = true,
                b'{' | b'(' | b'[' => depth += 1,
                b'}' | b')' | b']' => depth = depth.saturating_sub(1),
                b',' if depth == 0 => {
                    fields.push(text[start..index].trim());
                    start = index + 1;
                }
                _ => {}
            }
        }
        index += 1;
    }

    fields.push(text[start..].trim());
    Ok(fields)
}

fn render_tag_token(token: &str, tag_values: &HashMap<String, u32>) -> Result<String, io::Error> {
    if token.starts_with("0x") || token.chars().all(|ch| ch.is_ascii_digit()) {
        return Ok(token.to_owned());
    }

    tag_values
        .get(token)
        .map(|value| format!("0x{value:04x}"))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown ExifTag token `{token}` in tag table"),
            )
        })
}

fn parse_c_string_expr(token: &str) -> Result<Option<Vec<u8>>, io::Error> {
    let normalized = token.replace(' ', "");
    if normalized == "((void*)0)" || normalized == "NULL" {
        return Ok(None);
    }

    let mut bytes = Vec::new();
    let source = token.as_bytes();
    let mut index = 0usize;

    while index < source.len() {
        if source[index] != b'"' {
            index += 1;
            continue;
        }
        index += 1;
        let start = index;
        let mut escape = false;
        while index < source.len() {
            let byte = source[index];
            if escape {
                escape = false;
            } else if byte == b'\\' {
                escape = true;
            } else if byte == b'"' {
                break;
            }
            index += 1;
        }
        if index >= source.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unterminated C string in tag table",
            ));
        }
        bytes.extend(unescape_c_string(&token[start..index])?);
        index += 1;
    }

    if bytes.is_empty() && token != "\"\"" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse C string expression `{token}`"),
        ));
    }

    Ok(Some(bytes))
}

fn unescape_c_string(source: &str) -> Result<Vec<u8>, io::Error> {
    let mut output = Vec::new();
    let bytes = source.as_bytes();
    let mut index = 0usize;

    while index < bytes.len() {
        let byte = bytes[index];
        if byte != b'\\' {
            output.push(byte);
            index += 1;
            continue;
        }

        index += 1;
        if index >= bytes.len() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "dangling C string escape",
            ));
        }

        let escaped = bytes[index];
        match escaped {
            b'\\' | b'"' | b'\'' | b'?' => {
                output.push(escaped);
                index += 1;
            }
            b'a' => {
                output.push(0x07);
                index += 1;
            }
            b'b' => {
                output.push(0x08);
                index += 1;
            }
            b'f' => {
                output.push(0x0c);
                index += 1;
            }
            b'n' => {
                output.push(b'\n');
                index += 1;
            }
            b'r' => {
                output.push(b'\r');
                index += 1;
            }
            b't' => {
                output.push(b'\t');
                index += 1;
            }
            b'v' => {
                output.push(0x0b);
                index += 1;
            }
            b'x' => {
                index += 1;
                let hex_start = index;
                while index < bytes.len() && bytes[index].is_ascii_hexdigit() {
                    index += 1;
                }
                if hex_start == index {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "empty hexadecimal escape in C string",
                    ));
                }
                let value = u8::from_str_radix(&source[hex_start..index], 16)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                output.push(value);
            }
            b'0'..=b'7' => {
                let oct_start = index;
                index += 1;
                while index < bytes.len()
                    && index - oct_start < 3
                    && matches!(bytes[index], b'0'..=b'7')
                {
                    index += 1;
                }
                let value = u8::from_str_radix(&source[oct_start..index], 8)
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                output.push(value);
            }
            _ => {
                output.push(escaped);
                index += 1;
            }
        }
    }

    Ok(output)
}

fn normalize_support_levels(token: &str) -> String {
    token.replace('{', "[").replace('}', "]")
}

fn render_message(bytes: &Option<Vec<u8>>) -> String {
    match bytes {
        Some(bytes) => format!("Some(crate::i18n::message({}))", render_byte_string(bytes)),
        None => String::from("None"),
    }
}

fn render_byte_string(bytes: &[u8]) -> String {
    let mut literal = String::from("b\"");
    for &byte in bytes {
        match byte {
            b'\\' => literal.push_str("\\\\"),
            b'"' => literal.push_str("\\\""),
            b'\n' => literal.push_str("\\n"),
            b'\r' => literal.push_str("\\r"),
            b'\t' => literal.push_str("\\t"),
            0x20..=0x7e => literal.push(byte as char),
            _ => literal.push_str(&format!("\\x{byte:02x}")),
        }
    }
    literal.push_str("\\0\"");
    literal
}

fn parse_u32(token: &str) -> Result<u32, io::Error> {
    if let Some(hex) = token.strip_prefix("0x") {
        u32::from_str_radix(hex, 16)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    } else {
        token
            .parse::<u32>()
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }
}

struct ParsedTagEntry {
    tag: String,
    name: Option<Vec<u8>>,
    title: Option<Vec<u8>>,
    description: Option<Vec<u8>>,
    support_levels: String,
}
