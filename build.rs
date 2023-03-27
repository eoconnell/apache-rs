extern crate bindgen;
extern crate regex;

use bindgen::callbacks::IntKind;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

#[derive(Debug)]
struct ParseCallbacks;

impl bindgen::callbacks::ParseCallbacks for ParseCallbacks {
    fn int_macro(&self, name: &str, _value: i64) -> Option<IntKind> {
        match name {
            "MODULE_MAGIC_NUMBER_MAJOR" => Some(IntKind::I32),
            "MODULE_MAGIC_NUMBER_MINOR" => Some(IntKind::I32),
            "MODULE_MAGIC_COOKIE" => Some(IntKind::U64),
            _ => None,
        }
    }
    // Reformat comments so rustdoc does not interpret them as doc-tests.
    // See: https://github.com/rust-lang/rust-bindgen/issues/1313
    fn process_comment(&self, comment: &str) -> Option<String> {
        Some(
            regex::Regex::new(r"\n     +")
                .unwrap()
                .replace_all(comment, "\n    ")
                .to_string(),
        )
    }
}

fn apxs_interpolate(value: &str, vars: &HashMap<&str, &str>) -> String {
    // apxs interpolates both ${name} and $(name)
    if let Some(open) = value.find("${").or(value.find("$(")) {
        let keylen = value[open + 2..].find(['}', ')']).unwrap();
        let key = &value[open + 2..open + 2 + keylen];
        let replacement = apxs_interpolate(vars.get(key).unwrap(), vars);
        let replaced = format!(
            "{}{}{}",
            &value[..open],
            replacement,
            &value[open + 2 + keylen + 1..]
        );
        apxs_interpolate(replaced.as_str(), vars)
    } else {
        value.into()
    }
}

fn main() {
    // apxs -q <name> interpolation does not work with paths containing '@' from homebrew.
    // see: https://bz.apache.org/bugzilla/show_bug.cgi?id=61944
    let apxs_output = String::from_utf8_lossy(
        &Command::new("apxs")
            .arg("-q")
            .output()
            .expect("failed to run apxs")
            .stdout,
    )
    .to_string();
    let apxs_vars: HashMap<&str, &str> = apxs_output
        .lines()
        .map(|l| l.split_once('=').unwrap())
        .collect();
    let apxs_query = |name: &str| apxs_interpolate(&apxs_vars.get(name).unwrap(), &apxs_vars);

    let exclude = [
        "mod_ssl_openssl.h", // avoid generating bindings for openssl
        "mod_xml2enc.h",     // avoid generating bindings for libxml2
    ];

    // List the relevant header files to build bindings for.
    let mut header_files: Vec<String> = ["includedir", "APR_INCLUDEDIR", "APU_INCLUDEDIR"]
        .iter()
        .flat_map(|name| fs::read_dir(apxs_query(&name)).unwrap())
        .map(|r| r.unwrap().file_name().to_str().unwrap().to_string())
        .filter(|name| !exclude.contains(&name.as_str()))
        .collect();

    // Sort the list of header files.
    header_files.sort();
    let header_contents: String = header_files
        .iter()
        .map(|name| format!("#include \"{}\"\n", name))
        .collect();

    // The bindgen::Builder
    // Derived from https://rust-lang.github.io/rust-bindgen/tutorial-3.html
    let bindings = bindgen::Builder::default()
        .header_contents("wrapper.h", header_contents.as_str())
        // mod_xml2enc.h:46:42: error: unknown type name 'xmlCharEncoding'
        //.clang_arg("--include=libxml/encoding.h")
        .clang_args(apxs_query("EXTRA_INCLUDES").split_ascii_whitespace())
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .parse_callbacks(Box::new(ParseCallbacks))
        // Pretty print the bindings.
        .rustfmt_bindings(true)
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
