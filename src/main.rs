use std::{thread, time::Duration};

struct Game {
    grid: Vec<Vec<Cell>>,
    generation: usize,
}

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
        let game = Game { grid: Vec::new(), generation: 0 };

        // For now the grid will just be 100x100x
        game.grid[50][50] = Cell::new(CellState:Lsive);

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
}

fn main() {
    let mut game = Game::new(0);

    loop {
        game.step();

        thread::sleep(Duration::from_secs(3));

        println!("{game}");
    }
}
