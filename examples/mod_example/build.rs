fn main() {
    // Permit undefined symbols to internal symbols in the `httpd` binary when building on Mac.
    // See: https://github.com/rust-lang/rust/pull/66204
    println!("cargo:rustc-cdylib-link-arg=-undefined");
    println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
    // `apxs -c` sets rpath to the module directory.
    println!(
        "cargo:rustc-cdylib-link-arg=-Wl,-rpath,{}",
        String::from_utf8_lossy(
            &std::process::Command::new("apxs")
                .arg("-q")
                .arg("libexecdir")
                .output()
                .expect("failed to run apxs")
                .stdout,
        )
        .trim()
    );
}
