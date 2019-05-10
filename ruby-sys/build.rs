extern crate aloxide;
extern crate bindgen;

use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use aloxide::{Ruby, Version};

fn generate_bindings(ruby: &Ruby, path: &Path) {
    let header = ruby.wrapper_header().expect("Generate header");
    let header_dir = ruby.header_dir().expect("Get header dir");
    let arch_header_dir = ruby.arch_header_dir()
        .expect("Get arch header dir");

    bindgen::builder()
        .header_contents("rosy.h", &header)
        .clang_args(&[
            format!("-I{}", header_dir),
            format!("-I{}", arch_header_dir),
        ])
        .generate()
        .expect("Generate bindings")
        .write_to_file(path)
        .expect(&format!("Could not write bindings to {:?}", path));
}

fn set_rustc_env(var: impl Display, val: impl Display) {
    println!("cargo:rustc-env={}={}", var, val);
}

fn write_ruby_version_const(version: &Version, out_dir: &Path) {
    let path = out_dir.join("ruby_version.rs");
    set_rustc_env("ROSY_RUBY_VERSION_CONST", path.display());

    let mut file = File::create(&path).expect("Create Ruby version file");
    write!(
        file,
        "/// The version of the static Ruby library being used: **{v}**.\n\
        pub const RUBY_VERSION: &str = \"{v}\";",
        v = version,
    ).expect("Write Ruby version");
}

#[cfg(feature = "download")]
fn ruby() -> Ruby {
    unimplemented!()
}

#[cfg(not(feature = "download"))]
fn ruby() -> Ruby {
    Ruby::current().unwrap()
}

fn main() {
    let ruby = ruby();
    ruby.link(true).unwrap();

    let out_dir = env::var_os("OUT_DIR").expect("Couldn't get 'OUT_DIR'");
    let out_dir = PathBuf::from(out_dir);

    write_ruby_version_const(ruby.version(), &out_dir);

    let bindings_name = format!("ruby-{}.rs", ruby.version());
    let bindings_path = out_dir.join(&bindings_name);
    set_rustc_env("ROSY_BINDINGS_PATH", bindings_path.display());

    if !bindings_path.exists() || env::var_os("ROSY_BINDGEN").is_some() {
        generate_bindings(&ruby, &bindings_path);
    }
}
