use std::{
    num::NonZeroU8,
    sync::mpsc::{channel, Sender},
};

use crate::{math, physics::PhysicsSimulation};

use super::bodies::Body;

use rayon::prelude::*;

extern crate test;

pub struct Config<T, B>
where
    T: math::Float,
    B: Body<T>,
{
    pub gravity: [T; 3],
    pub dampening: [T; 3],
    pub grid_size: [usize; 3],
    pub grid_origin: [T; 3],
    pub cell_size: [usize; 3],
    pub outside_of_grid_bounds_behaviour: OutsideOfGridBoundsBehaviour<T, B>,
    pub bodies: Vec<B>,
}

impl<B> Default for Config<f32, B>
where
    B: Body<f32>,
{
    fn default() -> Self {
        Self {
            gravity: [0.0, 50.0, 0.0],
            dampening: [0.8, 1.0, 0.8],
            grid_size: [10; 3],
            grid_origin: [0.0; 3],
            cell_size: [5; 3],
            outside_of_grid_bounds_behaviour: OutsideOfGridBoundsBehaviour::ContinueUpdating,
            bodies: vec![],
        }
    }
}

// TODO: add Steps struct to solver.
pub struct Steps {
    pub particle_updates: NonZeroU8,
    pub penetration_removals: NonZeroU8,
}

/// A generic solver capable of handling most basic physics simulations.
pub struct CpuSolver<T, B>
where
    T: math::Float,
    B: Body<T>,
{
    pub gravity: [T; 3],
    pub dampening: [T; 3], // Where 1.0 is no dampening. Perhaps displacement_kept is a better name?
    pub bodies: Vec<B>,
    pub grid_size: [usize; 3], // This is in cell size units. This should probably be clarified.
    pub cell_size: [usize; 3], // TODO: asap work out how the usize vs isize nonsense will work, as we want this to work for negatives. Perhaps we can plus some sort of offset for the particle?
    pub grid_origin: [T; 3], // Remember that the origin is the bottom left corner of the grid, I think.
    pub grid: Vec<Vec<usize>>,
    pub outside_of_grid_bounds_behaviour: OutsideOfGridBoundsBehaviour<T, B>,
}

impl<T: math::Float, B: Body<T>> PhysicsSimulation<T> for CpuSolver<T, B> {
    /// Multithreaded! It suffers on small amounts of particles currently....
    /// Also handles collisions slightly differently to usual.
    fn update(&mut self, delta_time: T) {
        self.update_bodies(delta_time);

        self.place_bodies_into_grid();

        let (collision_sender, collision_receiver) = channel();
        self.detect_collisions_extra_experimental(&collision_sender);
        drop(collision_sender);
        self.respond_to_collisions(collision_receiver.iter(), delta_time);

        for cell in &mut self.grid {
            if cell.capacity() == 0 {
                continue;
            }
            // This is meant to keep memory usage low, with only a minor performance cost, but I'm not convinced.
            // Even though we check for 0, this still seems dodgy. Perhaps this should be a choice for the user.
            if cell.len() <= cell.capacity() / 2 {
                //println!("len: {}, capacity: {}", cell.len(), cell.capacity());
                cell.shrink_to_fit();
            }
            cell.clear();
        }
    }
}

impl<T, B> CpuSolver<T, B>
where
    T: math::Float,
    B: Body<T>,
{
    pub fn new(config: Config<T, B>) -> CpuSolver<T, B> {
        CpuSolver {
            gravity: config.gravity,
            dampening: config.dampening,
            bodies: config.bodies,
            grid_size: config.grid_size,
            cell_size: config.cell_size,
            grid_origin: config.grid_origin,
            grid: vec![vec![]; config.grid_size[0] * config.grid_size[1] * config.grid_size[2]],
            outside_of_grid_bounds_behaviour: config.outside_of_grid_bounds_behaviour,
        }
    }

    #[inline]
    fn place_bodies_into_grid(&mut self) {
        let real_grid_width = self.grid_size[0] * self.cell_size[0];
        let real_grid_height = self.grid_size[1] * self.cell_size[1];
        let real_grid_length = self.grid_size[2] * self.cell_size[2];

        for (body_index, body) in self.bodies.iter_mut().enumerate() {
            if body.is_none() {
                continue;
            }

            let body_position = body.position_unchecked();

            let corrected_position = [
                body_position[0] - self.grid_origin[0],
                body_position[1] - self.grid_origin[1],
                body_position[2] - self.grid_origin[2],
            ];

            let corrected_position_as_isize = [
                corrected_position[0].to_isize(),
                corrected_position[1].to_isize(),
                corrected_position[2].to_isize(),
            ];

            let corrected_position_as_usize = [
                corrected_position[0].to_usize(),
                corrected_position[1].to_usize(),
                corrected_position[2].to_usize(),
            ];

            let outside_side = [
                corrected_position_as_isize[0] as usize > real_grid_width - 1,
                corrected_position_as_isize[0] < 0,
                corrected_position_as_isize[1] as usize > real_grid_height - 1,
                corrected_position_as_isize[1] < 0,
                corrected_position_as_isize[2] as usize > real_grid_length - 1,
                corrected_position_as_isize[2] < 0,
            ];

            if outside_side[0]
                || outside_side[1]
                || outside_side[2]
                || outside_side[3]
                || outside_side[4]
                || outside_side[5]
            {
                //println!("corrected position: {:?}", corrected_position); // Very useful debug!
                // Perhaps have this per body? Nah too slow.
                match self.outside_of_grid_bounds_behaviour {
                    OutsideOfGridBoundsBehaviour::SwapDeleteParticle => {
                        todo!();
                        //self.particles.swap_remove(particle_index);
                        //continue;
                    }
                    OutsideOfGridBoundsBehaviour::DeleteParticle => {
                        todo!();
                        //self.particles.remove(particle_index);
                        //continue;
                    }
                    OutsideOfGridBoundsBehaviour::PutParticleInBounds => {
                        todo!();
                    }
                    OutsideOfGridBoundsBehaviour::TeleportParticleToPosition(_position) => {
                        todo!()
                        //particle.previous_position = position;
                        //particle.position = position;
                    }
                    OutsideOfGridBoundsBehaviour::ContinueUpdating => {
                        continue;
                    }
                    OutsideOfGridBoundsBehaviour::Custom(function) => {
                        function(body_index, body);
                        continue;
                    }
                }
            }

            let grid_cell_position = [
                corrected_position_as_usize[0] / self.cell_size[0],
                corrected_position_as_usize[1] / self.cell_size[1],
                corrected_position_as_usize[2] / self.cell_size[2],
            ];

            let grid_cell_position_isize = [
                grid_cell_position[0] as isize,
                grid_cell_position[1] as isize,
                grid_cell_position[2] as isize,
            ];

            let body_half_size = body.half_size_unchecked();
            let body_half_size_isize = [
                body_half_size[0]
                    .ceil()
                    .to_isize()
                    .div_ceil(self.cell_size[0] as isize),
                body_half_size[1]
                    .ceil()
                    .to_isize()
                    .div_ceil(self.cell_size[1] as isize),
                body_half_size[2]
                    .ceil()
                    .to_isize()
                    .div_ceil(self.cell_size[2] as isize),
            ];

            //println!("body half size isize: {:?}",body_half_size_isize);

            /*
            if body_half_size_isize == [1, 1, 1] {
                let grid_cell_index = math::index_from_position_3d(
                    grid_cell_position,
                    self.grid_size[0],
                    self.grid_size[1],
                );

                // If something is wrong, this debug information is usually helpful.
                self.grid
                    .get_mut(grid_cell_index)
                    .unwrap_or_else(|| {
                        println!("body_position: {:?}", body_position);
                        println!(
                            "corrected_position_as_isize: {:?}",
                            corrected_position_as_isize
                        );
                        println!("grid_cell_position: {:?}", grid_cell_position);
                        println!("self.grid_size: {:?}", self.grid_size);
                        panic!()
                    })
                    .push(body_index);

                continue;
            }
            */

            for x in (grid_cell_position_isize[0] - body_half_size_isize[0])
                ..(grid_cell_position_isize[0] + body_half_size_isize[0])
            {
                if x < 0 {
                    continue;
                }
                if x >= self.grid_size[0] as isize {
                    continue;
                }
                for y in (grid_cell_position_isize[1] - body_half_size_isize[1])
                    ..(grid_cell_position_isize[1] + body_half_size_isize[1])
                {
                    if y < 0 {
                        continue;
                    }
                    if y >= self.grid_size[1] as isize {
                        continue;
                    }
                    for z in (grid_cell_position_isize[2] - body_half_size_isize[2])
                        ..(grid_cell_position_isize[2] + body_half_size_isize[2])
                    {
                        if z < 0 {
                            continue;
                        }
                        if z >= self.grid_size[2] as isize {
                            continue;
                        }
                        let current_grid_cell_position = [x as usize, y as usize, z as usize];
                        let grid_cell_index = math::index_from_position_3d(
                            current_grid_cell_position,
                            self.grid_size[0],
                            self.grid_size[1],
                        );

                        // If something is wrong, this debug information is usually helpful.
                        self.grid
                            .get_mut(grid_cell_index)
                            .unwrap_or_else(|| {
                                println!("body_position: {:?}", body_position);
                                println!(
                                    "corrected_position_as_isize: {:?}",
                                    corrected_position_as_isize
                                );
                                println!("grid_cell_position: {:?}", grid_cell_position);
                                println!(
                                    "current_grid_cell_position: {:?}",
                                    current_grid_cell_position
                                );
                                println!("self.grid_size: {:?}", self.grid_size);
                                panic!()
                            })
                            .push(body_index);
                    }
                }
            }
        }
    }

    #[inline]
    #[allow(dead_code)]
    fn serial_update_bodies(&mut self, delta_time: T) {
        self.bodies.iter_mut().for_each(|body| {
            if body.is_none() {
                return;
            }

            body.update(self.gravity, self.dampening, delta_time);
        });
    }

    #[inline]
    fn update_bodies(&mut self, delta_time: T) {
        self.bodies.par_iter_mut().for_each(|body| {
            if body.is_none() {
                return;
            }

            body.update(self.gravity, self.dampening, delta_time);
        });
    }

    #[inline]
    fn detect_collisions_extra_experimental(&self, collision_sender: &Sender<(usize, usize)>) {
        //TODO: check whether this is the right place to par_iter, rather than one of the other for loops.
        (&self.grid).into_par_iter().for_each(|cell| {
            for lhs_body_index in cell {
                if !self.bodies[*lhs_body_index].collide_with_others() {
                    continue;
                }
                for rhs_body_index in cell {
                    if lhs_body_index == rhs_body_index {
                        continue;
                    }

                    if self.bodies[*lhs_body_index].detect_collision(&self.bodies[*rhs_body_index])
                    {
                        collision_sender
                            .send((*lhs_body_index, *rhs_body_index))
                            .unwrap();
                    }
                }
            }
        });
    }

    #[inline]
    fn respond_to_collisions(
        &mut self,
        collisions: std::sync::mpsc::Iter<'_, (usize, usize)>,
        delta_time: T,
    ) {
        for (lhs_body_index, rhs_body_index) in collisions {
            // This code is simple and elegant. By splitting at the largest index, it allows us to safely and &mutably yoink the verlet bodies.
            if lhs_body_index > rhs_body_index {
                let (lhs_bodies, rhs_bodies) = self.bodies.split_at_mut(lhs_body_index);
                //rhs_bodies[0].collide(&mut lhs_bodies[rhs_body_index], rhs_body_index, delta_time);
                rhs_bodies[0].respond_to_collision(
                    &mut lhs_bodies[rhs_body_index],
                    rhs_body_index,
                    delta_time,
                );
            } else {
                let (lhs_bodies, rhs_bodies) = self.bodies.split_at_mut(rhs_body_index);
                //lhs_bodies[lhs_body_index].collide(&mut rhs_bodies[0], rhs_body_index, delta_time);
                lhs_bodies[lhs_body_index].respond_to_collision(
                    &mut rhs_bodies[0],
                    rhs_body_index,
                    delta_time,
                );
            }
        }
    }
}

/// If a body is outside of the grid, what should it do?
pub enum OutsideOfGridBoundsBehaviour<T: math::Float, B: Body<T>> {
    SwapDeleteParticle,
    DeleteParticle,
    PutParticleInBounds,
    TeleportParticleToPosition([T; 3]),
    ContinueUpdating,
    Custom(fn(usize, &mut B)),
    // replace with body?
}

#[cfg(test)]
mod tests {
    use crate::physics::physics_3d::bodies::CommonBody;
    use crate::physics::physics_3d::verlet::bodies::Cuboid;
    use crate::physics::physics_3d::verlet::Particle;

    use crate::math::Float;

    use super::*;
    use rand::thread_rng;
    use rand::Rng;
    use test::Bencher;

    fn create_test_solver<T: Float + rand::distributions::uniform::SampleUniform>(
        amount: usize,
        gravity: T,
    ) -> CpuSolver<T, CommonBody<T>> {
        let mut verlet_bodies = Vec::with_capacity(amount);
        let mut rng = thread_rng();

        for _ in 0..amount {
            verlet_bodies
                .push_within_capacity(CommonBody::Cuboid(Cuboid {
                    particle: Particle::from_position([
                        rng.gen_range(T::from_f64(-50.0)..T::from_f64(50.0)),
                        rng.gen_range(T::from_f64(-50.0)..T::from_f64(50.0)),
                        rng.gen_range(T::from_f64(-50.0)..T::from_f64(50.0)),
                    ]),

                    half_size: [T::from_f64(0.5); 3],
                }))
                .unwrap();
        }

        CpuSolver::new(Config {
            gravity: [T::ZERO, gravity, T::ZERO],
            dampening: [T::from_f64(0.8), T::ONE, T::from_f64(0.8)],
            grid_size: [10, 10, 10],
            grid_origin: [T::from_f64(-50.0), T::from_f64(-50.0), T::from_f64(-50.0)],
            cell_size: [10, 10, 10],
            outside_of_grid_bounds_behaviour: OutsideOfGridBoundsBehaviour::ContinueUpdating,
            bodies: verlet_bodies,
        })
    }

    // (10,22), (1000,2_468), (5000,12_518), (10_000,28_238), (20_000,54_449), (50_000,171_110), (100_000,672_048)
    #[bench]
    fn bench_cpu_solver_serial_update_100_000_particles(b: &mut Bencher) {
        let mut solver = create_test_solver::<f32>(100_000, 0.0);
        b.iter(|| {
            solver.serial_update_bodies(0.04);
        })
    }

    // (10,7_581), (1000,21_242), (5000,32_108), (10_000,69_903), (20_000,171_272), (50_000,385_621), (100_000,821_128)
    #[bench]
    fn bench_cpu_solver_update_100_000_particles(b: &mut Bencher) {
        let mut solver = create_test_solver::<f32>(100_000, 0.0);
        b.iter(|| {
            solver.update_bodies(0.04);
        })
    }

    #[bench]
    fn bench_cpu_solver_30000_particles(b: &mut Bencher) {
        let mut solver = create_test_solver(30000, 0.0);
        b.iter(|| {
            solver.update(0.04);
        })
    }

    #[bench]
    fn bench_cpu_solver_1000_none_particles(b: &mut Bencher) {
        let mut verlet_bodies = Vec::with_capacity(1000);

        for _ in 0..1000 {
            verlet_bodies.push(CommonBody::None);
        }

        let mut solver = CpuSolver::new(Config {
            gravity: [0.0, 50.0, 0.0],
            dampening: [0.8, 1.0, 0.8],
            grid_size: [10, 10, 10],
            grid_origin: [-50.0, -50.0, -50.0],
            cell_size: [10, 10, 10],
            outside_of_grid_bounds_behaviour: OutsideOfGridBoundsBehaviour::ContinueUpdating,
            bodies: verlet_bodies,
        });
        b.iter(|| {
            solver.update(0.04);
        })
    }
}
