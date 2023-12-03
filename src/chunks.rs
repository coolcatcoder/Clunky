use crate::{biomes, events};
use std::{ops::Add, ops::Div, ops::Mul, ops::Rem};
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

#[derive(Clone, Copy, Debug)]
pub struct Position2D<T>(pub T, pub T);

impl<T: Mul<T, Output = T>> Mul for Position2D<T> {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        Position2D(self.0 * other.0, self.1 * other.1)
    }
}

impl<T: Mul<T, Output = T> + Copy> Mul<T> for Position2D<T> {
    type Output = Self;
    fn mul(self, other: T) -> Self {
        Position2D(self.0 * other, self.1 * other)
    }
}

#[derive(Clone)]
pub struct Chunk {
    pub map_objects: Vec<Vec<biomes::MapObject>>,
    pub generated: bool,
    pub starting_position: Position2D<u32>,
}

pub fn index_from_position<T>(position: Position2D<T>, width: T) -> T
where
    T: Mul<T, Output = T> + Add<T, Output = T>,
{
    position.1 * width + position.0
}

pub fn position_from_index<T>(index: T, width: T) -> Position2D<T>
where
    T: Rem<T, Output = T> + Div<T, Output = T> + Copy,
{
    Position2D(index % width, index / width)
}

pub fn generate_chunk(chunk: &mut Chunk) {
    chunk.generated = true;
    //chunk.map_objects = vec![biomes::MapObject::None; events::CHUNK_WIDTH_SQUARED as usize];

    for map_object in &chunk.map_objects {
        //map_object = events::generate_position(position, detail, scale, offset, rng, biome_noise, percent_range, main_seed)
    }
}
