use clunky::{
    buffer_contents::Colour3DInstance,
    math::{mul_3d_by_1d, Matrix4},
    physics::physics_3d::{
        aabb::AabbCentredOrigin,
        bodies::{CommonBody, ImmovableCuboid},
    },
};
use gltf::Gltf;

const GLTF_SCENES: &[&[u8]] = &[
    include_bytes!("./rooms/test.glb"),
    include_bytes!("./rooms/experiment.glb"),
];

pub struct Nameless {
    pub scenes: Vec<Scene>,
}

pub struct Scene {
    pub cuboid_instances: Vec<Colour3DInstance>,
    pub bodies: Vec<CommonBody<f32>>,
}

pub fn load_scenes() -> Nameless {
    let mut nameless = Nameless {
        scenes: Vec::with_capacity(GLTF_SCENES.len()),
    };

    for gltf_scene in GLTF_SCENES {
        let mut scene = Scene {
            cuboid_instances: Vec::with_capacity(10),
            bodies: Vec::with_capacity(15),
        };

        let gltf = Gltf::from_slice(gltf_scene).unwrap();

        for node in gltf.nodes() {
            let node_name = node.name().unwrap();

            println!("{}", node_name);

            let Some(_properties) = node.extras() else {
                continue;
            };
            if node_name.contains("single room") {
                continue;
            }

            let transform_decomposed = {
                let mut temp_transform_decomposed = node.transform().decomposed();

                //println!("{:?}",temp_transform_decomposed);

                //temp_transform_decomposed.0[0] = -temp_transform_decomposed.0[0];
                temp_transform_decomposed.0[1] = -temp_transform_decomposed.0[1];
                temp_transform_decomposed.0[2] = -temp_transform_decomposed.0[2];

                temp_transform_decomposed.1[1] = -temp_transform_decomposed.1[1];

                //temp_transform_decomposed.2 = neg_3d(temp_transform_decomposed.2);

                temp_transform_decomposed
            };

            let _colour = {
                if let Some(temp_colour) = node.extras() {
                    let temp_colour = temp_colour.get();

                    //let temp_colour = temp_colour.get(10..temp_colour.len() - 1).unwrap();

                    println!("{}", temp_colour);

                    temp_colour
                } else {
                    println!("no");
                    "[1.0,1.0,1.0,1.0]"
                }
            };

            //cgmath::Matrix4::from_axis_angle(axis, angle)

            if node_name.contains("(instance: cuboid)") {
                scene.cuboid_instances.push(Colour3DInstance::new(
                    [1.0; 4],
                    Matrix4::from_translation(transform_decomposed.0)
                        * Matrix4::from_quaternion(transform_decomposed.1)
                        * Matrix4::from_scale(transform_decomposed.2),
                ));
            }

            if node_name.contains("(physics: cuboid)") {}
            if node_name.contains("(physics: immovable cuboid)") {
                scene
                    .bodies
                    .push(CommonBody::ImmovableCuboid(ImmovableCuboid {
                        aabb: AabbCentredOrigin {
                            position: transform_decomposed.0,
                            half_size: mul_3d_by_1d(transform_decomposed.2, 0.5),
                        },
                    }));
            }
        }
        nameless.scenes.push(scene);
    }

    nameless
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
