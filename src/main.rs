use std::{thread, time::Duration, sync::{mpsc, Arc, Mutex}, process, io::{self, Write}, fs::File};
use crossterm::{terminal::{ClearType, self}, event::{self, Event, KeyEvent, KeyCode}, cursor};
use rand::Rng;

mod config;

use config::Config;
use serde_derive::Serialize;

/// Height, Y (Default)
const GRID_ROWS: usize = 50;
/// Width, X (Default)
const GRID_COLS: usize = 50;

#[derive(Debug)]
struct Game {
    /// 100x150 grid of cells (for now)
    grid: Vec<Vec<Cell>>,
    generation: usize,

    config: Arc<Mutex<Config>>,

    seed: usize,
    numbers_added: Vec<usize>,
}

#[derive(PartialEq, Eq, Debug, Clone)]
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
    pub fn live_neighbor_count(&self, grid: &[Vec<Cell>]) -> usize {
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
    /// Create a new Game with the specified Config
    pub fn new(config: Arc<Mutex<Config>>) -> Game {
        // Create an empty instance of a Game
        let mut game = Game { grid: Vec::new(), generation: 0, config, seed: 0, numbers_added: Vec::new() };

        // Cache the values of grid_rows and grid_cols
        let grid_rows = game.config.lock().unwrap().grid_rows.unwrap_or(GRID_ROWS);
        let grid_cols = game.config.lock().unwrap().grid_cols.unwrap_or(GRID_COLS);

        // ----- Initialize Grid Vector -----
        // Foreach row...
        for _ in 0..grid_rows {
            // Create a vector that will hold the row...
            let mut row = Vec::new();

            // Foreach column...
            for _ in 0..grid_cols {
                // Push a new dead cell to the row
                row.push(Cell::new(CellState::Dead));
            }

            // Push the row to the grid
            game.grid.push(row);
        }

        // Saving so the user can later view what seed was used
        game.seed = game.config.lock().unwrap().seed.unwrap_or_else( || rand::thread_rng().gen());
        // Foreach row...
        for row in &mut game.grid {
            // Foreach cell in the row...
            for cell in row {
                // Generate a random usize
                let random = rand::thread_rng().gen::<usize>();
                // Push it to our vector of numbers (remember this vector is only so the results can be replicated)
                game.numbers_added.push(random);
                // Create a usage seed variable witch will be the seed + random
                let usage_seed = game.seed.overflowing_add(random).0;
                
                // Set the cell to live if the seed is even
                *cell = if (usage_seed % 2) == 0 {
                    Cell::new(CellState::Live)
                } else {
                    Cell::new(CellState::Dead)
                };
            }
        }

        // ----- Write Seed and Numbers Added to File -----

        // Set each cells neighbors
        game.set_cell_neighbors();

        game
    }
    /// Setup the cell neighbors while making the grid wrap
    fn set_cell_neighbors(&mut self) {
        // Cache the grid_rows and grid_cols values
        let grid_rows = self.config.lock().unwrap().grid_rows.unwrap_or(GRID_ROWS) - 1;
        let grid_cols = self.config.lock().unwrap().grid_cols.unwrap_or(GRID_COLS) - 1;

        // Foreach row
        for (y, row) in self.grid.iter_mut().enumerate() {
            // Foreach cell in row
            for (x, cell) in row.iter_mut().enumerate() {
                // Guard for potential feature: Extendable and shrinkable grids during runtime
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
    /// Apply the rules to the current grid
    pub fn step(&mut self) {
        // Increment generation
        self.generation += 1;
        // Create a grid that will hold the future (this is because the rules have to be applied to all cells simultainiously, and if we 
        // modified grid while applying the rules that would not work, so instead we apply the result of the rules to future and keep grid
        // unmodified)
        let mut future = self.grid.clone();

        let grid_rows = self.config.lock().unwrap().grid_rows.unwrap_or(GRID_ROWS);
        let grid_cols = self.config.lock().unwrap().grid_cols.unwrap_or(GRID_COLS);
            
        // Foreach row
        for y in 0..grid_rows {
            // Foreach column
            for x in 0..grid_cols {
                // Set the futures cell state to the presents current cell with the rules applied
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

        // Set the grid to the future
        self.grid = future;
    }
    /// Print the grid
    pub fn show(&self) {
        // Clear the terminal
        println!("{}{}Generation: {}", terminal::Clear(ClearType::All), cursor::MoveTo(1, 1), self.generation);
        io::stdout().flush().unwrap();

        for row in &self.grid {
            for cell in row {
                if cell.is_live() {
                    print!("{}", self.config.lock().unwrap().live_cell.unwrap_or('■'));
                } else {
                    print!("{}", self.config.lock().unwrap().dead_cell.unwrap_or(' '));
                }
            }

            println!();
        }

        io::stdout().flush().unwrap();
    }

    pub fn save_seed(&self, file_name: &str) {
        #[derive(Serialize)]
        struct SeedSave {
            seed: usize,
            numbers_added: Vec<usize>,
        }

        let mut f = File::create(file_name).unwrap();

        let seed_save = SeedSave {
            seed: self.seed,
            numbers_added: self.numbers_added.clone(),
        };

        let toml = toml::to_string(&seed_save).unwrap();

        f.write_all(toml.as_bytes()).unwrap();
    }
    pub fn save_state(&self, file_name: &str) {

    }
}

enum Command {
    Pause, 
    Unpause,
    // Save seed to file name
    SaveSeed(String),
    // Save state to file name
    SaveCurrentState(String),
    IncGen,
    DecGen,
}

fn main() {
    let config = Arc::new(Mutex::new(Config::new("Config.toml").unwrap()));
    let mut game = Game::new(Arc::clone(&config));

    let (tx, rx) = mpsc::channel();

    let game_thread = thread::spawn(move || {
        let mut paused = false;

        loop {
            match rx.try_recv() {
                Ok(command) => {
                    match command {
                        Command::Pause => paused = true,
                        Command::Unpause => paused = false,
                        Command::SaveSeed(file_name) => game.save_seed(file_name.as_str()),
                        Command::SaveCurrentState(file_name) => game.save_state(file_name.as_str()),
                        Command::IncGen => {
                            paused = true;
                            game.step();
                            game.show();
                        },
                        Command::DecGen => {
                            paused = true;
                            //game.step_back();
                            game.show();
                        },
                    }
                }
                Err(_) => (),
            }

            if !paused {
                config.lock().unwrap().refresh().unwrap();

                game.show();
                game.step();
                
                thread::sleep(Duration::from_millis(config.lock().unwrap().speed.unwrap_or(0)));
            }
        }
    });

    let input_thread = thread::spawn(move || {
        //let _raw = terminal::enable_raw_mode().unwrap();
        
        let read_additional_data = || {
            let mut buf = String::new();
            io::stdin().read_line(&mut buf).unwrap();
            buf
        };

        loop {
            match event::read().unwrap() {
                Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) => process::exit(0),
                Event::Key(KeyEvent { code: KeyCode::Char('p'), .. }) => tx.send(Command::Pause).unwrap(),
                Event::Key(KeyEvent { code: KeyCode::Char('o'), .. }) => tx.send(Command::Unpause).unwrap(),
                Event::Key(KeyEvent { code: KeyCode::Char('s'), .. }) => {
                    tx.send(Command::SaveSeed(read_additional_data())).unwrap();
                }
                Event::Key(KeyEvent { code: KeyCode::Char('c'), .. }) => {
                    tx.send(Command::SaveCurrentState(read_additional_data())).unwrap();
                }
                Event::Key(KeyEvent { code: KeyCode::Left, .. }) => tx.send(Command::DecGen).unwrap(),
                Event::Key(KeyEvent { code: KeyCode::Right, .. }) => tx.send(Command::IncGen).unwrap(),
                _ => (),
            }
        }
    });

    game_thread.join().unwrap();
    input_thread.join().unwrap();
}