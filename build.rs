use std::env;
use std::fs;
use std::path::Path;

mod meshes_to_load;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("gltf_meshes.rs");
    fs::write(
        &dest_path,
        "pub const TEST: u8 = 1;
        "
    ).unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/meshes");
    println!("cargo:rerun-if-changed=meshes_to_load.rs");
}