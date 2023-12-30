use std::env;
use std::fs;
use std::path::Path;

mod meshes_to_load;

// blah

fn main() {
    let mut constants = String::from("");
    let mut debug = String::from("");

    for mesh_loader in meshes_to_load::MESH_LOADERS {
        let (gltf, buffers, _) = gltf::import(mesh_loader.path).unwrap();

        let mesh = gltf.meshes().nth(0).unwrap();
        let primitive = mesh.primitives().nth(0).unwrap();
        constants.push_str(&format!(
            "{}\n",
            (mesh_loader.primitive_and_buffers_to_arrays)(primitive, buffers, &mut debug)
        ));
    }

    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("gltf_meshes.rs");
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
    println!("cargo:rerun-if-changed=meshes_to_load.rs");
}
