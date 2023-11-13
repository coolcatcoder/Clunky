use crate::events;
use crate::vertex_data;

// should be multiple types of text, like screentext which is screen relative, and map text which is in world space

// perhaps these texts should be stored in a vec somewhere, for easy addition and removal, and due to them all storing their own vertices, it can be a simple memcopy for each one

pub const TEXT_SPRITE_SIZE: (f32, f32) = (1.0 / 30.0, 1.0 / 5.0);

pub struct ScreenText {
    vertices: Vec<vertex_data::VertexData>,
    indices: Vec<u32>,
}

impl ScreenText {
    pub fn new(
        mut position: (f32, f32),
        character_size: (f32, f32),
        letter_spacing: f32,
        text: &str,
    ) -> ScreenText {
        assert!(text.is_ascii());
        let characters = text.as_bytes();

        let mut vertices = vec![
            vertex_data::VertexData {
                position: [0.0, 0.0],
                uv: [0.0, 0.0],
            };
            4 * characters.len()
        ];
        let mut indices = vec![0u32; 6 * characters.len()];

        let mut vertex_start = 0;
        let mut index_start = 0;

        for character_index in 0..characters.len() {
            let character = characters[character_index] as char;

            let uv = get_uv_for_character(character);

            vertices[vertex_start] = vertex_data::VertexData {
                // top right
                position: [
                    position.0 + character_size.0 * 0.5,
                    position.1 + character_size.1 * 0.5,
                ],
                uv: [uv.0 + TEXT_SPRITE_SIZE.0, uv.1 + TEXT_SPRITE_SIZE.1],
            };

            vertices[vertex_start + 1] = vertex_data::VertexData {
                // bottom right
                position: [
                    position.0 + character_size.0 * 0.5,
                    position.1 - character_size.1 * 0.5,
                ],
                uv: [uv.0 + TEXT_SPRITE_SIZE.0, uv.1],
            };

            vertices[vertex_start + 2] = vertex_data::VertexData {
                // top left
                position: [
                    position.0 - character_size.0 * 0.5,
                    position.1 + character_size.1 * 0.5,
                ],
                uv: [uv.0, uv.1 + TEXT_SPRITE_SIZE.1],
            };

            vertices[vertex_start + 3] = vertex_data::VertexData {
                // bottom left
                position: [
                    position.0 - character_size.0 * 0.5,
                    position.1 - character_size.1 * 0.5,
                ],
                uv: [uv.0, uv.1],
            };

            indices[index_start] = vertex_start as u32;
            indices[index_start + 1] = vertex_start as u32 + 1;
            indices[index_start + 2] = vertex_start as u32 + 2;

            indices[index_start + 3] = vertex_start as u32 + 1;
            indices[index_start + 4] = vertex_start as u32 + 3;
            indices[index_start + 5] = vertex_start as u32 + 2;

            vertex_start += 4;
            index_start += 6;

            match characters.get(character_index + 1) {
                None => {}
                Some(next_character) => {
                    position.0 +=
                        get_distance_between_characters((character, *next_character as char))
                            * letter_spacing;
                }
            }
        }

        let screen_text = ScreenText { vertices, indices };

        screen_text
    }

    fn set_text() {
        todo!();
    }
}

const fn get_distance_between_characters(characters: (char, char)) -> f32 {
    match characters {
        ('N', 'o') => 1.3,
        ('T', 'i') => 1.2,
        ('i', 't') => 0.5,
        ('t', 'l') => 0.7,
        ('l', 'e') => 0.5,
        ('P', 'r') => 1.2,
        ('r', 'e') => 0.7,
        ('E', 'n') => 1.2,
        ('n', 't') => 1.1,
        ('t', 'e') => 0.7,
        ('r', '!') => 0.8,
        _ => 1.0,
    }
}

fn get_uv_for_character(character: char) -> (f32, f32) {
    match character {
        '0' => (0.0, 0.0),
        '1' => (TEXT_SPRITE_SIZE.0 * 1.0, 0.0),
        '2' => (TEXT_SPRITE_SIZE.0 * 2.0, 0.0),
        '3' => (TEXT_SPRITE_SIZE.0 * 3.0, 0.0),
        '4' => (TEXT_SPRITE_SIZE.0 * 4.0, 0.0),
        '5' => (TEXT_SPRITE_SIZE.0 * 5.0, 0.0),
        '6' => (TEXT_SPRITE_SIZE.0 * 6.0, 0.0),
        '7' => (TEXT_SPRITE_SIZE.0 * 7.0, 0.0),
        '8' => (TEXT_SPRITE_SIZE.0 * 8.0, 0.0),
        '9' => (TEXT_SPRITE_SIZE.0 * 9.0, 0.0),

        'A' => (TEXT_SPRITE_SIZE.0 * 0.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'B' => (TEXT_SPRITE_SIZE.0 * 1.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'C' => (TEXT_SPRITE_SIZE.0 * 2.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'D' => (TEXT_SPRITE_SIZE.0 * 3.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'E' => (TEXT_SPRITE_SIZE.0 * 4.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'F' => (TEXT_SPRITE_SIZE.0 * 5.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'G' => (TEXT_SPRITE_SIZE.0 * 6.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'H' => (TEXT_SPRITE_SIZE.0 * 7.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'I' => (TEXT_SPRITE_SIZE.0 * 8.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'J' => (TEXT_SPRITE_SIZE.0 * 9.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'K' => (TEXT_SPRITE_SIZE.0 * 10.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'L' => (TEXT_SPRITE_SIZE.0 * 11.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'M' => (TEXT_SPRITE_SIZE.0 * 12.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'N' => (TEXT_SPRITE_SIZE.0 * 13.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'O' => (TEXT_SPRITE_SIZE.0 * 14.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'P' => (TEXT_SPRITE_SIZE.0 * 15.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'Q' => (TEXT_SPRITE_SIZE.0 * 16.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'R' => (TEXT_SPRITE_SIZE.0 * 17.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'S' => (TEXT_SPRITE_SIZE.0 * 18.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'T' => (TEXT_SPRITE_SIZE.0 * 19.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'U' => (TEXT_SPRITE_SIZE.0 * 20.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'V' => (TEXT_SPRITE_SIZE.0 * 21.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'W' => (TEXT_SPRITE_SIZE.0 * 22.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'X' => (TEXT_SPRITE_SIZE.0 * 23.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'Y' => (TEXT_SPRITE_SIZE.0 * 24.0, TEXT_SPRITE_SIZE.1 * 1.0),
        'Z' => (TEXT_SPRITE_SIZE.0 * 25.0, TEXT_SPRITE_SIZE.1 * 1.0),

        'a' => (TEXT_SPRITE_SIZE.0 * 0.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'b' => (TEXT_SPRITE_SIZE.0 * 1.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'c' => (TEXT_SPRITE_SIZE.0 * 2.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'd' => (TEXT_SPRITE_SIZE.0 * 3.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'e' => (TEXT_SPRITE_SIZE.0 * 4.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'f' => (TEXT_SPRITE_SIZE.0 * 5.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'g' => (TEXT_SPRITE_SIZE.0 * 6.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'h' => (TEXT_SPRITE_SIZE.0 * 7.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'i' => (TEXT_SPRITE_SIZE.0 * 8.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'j' => (TEXT_SPRITE_SIZE.0 * 9.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'k' => (TEXT_SPRITE_SIZE.0 * 10.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'l' => (TEXT_SPRITE_SIZE.0 * 11.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'm' => (TEXT_SPRITE_SIZE.0 * 12.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'n' => (TEXT_SPRITE_SIZE.0 * 13.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'o' => (TEXT_SPRITE_SIZE.0 * 14.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'p' => (TEXT_SPRITE_SIZE.0 * 15.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'q' => (TEXT_SPRITE_SIZE.0 * 16.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'r' => (TEXT_SPRITE_SIZE.0 * 17.0, TEXT_SPRITE_SIZE.1 * 2.0),
        's' => (TEXT_SPRITE_SIZE.0 * 18.0, TEXT_SPRITE_SIZE.1 * 2.0),
        't' => (TEXT_SPRITE_SIZE.0 * 19.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'u' => (TEXT_SPRITE_SIZE.0 * 20.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'v' => (TEXT_SPRITE_SIZE.0 * 21.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'w' => (TEXT_SPRITE_SIZE.0 * 22.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'x' => (TEXT_SPRITE_SIZE.0 * 23.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'y' => (TEXT_SPRITE_SIZE.0 * 24.0, TEXT_SPRITE_SIZE.1 * 2.0),
        'z' => (TEXT_SPRITE_SIZE.0 * 25.0, TEXT_SPRITE_SIZE.1 * 2.0),

        ' ' => (TEXT_SPRITE_SIZE.0 * 29.0, TEXT_SPRITE_SIZE.1 * 2.0),
        ':' => (TEXT_SPRITE_SIZE.0 * 12.0, TEXT_SPRITE_SIZE.1 * 0.0),
        '-' => (TEXT_SPRITE_SIZE.0 * 11.0, TEXT_SPRITE_SIZE.1 * 0.0),
        '_' => (TEXT_SPRITE_SIZE.0 * 13.0, TEXT_SPRITE_SIZE.1 * 0.0),
        '.' => (TEXT_SPRITE_SIZE.0 * 10.0, TEXT_SPRITE_SIZE.1 * 0.0),
        '!' => (TEXT_SPRITE_SIZE.0 * 15.0, TEXT_SPRITE_SIZE.1 * 0.0),
        '%' => (TEXT_SPRITE_SIZE.0 * 17.0, TEXT_SPRITE_SIZE.1 * 0.0),
        '(' => (TEXT_SPRITE_SIZE.0 * 18.0, TEXT_SPRITE_SIZE.1 * 0.0),
        ')' => (TEXT_SPRITE_SIZE.0 * 19.0, TEXT_SPRITE_SIZE.1 * 0.0),
        ',' => (TEXT_SPRITE_SIZE.0 * 20.0, TEXT_SPRITE_SIZE.1 * 0.0),

        '@' => (TEXT_SPRITE_SIZE.0 * 0.0, TEXT_SPRITE_SIZE.1 * 3.0),
        _ => (TEXT_SPRITE_SIZE.0 * 14.0, 0.0),
    }
}

pub fn render_screen_texts(
    render_storage: &mut events::RenderStorage,
    screen_texts: &Vec<ScreenText>,
) {
    for screen_text in screen_texts {
        render_storage.vertices_text[render_storage.vertex_count_text as usize
            ..render_storage.vertex_count_text as usize + screen_text.vertices.len()]
            .copy_from_slice(screen_text.vertices.as_slice());

        let mut updated_indices = screen_text.indices.clone();
        updated_indices
            .iter_mut()
            .for_each(|x| *x += render_storage.vertex_count_text);
        render_storage.indices_text[render_storage.index_count_text as usize
            ..render_storage.index_count_text as usize + screen_text.indices.len()]
            .copy_from_slice(updated_indices.as_slice());

        render_storage.vertex_count_text += screen_text.vertices.len() as u32;
        render_storage.index_count_text += screen_text.indices.len() as u32;
    }
}
