use clunky::shaders::instanced_text_sdf::Instance as LetterInstance;
use fontdue::{Font, FontSettings};

use crate::CreaturesManager;

const BOOKMAN_OLD_STYLE_TTF: &[u8] = include_bytes!("fonts/bookman old style.ttf");

// Current plan: the main window should have the main menu, settings, pause menu everything like that.
// Once you boot a reality, it closes, and you have a selection menu instead, but by pausing, or clicking a settings icon, you can reopen the menu.

pub fn blah() {
    let bookman_old_style = Font::from_bytes(BOOKMAN_OLD_STYLE_TTF, FontSettings {
        ..Default::default()
    }).unwrap();

    let blah = bookman_old_style.horizontal_kern('a', 'w', 1.0).unwrap();

    println!("kern test: {blah}");
}

pub struct TextManager {
}

impl TextManager {
    pub fn new() -> Self {
        TextManager {

        }
    }

    pub fn on_selection_menu_resize(extent: [f32; 2], letters: &mut Vec<LetterInstance>, creatures_manager: &CreaturesManager) {
        letters.clear();

        let aspect_ratio = extent[0] / extent[1];

        let mut y = -1.0 / aspect_ratio;

        for creature_index in &creatures_manager.captured_creatures {
            y += 0.3 * aspect_ratio;

            letters.push(LetterInstance::new(
                [0.0, 0.0],
                [1.0, 0.0, 1.0, 1.0],
                0.01,
                0.2,
                glam::Affine2::from_translation([0.0, y].into())
                    * glam::Affine2::from_scale([0.25, 0.25].into()),
            ))
        }

        letters.shrink_to_fit();
    }
}