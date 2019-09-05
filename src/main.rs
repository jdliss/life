use ggez;
use rand;

use rand::Rng;

use ggez::event::{self, MouseButton, KeyCode, KeyMods};
use ggez::{graphics, Context, GameResult};
use std::time::{Duration, Instant};

const GRID_SIZE: (i16, i16) = (100, 100);
const GRID_CELL_SIZE: (i16, i16) = (10, 10);

const SCREEN_SIZE: (f32, f32) = (
    GRID_SIZE.0 as f32 * GRID_CELL_SIZE.0 as f32,
    GRID_SIZE.1 as f32 * GRID_CELL_SIZE.1 as f32,
    );

const UPDATES_PER_SECOND: f32 = 20.0;
const MILLIS_PER_UPDATE: u64 = (1.0 / UPDATES_PER_SECOND * 1000.0) as u64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct GridPosition {
    x: i16,
    y: i16,
}

impl GridPosition {
    pub fn new(x: i16, y: i16) -> Self {
        GridPosition { x, y }
    }
    pub fn random(max_x: i16, max_y: i16) -> Self {
        let mut rng = rand::thread_rng();
        (
            rng.gen_range::<i16, i16, i16>(0, max_x),
            rng.gen_range::<i16, i16, i16>(0, max_y),
            )
            .into()
    }
}

impl From<GridPosition> for graphics::Rect {
    fn from(pos: GridPosition) -> Self {
        graphics::Rect::new_i32(
            pos.x as i32 * GRID_CELL_SIZE.0 as i32,
            pos.y as i32 * GRID_CELL_SIZE.1 as i32,
            GRID_CELL_SIZE.0 as i32,
            GRID_CELL_SIZE.1 as i32,
            )
    }
}

impl From<(i16, i16)> for GridPosition {
    fn from(pos: (i16, i16)) -> Self {
        GridPosition { x: pos.0, y: pos.1 }
    }
}

#[derive(Clone, Debug)]
struct Cell {
    position: GridPosition,
    dead: bool
}

impl Cell {
    pub fn new(pos: GridPosition, dead: bool) -> Self {
        Cell {
            position: pos,
            dead: dead
        }
    }

    fn draw(&self, ctx: &mut Context) -> GameResult<()> {
        if !self.dead {
            let rectangle = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                self.position.into(),
                [1.0, 0.5, 0.0, 1.0].into(),
                )?;
            graphics::draw(ctx, &rectangle, (ggez::mint::Point2 { x: 0.0, y: 0.0 },))?;
            Ok(())
        } else {
            Ok(())
        }
    }
}

struct GameState {
    board: Vec<Vec<Cell>>,
    last_update: Instant,
    run: bool,
    reset_board: bool,
}

impl GameState {
    pub fn new(cell_count: i16) -> Self {
        let board = Self::generate_board(cell_count);

        GameState {
            board: board,
            last_update: Instant::now(),
            run: false,
            reset_board: false,
        }
    }

    fn generate_board(cell_count: i16) -> Vec<Vec<Cell>> {
        let mut board = vec![];

        // generate full grid of cells
        for x in 0..GRID_SIZE.0 {
            board.push( Vec::new());

            for y in 0..GRID_SIZE.1 {
                let cell_pos = GridPosition::new(x, y);
                let cell = Cell::new(cell_pos, true);
                board[x as usize].push(cell);
            }
        }

        let mut rng = rand::thread_rng();
        let mut random_positions = Vec::new();

        // get cell_count of random grid positions
        for _ in 0..cell_count {
            let random_pos = GridPosition::new(rng.gen_range(0, GRID_SIZE.0), rng.gen_range(0, GRID_SIZE.1));
            random_positions.push(random_pos);
        }

        // at these positions, set the cells to be alive (which will cause them to be displayed)
        for position in &random_positions {
            board[position.x as usize][position.y as usize].dead = false;
        }

        board
    }

    fn neighbor_count(board: &Vec<Vec<Cell>>, cell: &Cell) -> i16 {
        let mut neighbors = 0;

        let cell_x = cell.position.x as usize;
        let cell_y = cell.position.y as usize;

        if cell_x != 0 {
            // check left
            if !board[cell_x - 1][cell_y].dead {
                neighbors += 1;
            }

            if cell_y != 0 {
                // check top left
                if !board[cell_x - 1][cell_y - 1].dead {
                    neighbors += 1;
                }
            }

            if cell_y != GRID_SIZE.1 as usize - 1  {
                // check bottom left
                if !board[cell_x - 1][cell_y + 1].dead {
                    neighbors += 1;
                }
            }
        }

        if cell_x != GRID_SIZE.0 as usize - 1 {
            // check right
            if !board[cell_x + 1][cell_y].dead {
                neighbors += 1;
            }

            if cell_y != 0 {
                // check top right
                if !board[cell_x + 1][cell_y - 1].dead {
                    neighbors += 1;
                }
            }

            if cell_y != GRID_SIZE.1 as usize - 1  {
                // check bottom right
                if !board[cell_x + 1][cell_y + 1].dead {
                    neighbors += 1;
                }
            }
        }

        if cell_y != 0 {
            // check top
            if !board[cell_x][cell_y - 1].dead {
                neighbors += 1;
            }
        }

        if cell_y != GRID_SIZE.1 as usize - 1 {
            // check bottom
            if !board[cell_x][cell_y + 1].dead {
                neighbors += 1;
            }
        }

        neighbors
    }
}

impl event::EventHandler for GameState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if Instant::now() - self.last_update >= Duration::from_millis(MILLIS_PER_UPDATE) {
            if self.reset_board {
                for x in 0..GRID_SIZE.0 {
                    for y in 0..GRID_SIZE.1 {
                        self.board[x as usize][y as usize].dead = true;
                    }
                }

                self.reset_board = false;
            }

            if self.run {
                let board_copy = self.board.to_vec();

                for x in 0..GRID_SIZE.0 {
                    for y in 0..GRID_SIZE.1 {
                        let cell = self.board[x as usize][y as usize].clone();
                        let neighbors = Self::neighbor_count(&board_copy, &cell);

                        if cell.dead {
                            if neighbors == 3 {
                                self.board[x as usize][y as usize].dead = false;
                            }
                        } else {
                            if neighbors == 0 || neighbors == 1 || neighbors >= 4 {
                                self.board[x as usize][y as usize].dead = true;
                            }
                        }
                    }
                }
            }
            self.last_update = Instant::now();
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.439, 0.439, 0.439, 1.0].into());
        for vec in self.board.iter() {
            for cell in vec.iter() {
                cell.draw(ctx)?;
            }
        }

        graphics::present(ctx)?;
        ggez::timer::yield_now();
        Ok(())
    }

    fn key_down_event(&mut self, _ctx: &mut Context, keycode: KeyCode, _keymod: KeyMods, _repeat: bool) {
        match keycode {
            KeyCode::Space => {
                if self.run {
                    self.run = false;
                } else {
                    self.run = true;
                }
            },

            KeyCode::Back => {
                self.reset_board = true;
            },

            _ => println!("{:?} is not a valid command!", keycode)
        }
    }


    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, x: f32, y: f32) {
        let grid_x = x as i16 / GRID_CELL_SIZE.0;
        let grid_y = y as i16 / GRID_CELL_SIZE.1;
        let new_cell_pos = GridPosition::new(grid_x, grid_y);
        let cell = Cell::new(new_cell_pos, false);

        match self.board[grid_x as usize][grid_y as usize] {
            Cell { dead: true, .. } => self.board[grid_x as usize][grid_y as usize] = cell,
            Cell { dead: false, .. } => {
                self.board[grid_x as usize][grid_y as usize].dead = true;
            }
        }
    }
}

fn main() -> GameResult {
    let (ctx, events_loop) = &mut ggez::ContextBuilder::new("The Game of Life", "Jon Liss")
        .window_setup(ggez::conf::WindowSetup::default().title("The Game of Life"))
        .window_mode(ggez::conf::WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()?;

    let state = &mut GameState::new(0);
    event::run(ctx, events_loop, state)
}
