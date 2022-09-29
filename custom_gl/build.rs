extern crate gl_generator;

use gl_generator::{Api, Fallbacks, Profile, Registry};
use std::env;
use std::fs::File;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("gl_bindings.rs");
    let mut file = File::create(&dest).unwrap();

    Registry::new(Api::Gl, (4, 5), Profile::Core, Fallbacks::All, ["GL_ARB_bindless_texture"])
        .write_bindings(gl_generator::GlobalGenerator, &mut file)
        .unwrap();
}
