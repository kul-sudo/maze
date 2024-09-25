use ::rand::{prelude::*, thread_rng};
use macroquad::prelude::*;
use std::{
    sync::{LazyLock},
    time::Duration,
};

const CELLS_ROWS: usize = 50;

static CELLS_COLUMNS: LazyLock<usize> =
    LazyLock::new(|| (CELLS_ROWS as f32 * (screen_width() / screen_height())) as usize);

static CELL_SIZE: LazyLock<Vec2> = LazyLock::new(|| vec2(screen_width() / *CELLS_COLUMNS as f32, screen_height() / CELLS_ROWS as f32));

#[derive(Clone, Default, Debug)]
struct Cell {
    visited: bool,
    walls: BVec4,
}

#[derive(Clone, Debug)]
struct Cells {
    cells: Vec<Vec<Cell>>,
}

fn window_conf() -> Conf {
    Conf {
        window_title: "maze".to_owned(),
        fullscreen: true,
        ..Default::default()
    }
}

impl Cells {
    fn get_neighbors(&self, pos: U16Vec2) -> Vec<U16Vec2> {
        let mut neighbors = Vec::with_capacity(4);

        if pos.y > 0 {
            neighbors.push(U16Vec2::new(pos.x, pos.y - 1));
        }

        if pos.y < CELLS_ROWS as u16 - 1 {
            neighbors.push(U16Vec2::new(pos.x, pos.y + 1));
        }

        if pos.x > 0 {
            neighbors.push(U16Vec2::new(pos.x - 1, pos.y));
        }

        if pos.x < *CELLS_COLUMNS as u16 - 1 {
            neighbors.push(U16Vec2::new(pos.x + 1, pos.y));
        }

        neighbors
    }

    fn build(&mut self, pos: U16Vec2) {
        self.cells[pos.y as usize][pos.x as usize].visited = true;

        let mut neighbors = self.get_neighbors(pos);
        neighbors.shuffle(&mut thread_rng());

        for neighbor in neighbors {
            if self.cells[neighbor.y as usize][neighbor.x as usize].visited {
                continue;
            }

            if neighbor.y > pos.y {
                self.cells[neighbor.y as usize][neighbor.x as usize].walls.w = false;
                self.cells[pos.y as usize][pos.x as usize].walls.z = false;
            } else if neighbor.y < pos.y {
                self.cells[neighbor.y as usize][neighbor.x as usize].walls.z = false;
                self.cells[pos.y as usize][pos.x as usize].walls.w = false;
            } else if neighbor.x > pos.x {
                self.cells[neighbor.y as usize][neighbor.x as usize].walls.x = false;
                self.cells[pos.y as usize][pos.x as usize].walls.y = false;
            } else if neighbor.x < pos.x {
                self.cells[neighbor.y as usize][neighbor.x as usize].walls.y = false;
                self.cells[pos.y as usize][pos.x as usize].walls.x = false;
            }

            self.build(neighbor);
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    if cfg!(target_os = "linux") {
        set_fullscreen(true);
        std::thread::sleep(Duration::from_secs(1));
        next_frame().await;
    }

    let mut cells = Cells {
        cells: vec![vec![Cell { visited: false, walls: BVec4::TRUE }; *CELLS_COLUMNS]; CELLS_ROWS],
    };

    cells.build(U16Vec2::new(0, 0));

    loop {
        if is_key_pressed(KeyCode::Space) {
            cells = Cells {
                cells: vec![vec![Cell { visited: false, walls: BVec4::TRUE }; *CELLS_COLUMNS]; CELLS_ROWS],
            };

            cells.build(U16Vec2::new(0, 0));
        }

        for i in 0..cells.cells.len() {
            for j in 0..cells.cells[i].len() {
                let cell = &mut cells.cells[i][j];

                if cell.walls.x {
                    draw_line(
                        j as f32 * CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y,
                        j as f32 * CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y + CELL_SIZE.y,
                        5.0,
                        BLACK,
                    );
                }

                if cell.walls.y {
                    draw_line(
                        j as f32 * CELL_SIZE.x + CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y,
                        j as f32 * CELL_SIZE.x + CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y + CELL_SIZE.y,
                        5.0,
                        BLACK,
                    );
                }

                if cell.walls.w {
                    draw_line(
                        j as f32 * CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y,
                        j as f32 * CELL_SIZE.x + CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y,
                        5.0,
                        BLACK,
                    );
                }

                if cell.walls.z {
                    draw_line(
                        j as f32 * CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y + CELL_SIZE.y,
                        j as f32 * CELL_SIZE.x + CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y + CELL_SIZE.y,
                        5.0,
                        BLACK,
                    );
                }
            }
        }

        next_frame().await;

        clear_background(DARKBLUE);
    }
}
