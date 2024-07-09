use crate::math;
extern crate test;

pub trait Cell: Copy + Clone {}

#[derive(Clone)]
pub enum CellState<T>
where
    T: Cell,
{
    Decided(T),
    Undecided,
}

pub fn generate_2d_unoptimized_with_no_assumptions<T>(
    size: [usize; 2],
    starting_position: [usize; 2],
    default_possibilities: Vec<T>,
    get_possibilities: fn(&Vec<CellState<T>>, usize) -> Vec<T>,
    pick_possibility: fn(&Vec<CellState<T>>, Vec<T>, usize) -> T,
) -> Vec<T>
where
    T: Cell,
{
    let mut cells = vec![CellState::Undecided; size[0] * size[1]];

    let starting_index = math::index_from_position_2d(starting_position, size[0]);
    cells[starting_index] = CellState::Decided(pick_possibility(
        &cells,
        default_possibilities,
        starting_index,
    ));

    let mut undecided_cells = cells.len() - 1;

    while undecided_cells > 0 {
        let mut possibilities = vec![];
        let mut cell_index = usize::MAX;
        let mut smallest_len = usize::MAX;

        for potential_cell_index in 0..cells.len() {
            if let CellState::Decided(_) = cells[potential_cell_index] {
                continue;
            }

            let potential_possibilities = get_possibilities(&cells, potential_cell_index);
            if potential_possibilities.len() < smallest_len {
                possibilities = potential_possibilities;
                cell_index = potential_cell_index;
                smallest_len = possibilities.len();
            }
        }

        cells[cell_index] = CellState::Decided(pick_possibility(&cells, possibilities, cell_index));

        undecided_cells -= 1;
    }

    let mut unwrapped_cells = vec![];

    for cell in cells {
        if let CellState::Decided(cell) = cell {
            unwrapped_cells.push(cell);
        } else {
            unreachable!("A cell was not decided. This error message needs improving. This should never happen though.");
        }
    }

    unwrapped_cells
}

#[derive(Clone)]
pub enum CellStateStorePossibilities<T>
where
    T: Cell,
{
    Decided(T),
    Undecided(Vec<T>), // TODO: Allows for potential optimization by only computing the possibilities when an adjacent tile has changed, assuming adjacent tiles are the only thing that affects possibilities.
}

pub fn generate_2d_assumes_only_4_nearest_tiles_matter_and_starting_position_is_not_on_edge<T>(
    size: [usize; 2],
    starting_position: [usize; 2],
    default_possibilities: Vec<T>,
    get_possibilities: fn(&Vec<CellStateStorePossibilities<T>>, usize) -> Vec<T>,
    pick_possibility: fn(&Vec<CellStateStorePossibilities<T>>, &Vec<T>, usize) -> T,
) -> Vec<T>
where
    T: Cell,
{
    let mut cells = vec![
        CellStateStorePossibilities::Undecided(default_possibilities.clone());
        size[0] * size[1]
    ];

    let starting_index = math::index_from_position_2d(starting_position, size[0]);
    cells[starting_index] = CellStateStorePossibilities::Decided(pick_possibility(
        &cells,
        &default_possibilities,
        starting_index,
    ));

    cells[starting_index + 1] =
        CellStateStorePossibilities::Undecided(get_possibilities(&cells, starting_index + 1));

    cells[starting_index - 1] =
        CellStateStorePossibilities::Undecided(get_possibilities(&cells, starting_index - 1));

    cells[starting_index + size[0]] =
        CellStateStorePossibilities::Undecided(get_possibilities(&cells, starting_index + size[0]));

    cells[starting_index - size[0]] =
        CellStateStorePossibilities::Undecided(get_possibilities(&cells, starting_index - size[0]));

    let mut undecided_cells = cells.len() - 1;

    while undecided_cells > 0 {
        let mut cell_index = usize::MAX;
        let mut smallest_len = usize::MAX;
        let mut possibilities = &vec![];

        for (potential_cell_index, potential_cell) in cells.iter().enumerate() {
            if let CellStateStorePossibilities::Undecided(potential_possibilities) = &potential_cell
            {
                if potential_possibilities.len() < smallest_len {
                    cell_index = potential_cell_index;
                    smallest_len = possibilities.len();
                    possibilities = potential_possibilities;
                }
            }
        }

        let cell_position = math::position_from_index_2d(cell_index, size[0]);

        cells[cell_index] = CellStateStorePossibilities::Decided(pick_possibility(
            &cells,
            possibilities,
            cell_index,
        ));

        if cell_position[0] != size[0] - 1 {
            if let CellStateStorePossibilities::Undecided(_) = cells[cell_index + 1] {
                cells[cell_index + 1] = CellStateStorePossibilities::Undecided(get_possibilities(
                    &cells,
                    cell_index + 1,
                ));
            }
        }

        if cell_position[0] != 0 {
            if let CellStateStorePossibilities::Undecided(_) = cells[cell_index - 1] {
                cells[cell_index - 1] = CellStateStorePossibilities::Undecided(get_possibilities(
                    &cells,
                    cell_index - 1,
                ));
            }
        }

        if cell_position[1] != size[1] - 1 {
            if let CellStateStorePossibilities::Undecided(_) = cells[cell_index + size[0]] {
                cells[cell_index + size[0]] = CellStateStorePossibilities::Undecided(
                    get_possibilities(&cells, cell_index + size[0]),
                );
            }
        }

        if cell_position[1] != 0 {
            if let CellStateStorePossibilities::Undecided(_) = cells[cell_index - size[0]] {
                cells[cell_index - size[0]] = CellStateStorePossibilities::Undecided(
                    get_possibilities(&cells, cell_index - size[0]),
                );
            }
        }

        undecided_cells -= 1;
    }

    let mut unwrapped_cells = vec![];

    for cell in cells {
        if let CellStateStorePossibilities::Decided(cell) = cell {
            unwrapped_cells.push(cell);
        } else {
            unreachable!("A cell was not decided. This error message needs improving. This should never happen though.");
        }
    }

    unwrapped_cells
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;
    use test::Bencher;

    #[derive(Clone, Copy, Debug, PartialEq)]
    enum TestTiles {
        Tile1,
        Tile2,
        Tile3,
    }

    impl Cell for TestTiles {}

    fn get_possibilities_stored(
        _cells: &Vec<CellStateStorePossibilities<TestTiles>>,
        _cell_index: usize,
    ) -> Vec<TestTiles> {
        vec![TestTiles::Tile1, TestTiles::Tile2, TestTiles::Tile3]
    }

    fn get_possibilities(_cells: &Vec<CellState<TestTiles>>, _cell_index: usize) -> Vec<TestTiles> {
        vec![TestTiles::Tile1, TestTiles::Tile2, TestTiles::Tile3]
    }

    fn pick_possibility_stored(
        _cells: &Vec<CellStateStorePossibilities<TestTiles>>,
        possibilities: &Vec<TestTiles>,
        _cell_index: usize,
    ) -> TestTiles {
        possibilities[rand::thread_rng().gen_range(0..possibilities.len())]
    }

    fn pick_possibility(
        _cells: &Vec<CellState<TestTiles>>,
        possibilities: Vec<TestTiles>,
        _cell_index: usize,
    ) -> TestTiles {
        possibilities[rand::thread_rng().gen_range(0..possibilities.len())]
    }

    #[bench]
    fn bench_100_by_100_generate_2d_assumes_only_4_nearest_tiles_matter_and_starting_position_is_not_on_edge(
        b: &mut Bencher,
    ) {
        b.iter(|| {
            return generate_2d_assumes_only_4_nearest_tiles_matter_and_starting_position_is_not_on_edge([100,100], [50,50], vec![
                TestTiles::Tile1,
                TestTiles::Tile2,
                TestTiles::Tile3,
            ], get_possibilities_stored, pick_possibility_stored);
        })
    }

    #[bench]
    fn bench_100_by_100_generate_2d_unoptimized_with_no_assumptions(b: &mut Bencher) {
        panic!("too slow");
        /*
        b.iter(|| {
            return generate_2d_unoptimized_with_no_assumptions(
                [100, 100],
                [50, 50],
                vec![TestTiles::Tile1, TestTiles::Tile2, TestTiles::Tile3],
                get_possibilities,
                pick_possibility,
            );
        })
        */
    }
}
