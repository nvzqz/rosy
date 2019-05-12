extern crate aloxide;
extern crate bindgen;

use std::{
    env,
    fmt::Display,
    path::PathBuf,
};
use aloxide::Ruby;

#[path = "build/bindings.rs"]
mod bindings;
#[path = "build/ruby.rs"]
mod ruby;

const LINK_STATIC: bool = cfg!(feature = "static");

fn set_rustc_env(var: impl Display, val: impl Display) {
    println!("cargo:rustc-env={}={}", var, val);
}

fn rerun_if_env_changed(var: impl Display) {
    println!("cargo:rerun-if-env-changed={}", var);
}

fn print_config(ruby: &Ruby) {
    rerun_if_env_changed("ROSY_PRINT_RUBY_CONFIG");
    if env::var_os("ROSY_PRINT_RUBY_CONFIG").is_some() {
        println!("{}", ruby.run("require 'pp'; pp RbConfig::CONFIG").unwrap());
    }
}

fn main() {
    let ruby = ruby::get();
    print_config(&ruby);
    ruby.link(LINK_STATIC).unwrap();

    let out_dir = env::var_os("OUT_DIR").expect("Couldn't get 'OUT_DIR'");
    let out_dir = PathBuf::from(out_dir);

    ruby::write_version_const(ruby.version(), &out_dir);
    bindings::write(&ruby, &out_dir);
}
