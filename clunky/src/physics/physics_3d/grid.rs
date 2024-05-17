use crate::math::index_from_position_3d;

pub trait GridElement {
    fn half_size(&self) -> [f32; 3];
    fn position(&self) -> [f32; 3];
}

/// Config to make a grid out of.
/// 'extent' is measured in cell_size.
/// I think offset is using +y is up?
/// 'scaled_extent' is extent * cell_size.
#[derive(Debug)]
pub struct GridConfig {
    extent: [usize; 3],
    // Usize??? Perhaps f32?
    cell_size: [usize; 3],
    offset: [f32; 3],
}

impl GridConfig {
    pub fn new(extent: [usize; 3], cell_size: [usize; 3], offset: [f32; 3]) -> Self {
        Self {
            extent,
            cell_size,
            offset,
        }
    }
}

/// A simple grid.
/// +y is up though. I think?
pub struct Grid<T: Clone> {
    pub grid: Vec<Vec<T>>,
    pub grid_config: GridConfig,
}

impl<T: Clone> Grid<T> {
    pub fn new(grid_config: GridConfig) -> Self {
        Self {
            grid: vec![
                vec![];
                grid_config.extent[0] * grid_config.extent[1] * grid_config.extent[2]
            ],
            grid_config,
        }
    }

    pub fn index_grid_by_area<F>(
        &mut self,
        element_position: [f32; 3],
        element_half_size: [f32; 3],
        mut operation: F,
    ) where
        F: FnMut(&mut Vec<T>),
    {
        // Previously corrected_position, was available in usize, isize, and f32.
        // Also notably truncated, both now and previously. Did not round. At least I think so? It used "as".
        let unoffset_truncated_position = [
            (element_position[0] - self.grid_config.offset[0]) as isize,
            (element_position[1] - self.grid_config.offset[1]) as isize,
            (element_position[2] - self.grid_config.offset[2]) as isize,
        ];

        // What was here used to be checking to see if anything was outside.

        // This used to be usize, with an isize version below
        let grid_cell_position = [
            unoffset_truncated_position[0] / self.grid_config.cell_size[0] as isize,
            unoffset_truncated_position[1] / self.grid_config.cell_size[1] as isize,
            unoffset_truncated_position[2] / self.grid_config.cell_size[2] as isize,
        ];

        let element_half_grid_cell_extent = [
            (element_half_size[0].ceil() as isize).div_ceil(self.grid_config.cell_size[0] as isize),
            (element_half_size[1].ceil() as isize).div_ceil(self.grid_config.cell_size[1] as isize),
            (element_half_size[2].ceil() as isize).div_ceil(self.grid_config.cell_size[2] as isize),
        ];

        for x in (grid_cell_position[0] - element_half_grid_cell_extent[0])
            ..(grid_cell_position[0] + element_half_grid_cell_extent[0])
        {
            if x < 0 {
                continue;
            }
            if x >= self.grid_config.extent[0] as isize {
                continue;
            }
            for y in (grid_cell_position[1] - element_half_grid_cell_extent[1])
                ..(grid_cell_position[1] + element_half_grid_cell_extent[1])
            {
                if y < 0 {
                    continue;
                }
                if y >= self.grid_config.extent[1] as isize {
                    continue;
                }
                for z in (grid_cell_position[2] - element_half_grid_cell_extent[2])
                    ..(grid_cell_position[2] + element_half_grid_cell_extent[2])
                {
                    if z < 0 {
                        continue;
                    }
                    if z >= self.grid_config.extent[2] as isize {
                        continue;
                    }
                    let current_grid_cell_position = [x as usize, y as usize, z as usize];
                    let grid_cell_index = index_from_position_3d(
                        current_grid_cell_position,
                        self.grid_config.extent[0],
                        self.grid_config.extent[1],
                    );

                    // If something is wrong, having some debug information is usually helpful.
                    let grid_cell = &mut self.grid.get_mut(grid_cell_index).unwrap_or_else(|| {
                        println!("grid_config: {:?}", self.grid_config);
                        panic!()
                    });

                    operation(grid_cell);
                }
            }
        }
    }

    /// Checks whether the position and size have changed enough to warrant re-insertion.
    pub fn grid_area_changed() -> bool {
        todo!()
    }
}
