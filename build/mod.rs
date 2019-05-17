extern crate aloxide;

use std::{env, fmt::Display, path::PathBuf};

mod ruby;

const LINK_STATIC: bool = cfg!(feature = "static");

fn set_rustc_env(key: impl Display, val: impl Display) {
    println!("cargo:rustc-env={}={}", key, val);
}

fn rerun_if_env_changed(key: impl Display) {
    println!("cargo:rerun-if-env-changed={}", key);
}

fn main() {
    rerun_if_env_changed("RUBY");
    rerun_if_env_changed("ROSY_RUBY");
    rerun_if_env_changed("ROSY_RUBY_VERSION");
    rerun_if_env_changed("ROSY_PRINT_RUBY_CONFIG");

    #[cfg(feature = "rustc_version")]
    {
        use rustc_version::*;
        if version_meta().unwrap().channel == Channel::Nightly {
            println!("cargo:rustc-cfg=nightly");
        }
    }

    let out_dir = env::var_os("OUT_DIR").expect("Couldn't get 'OUT_DIR'");
    let out_dir = PathBuf::from(out_dir);

    if cfg!(feature = "_skip_linking") {
        let version = if cfg!(feature = "ruby_2_6") {
            "2.6"
        } else {
            "unknown"
        };
        ruby::write_version_const(&version, &out_dir);
    } else {
        let ruby = ruby::get();
        ruby::print_config(&ruby);
        ruby.link(LINK_STATIC).expect("Failed to link Ruby");
        ruby::write_version_const(ruby.version(), &out_dir);
    }
}
