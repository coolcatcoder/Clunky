use crate::{biomes, events, vertex_data};
use std::ops::Mul;
// This will probably have more comments than code, as everything here is confusing, and not easy to think about.

fn render_chunk(
    user_storage: &mut events::UserStorage,
    render_storage: &mut events::RenderStorage,
    chunk_position: (u32, u32),
) {
    let starting_index = events::index_from_position(chunk_position, events::CHUNK_GRID_WIDTH)
        * events::CHUNK_WIDTH_SQUARED as u32;
} // instead of rendering every frame, which is slow, you can just render the 9 chunks around the player, keeping an array of which chunks are rendered

// Potential revolution:
// each chunk stores a Vec containing their map objects. This will save a lot of memory. Might be slow though.

pub struct ChunkIdea1 {
    map_objects: Vec<biomes::MapObject>,
    generated: bool,
}

pub struct ChunkIdea2 {
    map_objects: Vec<(biomes::MapObject, Vec<vertex_data::MapVertex>, Vec<u32>)>, // map object, vertices, indices. Then do memcpy every frame
    generated: bool,
}

pub struct TestingHorriblePosition {
    x: u32,
    y: u32,
}

impl Mul for TestingHorriblePosition {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        TestingHorriblePosition { x: self.x * other.x, y: self.y * other.y }
    }
}

impl Mul for TestingHorriblePosition {
    type Output = Self;
    fn mul(self, other: u32) -> Self {
        TestingHorriblePosition { x: self.x * other, y: self.y * other }
    }
}

// pub struct Position<T> where T : Mul {
//     x: T,
//     y: T,
// }

// impl<T: Mul<Output = T>> Mul for Position<T> {
//     type Output = Self;
//     fn mul(self, other: Self) -> Self {
//         Position {
//             x: self.x * other.x,
//             y: self.y * other.y,
//         }
//     }
// }

// impl<T: Mul> Mul for Position<T> {
//     type Output = T;
//     fn mul(self, other: T) -> Self {
//         Position {
//             x: self.x * other,
//             y: self.y * other,
//         }
//     }
// }

#[derive(Clone)]
pub struct Chunk {
    pub map_objects: Vec<Vec<biomes::MapObject>>,
    pub generated: bool,
    pub starting_position: (u32,u32),
}

pub fn generate_chunk(chunk: &mut Chunk) {
    chunk.generated = true;
    //chunk.map_objects = vec![biomes::MapObject::None; events::CHUNK_WIDTH_SQUARED as usize];

    for map_object in &chunk.map_objects {
        //map_object = events::generate_position(position, detail, scale, offset, rng, biome_noise, percent_range, main_seed)
    }
}
