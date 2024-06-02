use clunky::{
    lost_code::is_pressed,
    math::add_3d,
    physics::physics_3d::{aabb::AabbCentredOrigin, verlet::Particle},
    shaders::instanced_simple_lit_colour_3d::Camera,
};
use rand::{thread_rng, Rng};
use winit::event::{KeyboardInput, VirtualKeyCode};

use crate::{
    body::{Body, Creature as CreatureBody},
    movement_in_direction, renderer, wasd_to_movement, BodyIndex, CreatureIndex, Engine,
    FIXED_DELTA_TIME,
};

pub enum CreatureType {
    Burgle(Burgle),
}

impl CreatureType {
    pub fn attempt_capture(
        &mut self,
        spreader: &mut CreatureType,
        body_index: BodyIndex,
        bodies: &mut Vec<Body>,
    ) -> bool {
        match self {
            CreatureType::Burgle(burgle) => true,
        }
    }

    pub fn accelerate(&mut self, acceleration: [f32; 3], bodies: &mut Vec<Body>) {
        match self {
            CreatureType::Burgle(burgle) => {
                let Body::Creature(body) = &mut bodies[burgle.body.0] else {
                    unreachable!()
                };

                body.particle.accelerate(acceleration);
            }
        }
    }

    pub fn on_physics_fixed_update_before_physics_tick_when_focused(
        &mut self,
        bodies: &mut Vec<Body>,
    ) {
        match self {
            CreatureType::Burgle(burgle) => {
                burgle.on_physics_fixed_update_before_physics_tick_when_focused(bodies);
            }
        }
    }

    pub fn on_keyboard_input(&mut self, input: KeyboardInput) {
        match self {
            CreatureType::Burgle(burgle) => match input.virtual_keycode.unwrap() {
                VirtualKeyCode::W => burgle.wasd_held[0] = is_pressed(input.state),
                VirtualKeyCode::A => burgle.wasd_held[1] = is_pressed(input.state),
                VirtualKeyCode::S => burgle.wasd_held[2] = is_pressed(input.state),
                VirtualKeyCode::D => burgle.wasd_held[3] = is_pressed(input.state),
                VirtualKeyCode::Space => burgle.jump_held = is_pressed(input.state),

                VirtualKeyCode::F => {
                    if is_pressed(input.state) {
                        burgle.sprinting = !burgle.sprinting;
                    }
                }

                VirtualKeyCode::K => {
                    if is_pressed(input.state) {
                        println!("{:?}", burgle)
                    }
                }
                _ => (),
            },
        }
    }

    pub fn on_mouse_movement(&mut self, movement: [f32; 2]) {
        match self {
            CreatureType::Burgle(burgle) => {
                burgle.rotation[0] -= movement[0];
                burgle.rotation[1] -= movement[1];
            }
        }
    }

    pub fn update_camera(&self, camera: &mut Camera, bodies: &[Body]) {
        match self {
            CreatureType::Burgle(burgle) => {
                let Body::Creature(body) = &bodies[burgle.body.0] else {
                    unreachable!()
                };

                camera.position = add_3d(body.particle.position, [0.0, -1.0, 0.0]);
                camera.rotation = burgle.rotation;
            }
        }
    }

    pub fn get_bodies_within_capture_range(&self, bodies: &[Body]) -> Vec<BodyIndex> {
        match self {
            CreatureType::Burgle(burgle) => {
                let Body::TriggerImmovableCuboid { collisions, .. } = &bodies[burgle.body.0 + 1]
                else {
                    unreachable!()
                };

                collisions.iter().map(|index| BodyIndex(*index)).collect()
            }
        }
    }
}

#[derive(Debug)]
pub struct Burgle {
    pub body: BodyIndex,

    wasd_held: [bool; 4],
    jump_held: bool,
    sprinting: bool,

    // I know euler angles are bad, but it is fine for now...
    rotation: [f32; 3],

    walk_speed: f32,
    run_speed: f32,
    jump_acceleration: f32,
    // TODO: add flight like wyvern, and energy
}

impl Burgle {
    pub fn new(
        engine: &mut Engine,
        position: [f32; 3],
        half_size: [f32; 3],
        trigger_half_size: [f32; 3],
        colour: [f32; 4],

        index: CreatureIndex,
    ) -> Burgle {
        let body_index = engine.physics.bodies.len();

        engine.physics.bodies.push(Body::Creature(CreatureBody {
            particle: Particle::from_position(position),
            half_size,

            mass: 1.0,
            dampening: [0.9, 1.0, 0.9],

            grounded: false,

            owner: index,
        }));
        engine.physics.bodies.push(Body::TriggerImmovableCuboid {
            aabb: AabbCentredOrigin {
                position,
                half_size: trigger_half_size,
            },
            collisions: vec![],
        });

        let mut renderer = renderer(engine);
        renderer.add_cuboid_colour_from_body_index(body_index, colour);

        let mut rng = thread_rng();

        let walk_speed = rng.gen_range(25.0..50.0);

        Burgle {
            body: BodyIndex(body_index),

            wasd_held: [false; 4],
            jump_held: false,
            sprinting: false,

            rotation: [0.0; 3],

            walk_speed,
            run_speed: walk_speed + rng.gen_range(5.0..100.0),
            jump_acceleration: rng.gen_range(-1000.0..-300.0),
        }
    }

    fn on_physics_fixed_update_before_physics_tick_when_focused(&mut self, bodies: &mut Vec<Body>) {
        let Body::Creature(body) = &mut bodies[self.body.0] else {
            unreachable!()
        };
        //let velocity = body.particle.calculate_velocity(FIXED_DELTA_TIME as f32);
        //println!("velocity {:?}", velocity);

        let mut motion = wasd_to_movement(self.wasd_held);

        if self.sprinting {
            motion[0] *= self.run_speed;
            motion[1] *= self.run_speed;
        } else {
            motion[0] *= self.walk_speed;
            motion[1] *= self.walk_speed;
        }

        motion = movement_in_direction(motion, [self.rotation[0], self.rotation[1]]);

        if self.jump_held {
            if body.grounded {
                body.particle.accelerate([0.0, self.jump_acceleration, 0.0]);
            }
        }

        body.particle.accelerate([motion[0], 0.0, motion[1]]);
    }
}
