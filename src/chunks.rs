use crate::{biomes, events, vertex_data};
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

pub fn generate_chunk() {
    
}
