use crate::events;

pub struct PerkOrCurse<'a> {
    // TODO: Get a better name please.
    pub name: &'a str,
    pub description: &'a str,
    pub cost: u8,
    pub condition: fn(&mut events::UserStorage, &mut events::RenderStorage) -> bool,
    pub effect: fn(&mut events::UserStorage, &mut events::RenderStorage),
}

#[derive(Debug)]
pub enum PerkOrCursePointer {
    Duplicates(usize),
    NoDuplicates(usize),
}

#[derive(Debug)]
pub struct PerksAndCurses {
    pub cost: i16,
    pub one_time_perks_owned: Vec<usize>,
    pub one_time_curses_owned: Vec<usize>,
    pub offered_perks: Vec<PerkOrCursePointer>,
    pub offered_curses: Vec<PerkOrCursePointer>,
}

pub const COST_Y: f32 = 0.7;
pub const DESCRIPTION_Y: f32 = 0.0;

pub const PERKS: [PerkOrCurse; 3] = [
    PerkOrCurse {
        name: "test",
        description: "this is a test perk",
        cost: 5,
        condition: |_user_storage: &mut events::UserStorage,
                    _render_storage: &mut events::RenderStorage|
         -> bool { true },
        effect: |_user_storage: &mut events::UserStorage,
                 _render_storage: &mut events::RenderStorage| { println!("p0") },
    },
    PerkOrCurse {
        name: "Bug",
        description: "this should not happen",
        cost: 5,
        condition: |_user_storage: &mut events::UserStorage,
                    _render_storage: &mut events::RenderStorage|
         -> bool { false },
        effect: |_user_storage: &mut events::UserStorage,
                 _render_storage: &mut events::RenderStorage| { println!("p1") },
    },
    PerkOrCurse {
        name: "Extra Health",
        description: "Start the game with 5 extra health!",
        cost: 1,
        condition: |_user_storage, _render_storage| -> bool { true },
        effect: |user_storage, _render_storage| {
            user_storage.player.starting_statistics.health += 5;
        },
    },
];

pub const PERKS_NO_DUPLICATES: [PerkOrCurse; 1] = [PerkOrCurse {
    name: "no dupe",
    description: "this is a test perk without duplicates",
    cost: 3,
    condition: |_user_storage: &mut events::UserStorage,
                _render_storage: &mut events::RenderStorage|
     -> bool { true },
    effect: |_user_storage: &mut events::UserStorage,
             _render_storage: &mut events::RenderStorage| { println!("pnd0") },
}];

pub const CURSES: [PerkOrCurse; 2] = [
    PerkOrCurse {
        name: "test",
        description: "this is a test curse",
        cost: 5,
        condition: |_user_storage: &mut events::UserStorage,
                    _render_storage: &mut events::RenderStorage|
         -> bool { true },
        effect: |_user_storage: &mut events::UserStorage,
                 _render_storage: &mut events::RenderStorage| { println!("c0") },
    },
    PerkOrCurse {
        name: "Less Health",
        description: "Start the game with 5 less health!",
        cost: 1,
        condition: |user_storage, _render_storage| -> bool {
            user_storage.player.starting_statistics.health - 5 > 0
        },
        effect: |user_storage, _render_storage| {
            user_storage.player.starting_statistics.health -= 5;
        },
    },
];

pub const CURSES_NO_DUPLICATES: [PerkOrCurse; 1] = [PerkOrCurse {
    name: "no dupe",
    description: "this is a test curse without duplicates",
    cost: 3,
    condition: |_user_storage: &mut events::UserStorage,
                _render_storage: &mut events::RenderStorage|
     -> bool { true },
    effect: |_user_storage: &mut events::UserStorage,
             _render_storage: &mut events::RenderStorage| { println!("cnd0") },
}];
