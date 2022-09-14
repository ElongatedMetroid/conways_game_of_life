use std::{fmt, thread, time::Duration, process};

struct Game {
    /// 100x150 grid of cells (for now)
    grid: Vec<Vec<Cell>>,
    generation: usize,
}

#[derive(PartialEq)]
enum CellState {
    Live,
    Dead,
}

struct Cell {
    state: CellState,
    neighbors: Vec<Box<Cell>>,
}

impl Cell {
    fn new(state: CellState) -> Cell {
        Cell { state, neighbors: Vec::new() }
    }
}

impl Game {
    pub fn new(seed: usize) -> Game {
        let mut game = Game { grid: Vec::new(), generation: 0 };

        for _ in 0..100 {
            let mut row = Vec::new();

            for _ in 0..150 {
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

        self.set_cell_neighbors();
    }
    fn set_cell_neighbors(&mut self) {
        
    }
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut grid = String::new();

        for row in &self.grid {
            for cell in row {
                if cell.state == CellState::Dead {
                    grid.push('▢');
                } else {
                    grid.push('■');
                }
            }

            grid.push('\n');
        }

        write!(f, "{grid}")
    }
}

fn main() {
    let mut game = Game::new(0);

    loop {
        game.step();

        thread::sleep(Duration::from_secs(3));

        println!("{game}");

        process::exit(0);
    }
}
