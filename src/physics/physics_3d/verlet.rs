use rayon::prelude::*;

use crate::math;

use self::bodies::Body;
extern crate test;

pub mod bodies;

#[derive(Debug, Clone, Copy)]
pub struct Particle<T>
where
    T: math::Number,
{
    pub position: [T; 3],
    pub previous_position: [T; 3],
    pub acceleration: [T; 3],
}

impl<T> Particle<T>
where
    T: math::Number,
{
    #[inline]
    pub fn from_position(position: [T; 3]) -> Particle<T> {
        Particle {
            position,
            previous_position: position,
            acceleration: [T::ZERO; 3],
        }
    }

    #[inline] // Add must use attributes.
    pub fn calculate_displacement(&self) -> [T; 3] {
        math::sub_3d(self.position, self.previous_position)
    }

    pub fn update(&mut self, delta_time: T, displacement: [T; 3]) {
        self.previous_position = self.position;

        let acceleration = math::mul_3d_by_1d(self.acceleration, delta_time * delta_time);

        self.position = math::add_3d(math::add_3d(self.position, displacement), acceleration);

        self.acceleration = [T::ZERO; 3];
    }

    #[inline]
    pub fn accelerate(&mut self, acceleration: [T; 3]) {
        self.acceleration[0] += acceleration[0];
        self.acceleration[1] += acceleration[1];
        self.acceleration[2] += acceleration[2];
    }
}

// Solvers are always way too set in stone for my liking currently.

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
    pub outside_of_grid_bounds_behaviour: OutsideOfGridBoundsBehaviour<T>,
}

impl<T, B> CpuSolver<T, B>
where
    T: math::Float,
    B: Body<T>,
{
    pub fn new(
        gravity: [T; 3],
        dampening: [T; 3],
        grid_size: [usize; 3],
        grid_origin: [T; 3],
        cell_size: [usize; 3],
        outside_of_grid_bounds_behaviour: OutsideOfGridBoundsBehaviour<T>,
        bodies: Vec<B>,
    ) -> CpuSolver<T, B> {
        CpuSolver {
            gravity,
            dampening,
            bodies,
            grid_size,
            cell_size,
            grid_origin,
            grid: vec![vec![]; grid_size[0] * grid_size[1] * grid_size[2]],
            outside_of_grid_bounds_behaviour,
        }
    }

    pub fn update(&mut self, delta_time: T) {
        let real_grid_width = self.grid_size[0] * self.cell_size[0];
        let real_grid_height = self.grid_size[1] * self.cell_size[1];
        let real_grid_length = self.grid_size[2] * self.cell_size[2];

        for (verlet_body_index, verlet_body) in (0..self.bodies.len())
            .into_iter()
            .zip(&mut self.bodies)
        {
            let verlet_body_position =
                verlet_body.update_and_get_position(self.gravity, self.dampening, delta_time);

            let corrected_position = [
                verlet_body_position[0] - self.grid_origin[0], // + or - ????
                verlet_body_position[1] - self.grid_origin[1],
                verlet_body_position[2] - self.grid_origin[2],
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
                // Perhaps have this per verlet body?
                match self.outside_of_grid_bounds_behaviour {
                    OutsideOfGridBoundsBehaviour::DeleteParticle => {
                        todo!();
                        //self.particles.swap_remove(particle_index);
                        //continue;
                    }
                    OutsideOfGridBoundsBehaviour::DeleteParticleButPreserveOrder => {
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
                }
            }

            let grid_cell_position = [
                corrected_position_as_usize[0] / self.cell_size[0],
                corrected_position_as_usize[1] / self.cell_size[1],
                corrected_position_as_usize[2] / self.cell_size[2],
            ];

            let grid_cell_index = math::index_from_position_3d(
                grid_cell_position,
                self.grid_size[0],
                self.grid_size[1],
            );

            //self.grid[grid_cell_index].push(verlet_body_index); // got a panic here, so something must be terribly wrong. Update I think I fixed it with my -1 to everything on ouside side. Remove this once we are certain it is all good.
            self.grid.get_mut(grid_cell_index).unwrap_or_else(|| {
                println!("verlet_body_position: {:?}", verlet_body_position);
                println!("corrected_position_as_isize: {:?}", corrected_position_as_isize);
                println!("grid_cell_position: {:?}", grid_cell_position);
                panic!()
            }).push(verlet_body_index);
        }

        // TODO: How the hell can we multithread this?
        for cell_index in 0..self.grid.len() {
            let cell = &self.grid[cell_index];
            let cell_position =
                math::position_from_index_3d(cell_index, self.grid_size[0], self.grid_size[1]);

            // Debating how much I like performance. I don't want to write by hand 26 different cell lets. This will do:
            let mut neighbours = Vec::with_capacity(27); // 27 cause this includes the center cell.

            for x in -1..=1 {
                for y in -1..=1 {
                    for z in -1..=1 {
                        let position = [
                            cell_position[0] as isize + x,
                            cell_position[1] as isize + y,
                            cell_position[2] as isize + z,
                        ];

                        if position[0] >= 0
                            && position[0] < self.grid_size[0] as isize
                            && position[1] >= 0
                            && position[1] < self.grid_size[1] as isize
                            && position[2] >= 0
                            && position[2] < self.grid_size[2] as isize
                        {
                            neighbours.push(
                                &self.grid[math::index_from_position_3d(
                                    [
                                        position[0] as usize,
                                        position[1] as usize,
                                        position[2] as usize,
                                    ],
                                    self.grid_size[0],
                                    self.grid_size[1],
                                )],
                            );
                        }
                    }
                }
            }

            for lhs_verlet_body_index in cell {
                if !self.bodies[*lhs_verlet_body_index].collide_with_others() {
                    continue;
                }

                for neighbour in &neighbours {
                    for rhs_verlet_body_index in *neighbour {
                        if lhs_verlet_body_index == rhs_verlet_body_index {
                            continue;
                        }

                        // This code is simple and elgant. By splitting at the largest index, it allows us to safely and &mutably yoink the verlet bodies.
                        if lhs_verlet_body_index > rhs_verlet_body_index {
                            let (lhs_verlet_bodies, rhs_verlet_bodies) =
                                self.bodies.split_at_mut(*lhs_verlet_body_index);
                            rhs_verlet_bodies[0]
                                .collide(&mut lhs_verlet_bodies[*rhs_verlet_body_index], *rhs_verlet_body_index);
                            //collide_test(&mut rhs_verlet_bodies[0], &mut lhs_verlet_bodies[*rhs_verlet_body_index]);
                            //test::black_box(lhs_verlet_bodies);
                            //test::black_box(rhs_verlet_bodies);
                            //test::black_box(&mut rhs_verlet_bodies[0]);
                            //test::black_box(&mut lhs_verlet_bodies[*rhs_verlet_body_index]);
                        } else {
                            let (lhs_verlet_bodies, rhs_verlet_bodies) =
                                self.bodies.split_at_mut(*rhs_verlet_body_index);
                            lhs_verlet_bodies[*lhs_verlet_body_index]
                                .collide(&mut rhs_verlet_bodies[0], *rhs_verlet_body_index);
                            //collide_test(&mut lhs_verlet_bodies[*lhs_verlet_body_index], &mut rhs_verlet_bodies[0]);
                            //test::black_box(lhs_verlet_bodies);
                            //test::black_box(rhs_verlet_bodies);
                            //test::black_box(&mut lhs_verlet_bodies[*lhs_verlet_body_index]);
                            //test::black_box(&mut rhs_verlet_bodies[0]);
                        }
                    }
                }

                // cell is already part of the neighbours, so we don't have to worry about it.
            }
        }

        for cell in &mut self.grid {
            if cell.len() <= cell.capacity() / 2 {
                cell.shrink_to_fit();
            }
            cell.clear();
        }
    }
}

pub enum OutsideOfGridBoundsBehaviour<T: math::Number> {
    DeleteParticle,
    DeleteParticleButPreserveOrder,
    PutParticleInBounds,
    TeleportParticleToPosition([T; 3]),
    ContinueUpdating,
}

#[cfg(test)]
mod tests {
    use self::bodies::Cuboid;
    use self::bodies::CommonBody;

    use super::*;
    use rand::thread_rng;
    use rand::Rng;
    use test::Bencher;

    #[bench]
    fn bench_single_threaded_solver_1000_particles(b: &mut Bencher) {
        let mut verlet_bodies = Vec::with_capacity(1000);
        let mut rng = thread_rng();

        for _ in 0..1000 {
            verlet_bodies.push(CommonBody::Cuboid(Cuboid {
                particle: Particle::from_position([
                    rng.gen_range(-50.0..50.0),
                    rng.gen_range(-50.0..50.0),
                    rng.gen_range(-50.0..50.0),
                ]),

                    
                    half_size: [0.5, 0.5, 0.5],
                
            }));
        }

        let mut solver = CpuSolver::new(
            [0.0, 50.0, 0.0],
            [0.8, 1.0, 0.8],
            [10, 10, 10],
            [-50.0, -50.0, -50.0],
            [10, 10, 10],
            OutsideOfGridBoundsBehaviour::ContinueUpdating,
            verlet_bodies,
        );
        b.iter(|| {
            solver.update(0.04);
        })
    }
}
