use crate::menus;

pub const MAX_SUBSTEPS: u32 = 150;

pub fn start(render_storage: &mut crate::RenderStorage) -> UserStorage {
    let mut user_storage = UserStorage {
        wasd_held: (false, false, false, false),
        zoom_held: (false, false),
        camera_3d_position: [0.0, 2.0, 0.0],
        camera_3d_rotation: [0.0, 0.0, 90.0], // Set z to 90.0 to have the best time ever!
        camera_3d_scale: [0.5, 2.3, 1.0], // Fun: [0.5, 2.3, 1.0]
        sensitivity: 0.25,
        sprinting: true,
    };

    (menus::STARTING_MENU.get_data().start)(&mut user_storage, render_storage);

    user_storage
}

pub struct UserStorage {
    // This is for the user's stuff. The event loop should not touch this. On going debate on whether menu belongs in here or in render storage
    pub wasd_held: (bool, bool, bool, bool),
    pub zoom_held: (bool, bool),

    pub camera_3d_position: [f32; 3],
    pub camera_3d_rotation: [f32; 3],
    pub camera_3d_scale: [f32; 3],

    pub sensitivity: f32,

    pub sprinting: bool,
}
