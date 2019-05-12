extern crate aloxide;
extern crate cc;

use std::path::Path;
use aloxide::Ruby;

fn rerun_if_changed(path: &Path) {
    println!("cargo:rerun-if-changed={}", path.display());
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
    // Ruby is already linked via `ruby-sys`
    let ruby = ruby();
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src").join("c");

    let ext_c = src.join("ruby_ext.c");
    let ext_h = src.join("ruby_ext.h");

    rerun_if_changed(&ext_c);
    rerun_if_changed(&ext_h);

    cc::Build::new()
        .file(&ext_c)
        .include(&ruby.header_dir().unwrap())
        .include(&ruby.arch_header_dir().unwrap())
        .warnings(false)
        .compile("ruby_ext");
}
