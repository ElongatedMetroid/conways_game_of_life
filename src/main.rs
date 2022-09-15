use std::{fmt, thread, time::Duration, process};

/// Height, Y
const GRID_ROWS: usize = 100;
/// Width, X
const GRID_COLS: usize = 150;

#[derive(Debug)]
struct Game {
    /// 100x150 grid of cells (for now)
    grid: Vec<Vec<Cell>>,
    generation: usize,
}

#[derive(PartialEq, Debug)]
enum CellState {
    /// Cell is live (active)
    Live,
    /// Cell is dead (inactive)
    Dead,
}

#[derive(Debug)]
struct Cell {
    /// Holds the current state of the cell
    state: CellState,
    /// Holds the index of the neighbors of Self
    neighbors: Vec<Vec2>,
}

#[derive(Debug)]
struct Vec2 {
    x: usize, 
    y: usize,
}

impl Vec2 {
    pub fn new(x: usize, y: usize) -> Vec2 {
        Vec2 { x, y }
    }
}

impl Cell {
    /// Returns a new empty cell
    fn new(state: CellState) -> Cell {
        Cell { state, neighbors: Vec::new() }
    }
    pub fn is_live(&self) -> bool {
        self.state == CellState::Live
    }
}

impl Game {
    pub fn new(seed: usize) -> Game {
        let mut game = Game { grid: Vec::new(), generation: 0 };

        for _ in 0..GRID_ROWS {
            let mut row = Vec::new();

            for _ in 0..GRID_COLS {
                row.push(Cell::new(CellState::Dead));
            }

            game.grid.push(row);
        }

        // For now the grid will just be 100x100x
        game.grid[50][50] = Cell::new(CellState::Live);

        game.set_cell_neighbors();

        game
    }
    pub fn step(&mut self) {
        // Any cell with fewer than two live neightbours dies
        // as if by underpopulation

        // Any live cell with two or three live neightbours lives
        // on to the next generation

        // Any live cell with more than three live neightbours
        // dies, as if by overpopulation

        // Any dead cell with exactly three live neighbours 
        // becomes a live cell, as if by reproduction
    }
    fn set_cell_neighbors(&mut self) {
        for (y, row) in self.grid.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                // Top Left
                cell.neighbors.push(
                    // Wrap to bottom right if overflow
                    Vec2::new(if x.overflowing_sub(1).1 {
                        GRID_COLS - 1
                    } else { x - 1 }, if y.overflowing_sub(1).1 {
                        GRID_ROWS - 1
                    } else { y - 1 })
                );
                // Top
                cell.neighbors.push(
                    // Wrap y to the bottom y on the same x avis
                    Vec2::new(x, if y.overflowing_sub(1).1 {
                        GRID_ROWS - 1
                    } else { y - 1 })
                );
                // Top Right
                cell.neighbors.push(
                    // Wrap to bottom left
                    Vec2::new(if x+1 > GRID_COLS - 1 {
                        // All the way left on x
                        0
                    } else { x + 1 }, if y.overflowing_sub(1).1 {
                        GRID_ROWS - 1
                    } else { y - 1 })
                );
                // Left
                cell.neighbors.push(
                    // Wrap to right
                    Vec2::new(if x.overflowing_sub(1).1 {
                        GRID_COLS - 1
                    } else { x - 1 }, y)
                );
                // Right
                cell.neighbors.push(
                    // Wrap to left
                    Vec2::new(if x+1 > GRID_COLS - 1 {
                        0
                    } else { x + 1 }, y)
                );
                // Bottom Left
                cell.neighbors.push(
                    // Wrap to top right
                    Vec2::new(if x.overflowing_sub(1).1 {
                        GRID_COLS - 1
                    } else { x - 1 }, if y+1 > GRID_ROWS - 1 {
                        0
                    } else { y + 1 })
                );
                // Bottom
                cell.neighbors.push(
                    // Wrap to top
                    Vec2::new(x, if y+1 > GRID_ROWS - 1 {
                        0
                    } else { y + 1 })
                );
                // Bottom Right
                cell.neighbors.push(
                    // Wrap to top left
                    Vec2::new(if x+1 > GRID_COLS - 1 {
                        0
                    } else { x + 1 }, if y+1 > GRID_ROWS - 1 {
                        0
                    } else { y + 1 })
                );
            }
        }
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut grid = String::new();

        for row in &self.grid {
            for cell in row {
                if cell.is_live() {
                    grid.push('■');
                } else {
                    grid.push('▢');
                }
            }

            grid.push('\n');
        }

        write!(f, "{grid}")
    }
}

fn main() {
    let mut game = Game::new(0);

    println!("{:#?}", game);

    loop {
        game.step();

        thread::sleep(Duration::from_secs(1));

        println!("{game}");

        process::exit(0);
    }
}
