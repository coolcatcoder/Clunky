use crate::menus;

pub const MAX_SUBSTEPS: u32 = 150;

pub fn start(render_storage: &mut crate::RenderStorage) -> UserStorage {
    render_storage.camera.scale = 0.12;

    render_storage.brightness = 2.5;

    let mut user_storage = UserStorage {
        wasd_held: (false, false, false, false),
        zoom_held: (false, false),
    };

    (menus::STARTING_MENU.get_data().start)(&mut user_storage, render_storage);

    user_storage
}

pub struct UserStorage {
    // This is for the user's stuff. The event loop should not touch this. On going debate on whether menu belongs in here or in render storage
    pub wasd_held: (bool, bool, bool, bool),
    pub zoom_held: (bool, bool),
}
