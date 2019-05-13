use std::{
    env,
    fs::File,
    ffi::OsStr,
    io::Write,
    path::{Path, PathBuf},
};
use aloxide::{Ruby, Version};

// An external tool that manages the Ruby installation
enum Manager {
    // Download and build Ruby ourselves
    Download,
    // https://www.github.com/rbenv/rbenv
    Rbenv,
    // https://github.com/rvm/rvm
    Rvm,
}

impl Manager {
    fn from_var(var: &OsStr) -> Option<Self> {
        if var == "download" {
            Some(Manager::Download)
        } else if var == "rbenv" {
            Some(Manager::Rbenv)
        } else if var == "rvm" {
            Some(Manager::Rvm)
        } else {
            None
        }
    }

    fn ruby(self, v: &Version) -> Ruby {
        match self {
            Manager::Rbenv => {
                Ruby::from_rbenv(v).expect("Could not get Ruby from 'rbenv'")
            },
            Manager::Rvm => {
                Ruby::from_rvm(v).expect("Could not get Ruby from 'rvm'")
            },
            Manager::Download => download(),
        }
    }
}

enum Driver {
    Manager(Manager),
    // A path to a Ruby executable
    Path(PathBuf),
}

impl Driver {
    fn get() -> Option<Self> {
        let ruby = env::var_os("ROSY_RUBY")?;
        if ruby.is_empty() {
            None
        } else if let Some(manager) = Manager::from_var(&ruby) {
            Some(Driver::Manager(manager))
        } else {
            Some(Driver::Path(ruby.into()))
        }
    }

    fn ruby(self) -> Ruby {
        match self {
            Driver::Manager(manager) => {
                manager.ruby(&version().expect("'ROSY_RUBY_VERSION' not set"))
            },
            Driver::Path(ruby) => {
                Ruby::from_bin(&ruby)
                    .expect(&format!(
                        "Could not get Ruby from '{}'",
                        ruby.display(),
                    ))
            }
        }
    }
}

fn version() -> Option<Version> {
    super::rerun_if_env_changed("ROSY_RUBY_VERSION");
    Some(env::var_os("ROSY_RUBY_VERSION")?
        .to_str()
        .expect("'ROSY_RUBY_VERSION' is not UTF-8")
        .parse()
        .expect("Could not parse 'ROSY_RUBY_VERSION'"))
}

pub fn write_version_const(version: &Version, out_dir: &Path) {
    let path = out_dir.join("ruby_version.rs");
    super::set_rustc_env("ROSY_RUBY_VERSION_CONST", path.display());

    let mut file = File::create(&path).expect("Create Ruby version file");
    write!(
        file,
        "/// The version of the Ruby library API being used: **{v}**.\n\
        ///\n\
        /// Note that this may differ from the version of the actual Ruby\n\
        /// library being linked to when using dynamic linking.\n\
        pub const RUBY_VERSION: &str = \"{v}\";",
        v = version,
    ).expect("Could not write `RUBY_VERSION` const");
}

#[cfg(feature = "download")]
fn download() -> Ruby {
    unimplemented!("Can't download yet")
}

#[cfg(not(feature = "download"))]
fn download() -> Ruby {
    panic!("Enable 'download' feature in 'Cargo.toml' to download Ruby");
}

pub fn print_config(ruby: &Ruby) {
    super::rerun_if_env_changed("ROSY_PRINT_RUBY_CONFIG");
    if env::var_os("ROSY_PRINT_RUBY_CONFIG").is_some() {
        println!("{}", ruby.run("require 'pp'; pp RbConfig::CONFIG").unwrap());
    }
}

pub fn get() -> Ruby {
    if let Some(driver) = Driver::get() {
        driver.ruby()
    } else {
        Ruby::current().expect("Could not get system Ruby in 'PATH'")
    }
}
