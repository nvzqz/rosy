extern crate aloxide;

use std::{env, fmt::Display, path::PathBuf};

#[path = "build/ruby.rs"]
mod ruby;

const LINK_STATIC: bool = cfg!(feature = "static");

fn set_rustc_env(key: impl Display, val: impl Display) {
    println!("cargo:rustc-env={}={}", key, val);
}

fn rerun_if_env_changed(key: impl Display) {
    println!("cargo:rerun-if-env-changed={}", key);
}

fn main() {
    // Ruby is already linked via `ruby-sys`
    let ruby = ruby::get();
    ruby::print_config(&ruby);
    ruby.link(LINK_STATIC).unwrap();

    let out_dir = env::var_os("OUT_DIR").expect("Couldn't get 'OUT_DIR'");
    let out_dir = PathBuf::from(out_dir);

    ruby::write_version_const(ruby.version(), &out_dir);
}
