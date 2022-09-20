use std::{thread, time::Duration, rc::Rc, cell::RefCell};

use config::Config;
use rand::Rng;

mod config;

/// Height, Y
const GRID_ROWS: usize = 50;
/// Width, X
const GRID_COLS: usize = 50;

#[derive(Debug)]
struct Game {
    /// 100x150 grid of cells (for now)
    grid: Vec<Vec<Cell>>,
    generation: usize,

    config: Rc<RefCell<Config>>,
}

#[derive(PartialEq, Debug, Clone)]
enum CellState {
    /// Cell is live (active)
    Live,
    /// Cell is dead (inactive)
    Dead,
}

#[derive(Debug, Clone)]
struct Cell {
    /// Holds the current state of the cell
    state: CellState,
    /// Holds the index of the neighbors of Self
    neighbors: Vec<Vec2>,
}

#[derive(Debug, Clone)]
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
    /// Returns a usize containing the amount of neighbors alive
    pub fn live_neighbor_count(&self, grid: &Vec<Vec<Cell>>) -> usize {
        let mut live_count = 0;

        for neighbor in &self.neighbors {
            if grid[neighbor.y][neighbor.x].is_live() {
                live_count+=1;
            }
        }

        live_count
    }
}

impl Game {
    pub fn new(config: Rc<RefCell<Config>>) -> Game {
        let mut game = Game { grid: Vec::new(), generation: 0, config };

        let grid_rows = game.config.borrow().grid_rows.unwrap_or(GRID_ROWS);
        let grid_cols = game.config.borrow().grid_cols.unwrap_or(GRID_COLS);

        for _ in 0..grid_rows {
            let mut row = Vec::new();

            for _ in 0..grid_cols {
                row.push(Cell::new(CellState::Dead));
            }

            game.grid.push(row);
        }

        // Saving so the user can later view what seed was used
        let seed = game.config.borrow().seed.unwrap_or(rand::thread_rng().gen());
        let mut numbers_added = Vec::new();
        for y in 0..grid_rows {
            for x in 0..grid_cols {
                let random = rand::thread_rng().gen::<usize>();
                numbers_added.push(random);
                let usage_seed = seed.overflowing_add(random).0;
                
                game.grid[y][x] = if (usage_seed % 2) == 0 {
                    Cell::new(CellState::Live)
                } else {
                    Cell::new(CellState::Dead)
                };
            }
        }

        game.set_cell_neighbors();

        game
    }
    pub fn step(&mut self) {
        self.generation += 1;
        let mut future = self.grid.clone();

        for y in 0..self.config.borrow().grid_rows.unwrap_or(GRID_ROWS) {
            for x in 0..self.config.borrow().grid_cols.unwrap_or(GRID_COLS) {
                future[y][x].state = if self.grid[y][x].live_neighbor_count(&self.grid) < 2 {
                    // Any cell with fewer than two live neighbors dies
                    // as if by underpopulation
                    CellState::Dead
                } else if self.grid[y][x].live_neighbor_count(&self.grid) == 2 && self.grid[y][x].is_live() ||
                          self.grid[y][x].live_neighbor_count(&self.grid) == 3 && self.grid[y][x].is_live() {
                    // Any live cell with two or three live neightbours lives
                    // on to the next generation
                    CellState::Live
                } else if self.grid[y][x].live_neighbor_count(&self.grid) > 3 && self.grid[y][x].is_live() {
                    // Any live cell with more than three live neightbours
                    // dies, as if by overpopulation
                    CellState::Dead
                } else if self.grid[y][x].live_neighbor_count(&self.grid) == 3 && !self.grid[y][x].is_live() {
                    // Any dead cell with exactly three live neighbours 
                    // becomes a live cell, as if by reproduction
                    CellState::Live
                } else {
                    self.grid[y][x].state.clone()
                };
            }
        }

        self.grid = future;
    }
    /// Setup the cell neighbors while making the grid wrap
    fn set_cell_neighbors(&mut self) {
        let grid_rows = self.config.borrow().grid_rows.unwrap_or(GRID_ROWS) - 1;
        let grid_cols = self.config.borrow().grid_cols.unwrap_or(GRID_COLS) - 1;

        for (y, row) in self.grid.iter_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                if !cell.neighbors.is_empty() {
                    cell.neighbors.clear();
                }
                // Top Left
                cell.neighbors.push(
                    // Wrap to bottom right if overflow
                    Vec2::new(if x.overflowing_sub(1).1 {
                        grid_cols
                    } else { x - 1 }, if y.overflowing_sub(1).1 {
                        grid_rows
                    } else { y - 1 })
                );
                // Top
                cell.neighbors.push(
                    // Wrap y to the bottom y on the same x avis
                    Vec2::new(x, if y.overflowing_sub(1).1 {
                        grid_rows
                    } else { y - 1 })
                );
                // Top Right
                cell.neighbors.push(
                    // Wrap to bottom left
                    Vec2::new(if x+1 > grid_cols {
                        // All the way left on x
                        0
                    } else { x + 1 }, if y.overflowing_sub(1).1 {
                        grid_rows
                    } else { y - 1 })
                );
                // Left
                cell.neighbors.push(
                    // Wrap to right
                    Vec2::new(if x.overflowing_sub(1).1 {
                        grid_cols
                    } else { x - 1 }, y)
                );
                // Right
                cell.neighbors.push(
                    // Wrap to left
                    Vec2::new(if x+1 > grid_cols {
                        0
                    } else { x + 1 }, y)
                );
                // Bottom Left
                cell.neighbors.push(
                    // Wrap to top right
                    Vec2::new(if x.overflowing_sub(1).1 {
                        grid_cols
                    } else { x - 1 }, if y+1 > grid_rows {
                        0
                    } else { y + 1 })
                );
                // Bottom
                cell.neighbors.push(
                    // Wrap to top
                    Vec2::new(x, if y+1 > grid_rows {
                        0
                    } else { y + 1 })
                );
                // Bottom Right
                cell.neighbors.push(
                    // Wrap to top left
                    Vec2::new(if x+1 > grid_cols {
                        0
                    } else { x + 1 }, if y+1 > grid_rows {
                        0
                    } else { y + 1 })
                );
            }
        }
    }
    pub fn show(&self) {
        print!("\x1B[2J\x1B[1;1H");

        let mut grid = String::new();

        for row in &self.grid {
            for cell in row {
                if cell.is_live() {
                    grid.push(self.config.borrow().live_cell.unwrap_or('■'));
                } else {
                    grid.push(self.config.borrow().dead_cell.unwrap_or(' ')); // ▢
                }
            }

            grid.push('\n');
        }

        println!("GENERATION: {}\n\n{grid}", self.generation);
    }
}

fn main() {
    let config = Config::new("Config.toml").unwrap();
    let config = Rc::new(RefCell::new(config));
    let mut game = Game::new(Rc::clone(&config));

    loop {
        // If configuration was updated...
        if config.borrow_mut().refresh().unwrap() {
            game.set_cell_neighbors();
        }

        game.show();
        game.step();

        thread::sleep(Duration::from_millis(config.borrow().speed.unwrap_or(0)));
    }
}
