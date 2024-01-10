use std::env;
use std::fs;
use std::path::Path;

mod gltf_to_load;

fn main() {
    let mut constants = String::from("");
    let mut debug = String::from("");

    for loader in gltf_to_load::LOADERS {
        let (gltf, buffers, _) = gltf::import(loader.path).unwrap();

        constants.push_str(&format!(
            "{}\n",
            (loader.gltf_and_buffers_to_constants)(gltf, buffers, &mut debug)
        ));
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("loaded_from_gltf.rs");
    fs::write(
        &dest_path,
        format!(
            "
        pub const DEBUG: &str = \"{0}\";
        {1}
        ",
            debug, constants
        ),
    )
    .unwrap();
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/meshes");
    println!("cargo:rerun-if-changed=gltf_to_load.rs");
}
