use std::{
    env,
    fs::File,
    ffi::OsStr,
    fmt::Display,
    io::Write,
    path::{Path, PathBuf},
};
use aloxide::{Ruby, Version};

struct Manager {
    client: ManagerClient,
    version: Version,
}

impl Manager {
    fn from_var(var: &OsStr) -> Option<Self> {
        let var = var.to_str()?;

        let (client_str, version_str) = match var.find(':') {
            Some(index) => {
                let (start, end) = var.split_at(index);
                (start, Some(&end[1..]))
            },
            None => (var, None),
        };

        let client = match client_str {
            "download" => ManagerClient::Download,
            "rbenv"    => ManagerClient::Rbenv,
            "rvm"      => ManagerClient::Rvm,
            _          => panic!("Unknown Ruby client '{}'", client_str),
        };

        let version = match version_str {
            Some(version) => {
                version.parse::<Version>()
                    .expect("Could not parse Ruby version")
            },
            None => version().expect("'ROSY_RUBY_VERSION' not set"),
        };

        Some(Manager { client, version })
    }

    fn ruby(&self) -> Ruby {
        self.client.ruby(&self.version)
    }
}

// An external tool that manages the Ruby installation
enum ManagerClient {
    // Download and build Ruby ourselves
    Download,
    // https://www.github.com/rbenv/rbenv
    Rbenv,
    // https://github.com/rvm/rvm
    Rvm,
}

impl ManagerClient {
    fn ruby(&self, v: &Version) -> Ruby {
        match self {
            ManagerClient::Rbenv => {
                Ruby::from_rbenv(v).expect("Could not get Ruby from 'rbenv'")
            },
            ManagerClient::Rvm => {
                Ruby::from_rvm(v).expect("Could not get Ruby from 'rvm'")
            },
            ManagerClient::Download => download(v),
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

    fn ruby(&self) -> Ruby {
        match self {
            Driver::Manager(manager) => manager.ruby(),
            Driver::Path(ruby) => {
                let error = format!(
                    "Could not get Ruby from '{}'",
                    ruby.display(),
                );
                Ruby::from_bin(ruby).expect(&error)
            }
        }
    }
}

fn version() -> Option<Version> {
    Some(env::var_os("ROSY_RUBY_VERSION")?
        .to_str()
        .expect("'ROSY_RUBY_VERSION' is not UTF-8")
        .parse()
        .expect("Could not parse 'ROSY_RUBY_VERSION'"))
}

pub fn write_version_const(version: &dyn Display, out_dir: &Path) {
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
fn download(version: &Version) -> Ruby {
    unimplemented!("Can't download Ruby {} yet", version)
}

#[cfg(not(feature = "download"))]
fn download(version: &Version) -> Ruby {
    panic!(
        "Enable 'download' feature in 'Cargo.toml' to download Ruby {}",
        version
    );
}

pub fn print_config(ruby: &Ruby) {
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
