use vulkano::buffer::subbuffer::BufferWriteGuard;

use crate::vertex_data;

pub const STARTING_VERTICES: [vertex_data::VertexData; 7] = [
    vertex_data::VertexData {
        position: [-1.0, 1.0],
    },
    vertex_data::VertexData {
        position: [1.0, 1.0],
    },
    vertex_data::VertexData {
        position: [1.0, -1.0],
    },

    vertex_data::VertexData {
        position: [-10.0, 1.5],
    },
    vertex_data::VertexData {
        position: [-1.0, 1.5],
    },
    vertex_data::VertexData {
        position: [-10.0, 1.0],
    },
    vertex_data::VertexData {
        position: [-1.0, 1.0],
    },
];

pub const STARTING_INDICES: [u16; 9] = [
    0,1,2,

    3,4,5,
    4,5,6,
    ];

pub fn start() -> Storage {
    Storage { direction: -1.0 }
}

pub fn update(
    storage: &mut Storage,
    mut vertices: BufferWriteGuard<'_, [vertex_data::VertexData]>,
    mut indices: BufferWriteGuard<'_, [u16]>,
    delta_time: f32,
    average_fps: f32,
) {
    //println!("delta time: {}", delta_time);
    //println!("average fps: {}", average_fps);
    vertices[0].position[1] += storage.direction * delta_time;

    if vertices[0].position[1] < -1.0 {
        storage.direction = 1.0;
    } else if vertices[0].position[1] > 1.0 {
        storage.direction = -1.0;
    }
}

pub fn late_update(storage: &mut Storage, delta_time: f32, average_fps: f32) {}

pub struct Storage {
    direction: f32,
}
