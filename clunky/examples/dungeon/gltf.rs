use clunky::buffer_contents::Colour3DInstance;
use gltf::Gltf;

const SCENES: &[&[u8]] = &[include_bytes!("./rooms/test.glb")];

pub struct Nameless<'a> {
    scenes: Scene<'a>,
}

pub struct Scene<'a> {
    cuboid_instances: &'a[Colour3DInstance]
}

pub fn load_scenes<'a>() -> Nameless<'a> {
    for scene in SCENES {
        let gltf = Gltf::from_slice(scene).unwrap();

        for node in gltf.nodes() {
            let node_name = node.name().unwrap();
            let transform_decomposed = node.transform().decomposed();
            let colour = {
                if let Some(temp_colour) = node.extras() {
                    let temp_colour = temp_colour.get();

                    //let temp_colour = temp_colour.get(10..temp_colour.len() - 1).unwrap();

                    println!("{}",temp_colour);

                    temp_colour
                } else {
                    "[1.0,1.0,1.0,1.0]"
                }
            };

            if node_name.contains("(instance: cuboid)") {

            }
            if node_name.contains("(physics: cuboid)") {
                
            }
            if node_name.contains("(physics: immovable cuboid)") {
                
            }
        }
    };
    todo!()
}

/*
fn dungeon_3d_scene_to_arrays(
    gltf: &Document,
    debug: &mut String,
    scene_prefix: &'static str,
    node_names_and_corresponding_infixes: &[(&'static str, &'static str)],
) -> String {
    let mut constant_arrays = vec![];

    for strings in node_names_and_corresponding_infixes {
        constant_arrays.push(format!(
            "pub const {}_{}_COLOUR_3D_INSTANCES: &[buffer_contents::Colour3DInstance] = &[",
            scene_prefix, strings.1
        ));
    }

    for node in gltf.nodes() {
        if let Some(node_name) = node.name() {
            let transform_decomposed = node.transform().decomposed();
            let colour = {
                if let Some(temp_colour) = node.extras() {
                    let temp_colour = temp_colour.get();

                    let temp_colour = temp_colour.get(10..temp_colour.len() - 1).unwrap();

                    temp_colour
                } else {
                    "[1.0,1.0,1.0,1.0]"
                }
            };

            for strings_index in 0..node_names_and_corresponding_infixes.len() {
                let strings = node_names_and_corresponding_infixes[strings_index];
                if node_name.contains(strings.0) {
                    constant_arrays[strings_index].push_str(&format!(
                        "
                    buffer_contents::Colour3DInstance::new(
                        {},
                        math::Matrix4::from_translation([{:?},-{:?},-{:?}]).multiply(math::Matrix4::from_scale([{:?},{:?},{:?}])),
                    ),
                    ",
                        colour,
                        transform_decomposed.0[0], transform_decomposed.0[1], transform_decomposed.0[2], transform_decomposed.2[0], transform_decomposed.2[1], transform_decomposed.2[2],
                    ));
                }
            }
        }
    }

    let mut combined_arrays = String::from("");

    for mut constant_array in constant_arrays {
        constant_array.push_str("];");

        combined_arrays.push_str(&constant_array);
    }

    debug.push_str(&format!("{}\n", combined_arrays));
    combined_arrays
}
*/