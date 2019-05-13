use std::{
    env,
    path::Path,
};
use aloxide::Ruby;

fn should_rustfmt() -> bool {
    env::var_os("TRAVIS") == Some("true")
}

pub fn write(ruby: &Ruby, out_dir: &Path) {
    let path = out_dir.join(format!("ruby-{}.rs", ruby.version()));
    super::set_rustc_env("ROSY_BINDINGS_PATH", path.display());

    super::rerun_if_env_changed("ROSY_BINDGEN");
    if path.exists() && env::var_os("ROSY_BINDGEN").is_none() {
        return;
    }

    let header = ruby.wrapper_header().expect("Generate header");
    let header_dir = ruby.header_dir().expect("Get header dir");
    let arch_header_dir = ruby.arch_header_dir().expect("Get arch header dir");

    bindgen::builder()
        .header_contents("rosy.h", &header)
        .clang_args(&[
            format!("-I{}", header_dir),
            format!("-I{}", arch_header_dir),
        ])
        .default_enum_style(bindgen::EnumVariation::ModuleConsts)
        .rustified_enum("ruby_value_type")
        .rustfmt_bindings(should_rustfmt())
        .generate()
        .expect("Generate bindings")
        .write_to_file(&path)
        .expect(&format!("Could not write bindings to {:?}", path));
}
