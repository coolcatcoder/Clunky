use crate::math;
extern crate test;

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

// Solvers are always way too set in stone for my liking currently. Same with the VerletBody struct.

/// This should not exist. This will be replaced with a trait accessible from usercode, so they can specify what bodies they want.
pub enum VerletBody<T>
where
    T: math::Float,
{
    SimpleBox(SimpleBox<T>),
    ImmovableSimpleBox(ImmovableSimpleBox<T>)
}

impl<T> VerletBody<T>
where
    T: math::Float,
{
    pub fn update_and_get_position(
        &mut self,
        gravity: [T; 3],
        dampening: [T; 3],
        delta_time: T,
    ) -> [T; 3] {
        match self {
            VerletBody::SimpleBox(simple_box) => {
                simple_box.update_and_get_position(gravity, dampening, delta_time)
            }
            VerletBody::ImmovableSimpleBox(immovable_simple_box) => {
                immovable_simple_box.update_and_get_position(gravity, dampening, delta_time)
            }
        }
    }

    // terribly named, but very useful. TODO: Replace with proper lhs and rhs bitwise values for checking how objects can interact.
    pub fn collide_with_others(&mut self) -> bool {
        match self {
            VerletBody::SimpleBox(_) => true,
            VerletBody::ImmovableSimpleBox(_) => false,
        }
    }

    // This function is insanely expensive, no clue why.
    #[inline]
    pub fn collide(&mut self, other: &mut VerletBody<T>) {
        match (self, other) {
            // simple box
            (VerletBody::SimpleBox(lhs_simple_box), VerletBody::SimpleBox(rhs_simple_box)) => {
                // TODO: Mess with ref mut and mut and all that, and you will gain and lose 70x performance, somehow.
                if lhs_simple_box.aabb.is_intersected_by_aabb(rhs_simple_box.aabb) {
                    lhs_simple_box.particle.position = lhs_simple_box.particle.previous_position; // these lines in particular are slow
                    rhs_simple_box.particle.position = rhs_simple_box.particle.previous_position;
                }
            }
            (VerletBody::SimpleBox(lhs_simple_box), VerletBody::ImmovableSimpleBox(rhs_immovable_simple_box)) => {
                if lhs_simple_box.aabb.is_intersected_by_aabb(rhs_immovable_simple_box.aabb) {
                    let collision_axis = rhs_immovable_simple_box.aabb.get_collision_axis_with_direction(super::aabb::AabbCentredOrigin {
                        position: lhs_simple_box.particle.previous_position,
                        half_size: lhs_simple_box.aabb.half_size,
                    });

                    //println!("collision axis: {:?}", collision_axis);

                    lhs_simple_box.particle.position = lhs_simple_box.particle.previous_position;

                    //println!("Collision!");
                }
            }

            // immovable simple box (This cannot happen, as immovable simple boxes don't check to see if they have collided with others.)
            (VerletBody::ImmovableSimpleBox(_), VerletBody::SimpleBox(_)) => unreachable!(),
            (VerletBody::ImmovableSimpleBox(_), VerletBody::ImmovableSimpleBox(_)) => unreachable!(),
        }
    }
}

// Horrifically bad name.
//#[derive(Clone, Copy)]
pub struct SimpleBox<T>
where
    T: math::Float,
{
    pub particle: Particle<T>, // Do we even want to store particles with collision data?
    pub aabb: crate::physics::physics_3d::aabb::AabbCentredOrigin<T>,
}

impl<T> SimpleBox<T>
where
    T: math::Float,
{
    pub fn update_and_get_position(
        &mut self,
        gravity: [T; 3],
        dampening: [T; 3],
        delta_time: T,
    ) -> [T; 3] {
        self.particle.accelerate(gravity);
        self.particle.update(
            delta_time,
            math::mul_3d(self.particle.calculate_displacement(), dampening),
        );

        self.aabb.position = self.particle.position;

        self.particle.position
    }
}

pub struct ImmovableSimpleBox<T>
where
    T: math::Float,
{
    pub aabb: crate::physics::physics_3d::aabb::AabbCentredOrigin<T>,
}

impl<T> ImmovableSimpleBox<T>
where
    T: math::Float,
{
    pub fn update_and_get_position(
        &mut self,
        _gravity: [T; 3],
        _dampening: [T; 3],
        _delta_time: T,
    ) -> [T; 3] {
        self.aabb.position
    }
}

pub struct SingleThreadedSolver<T>
where
    T: math::Float,
{
    pub gravity: [T; 3],
    pub dampening: [T; 3], // Where 1.0 is no dampening. Perhaps displacement_kept is a better name?
    pub verlet_bodies: Vec<VerletBody<T>>,
    pub grid_size: [usize; 3], // This is in cell size units. This should probably be clarified.
    pub cell_size: [usize; 3], // TODO: asap work out how the usize vs isize nonsense will work, as we want this to work for negatives. Perhaps we can plus some sort of offset for the particle?
    pub grid_origin: [T; 3], // Remember that the origin is the bottom left corner of the grid, I think.
    pub grid: Vec<Vec<usize>>,
    pub outside_of_grid_bounds_behaviour: OutsideOfGridBoundsBehaviour<T>,
}

impl<T> SingleThreadedSolver<T>
where
    T: math::Float,
{
    pub fn new(
        gravity: [T; 3],
        dampening: [T; 3],
        grid_size: [usize; 3],
        grid_origin: [T; 3],
        cell_size: [usize; 3],
        outside_of_grid_bounds_behaviour: OutsideOfGridBoundsBehaviour<T>,
        verlet_bodies: Vec<VerletBody<T>>,
    ) -> SingleThreadedSolver<T> {
        SingleThreadedSolver {
            gravity,
            dampening,
            verlet_bodies,
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

        for (verlet_body_index, verlet_body) in (0..self.verlet_bodies.len())
            .into_iter()
            .zip(&mut self.verlet_bodies)
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
                corrected_position_as_isize[0] as usize > real_grid_width,
                corrected_position_as_isize[0] < 0,
                corrected_position_as_isize[1] as usize > real_grid_height,
                corrected_position_as_isize[1] < 0,
                corrected_position_as_isize[2] as usize > real_grid_length,
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

            self.grid[grid_cell_index].push(verlet_body_index);
        }

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
                if !self.verlet_bodies[*lhs_verlet_body_index].collide_with_others() {
                    continue;
                }

                for neighbour in &neighbours {
                    for rhs_verlet_body_index in *neighbour {
                        if lhs_verlet_body_index == rhs_verlet_body_index {
                            continue
                        }

                        // This code is simple and elgant. By splitting at the largest index, it allows us to safely and &mutably yoink the verlet bodies.
                        if lhs_verlet_body_index > rhs_verlet_body_index {
                            let (lhs_verlet_bodies, rhs_verlet_bodies) = self.verlet_bodies.split_at_mut(*lhs_verlet_body_index);
                            rhs_verlet_bodies[0].collide(&mut lhs_verlet_bodies[*rhs_verlet_body_index]);
                            //collide_test(&mut rhs_verlet_bodies[0], &mut lhs_verlet_bodies[*rhs_verlet_body_index]);
                            //test::black_box(lhs_verlet_bodies);
                            //test::black_box(rhs_verlet_bodies);
                            //test::black_box(&mut rhs_verlet_bodies[0]);
                            //test::black_box(&mut lhs_verlet_bodies[*rhs_verlet_body_index]);
                        } else {
                            let (lhs_verlet_bodies, rhs_verlet_bodies) = self.verlet_bodies.split_at_mut(*rhs_verlet_body_index);
                            lhs_verlet_bodies[*lhs_verlet_body_index].collide(&mut rhs_verlet_bodies[0]);
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
    use super::*;
    use rand::thread_rng;
    use rand::Rng;
    use test::Bencher;

    #[bench]
    fn bench_single_threaded_solver_1000_particles(b: &mut Bencher) {
        let mut verlet_bodies = Vec::with_capacity(1000);
        let mut rng = thread_rng();

        for _ in 0..1000 {
            verlet_bodies.push(VerletBody::SimpleBox(SimpleBox {
                particle: Particle::from_position([0.0, 0.0, 0.0]),
                aabb: crate::physics::physics_3d::aabb::AabbCentredOrigin {
                    position: [rng.gen_range(-50.0..50.0), rng.gen_range(-50.0..50.0), rng.gen_range(-50.0..50.0)],
                    half_size: [0.5, 0.5, 0.5]
                }
            }));
        }

        let mut solver = SingleThreadedSolver::new(
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
