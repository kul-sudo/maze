use macroquad::prelude::{
    clear_background, draw_circle, draw_line, is_key_down, is_key_pressed, is_mouse_button_down,
    mouse_position, next_frame, screen_height, screen_width, set_fullscreen, u16vec2, vec2, BVec4,
    Conf, KeyCode, MouseButton, U16Vec2, Vec2, BLACK, DARKGREEN, ORANGE, YELLOW,
};
use rand::prelude::SliceRandom;
use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};
use std::{sync::LazyLock, time::Duration};

const CELLS_ROWS: usize = 30;

static CELLS_COLUMNS: LazyLock<usize> =
    LazyLock::new(|| (CELLS_ROWS as f32 * (screen_width() / screen_height())) as usize);

static CELL_SIZE: LazyLock<Vec2> = LazyLock::new(|| {
    vec2(
        screen_width() / *CELLS_COLUMNS as f32,
        screen_height() / CELLS_ROWS as f32,
    )
});

const COINS_N: usize = 10;

#[derive(Clone, Default, Debug)]
struct Cell {
    visited: bool,
    walls: BVec4,
}

#[derive(Clone, Debug)]
struct Cells {
    cells: Vec<Vec<Cell>>,
}

struct Player {
    pos: U16Vec2,
}

#[derive(Clone, Debug)]
struct Coin {
    pos: U16Vec2,
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
            neighbors.push(u16vec2(pos.x, pos.y - 1));
        }

        if pos.y < CELLS_ROWS as u16 - 1 {
            neighbors.push(u16vec2(pos.x, pos.y + 1));
        }

        if pos.x > 0 {
            neighbors.push(u16vec2(pos.x - 1, pos.y));
        }

        if pos.x < *CELLS_COLUMNS as u16 - 1 {
            neighbors.push(u16vec2(pos.x + 1, pos.y));
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
            } else {
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

    let mut rng = StdRng::from_rng(&mut thread_rng()).unwrap();

    let mut player = Player { pos: U16Vec2::ZERO };
    let mut coins = Vec::with_capacity(COINS_N);

    for _ in 0..COINS_N {
        coins.push(Coin {
            pos: u16vec2(
                rng.gen_range(0..*CELLS_COLUMNS as u16 - 1),
                rng.gen_range(0..CELLS_ROWS as u16 - 1),
            ),
        });
    }

    let mut cells = Cells {
        cells: vec![
            vec![
                Cell {
                    visited: false,
                    walls: BVec4::TRUE
                };
                *CELLS_COLUMNS
            ];
            CELLS_ROWS
        ],
    };

    cells.build(U16Vec2::new(0, 0));

    loop {
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            if player.pos.x > 0
                && !cells.cells[player.pos.y as usize][player.pos.x as usize]
                    .walls
                    .x
            {
                player.pos.x -= 1;
            }
        }

        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            if player.pos.x < *CELLS_COLUMNS as u16 - 1
                && !cells.cells[player.pos.y as usize][player.pos.x as usize]
                    .walls
                    .y
            {
                player.pos.x += 1;
            }
        }

        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            if player.pos.y > 0
                && !cells.cells[player.pos.y as usize][player.pos.x as usize]
                    .walls
                    .w
            {
                player.pos.y -= 1;
            }
        }

        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            if player.pos.y < CELLS_ROWS as u16 - 1
                && !cells.cells[player.pos.y as usize][player.pos.x as usize]
                    .walls
                    .z
            {
                player.pos.y += 1;
            }
        }

        if is_key_pressed(KeyCode::Space) {
            cells = Cells {
                cells: vec![
                    vec![
                        Cell {
                            visited: false,
                            walls: BVec4::TRUE
                        };
                        *CELLS_COLUMNS
                    ];
                    CELLS_ROWS
                ],
            };

            cells.build(U16Vec2::ZERO);
        }

        if is_mouse_button_down(MouseButton::Left) {
            let (x, y) = mouse_position();

            player.pos = u16vec2(
                (x / CELL_SIZE.x as f32) as u16,
                (y / CELL_SIZE.y as f32) as u16,
            );
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
                        DARKGREEN,
                    );
                }

                if cell.walls.y {
                    draw_line(
                        j as f32 * CELL_SIZE.x + CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y,
                        j as f32 * CELL_SIZE.x + CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y + CELL_SIZE.y,
                        5.0,
                        DARKGREEN,
                    );
                }

                if cell.walls.w {
                    draw_line(
                        j as f32 * CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y,
                        j as f32 * CELL_SIZE.x + CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y,
                        5.0,
                        DARKGREEN,
                    );
                }

                if cell.walls.z {
                    draw_line(
                        j as f32 * CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y + CELL_SIZE.y,
                        j as f32 * CELL_SIZE.x + CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y + CELL_SIZE.y,
                        5.0,
                        DARKGREEN,
                    );
                }
            }
        }

        draw_circle(
            player.pos.x as f32 * CELL_SIZE.x + CELL_SIZE.x / 2.0,
            player.pos.y as f32 * CELL_SIZE.y + CELL_SIZE.y / 2.0,
            CELL_SIZE.x * 0.4,
            YELLOW,
        );

        for coin in &coins {
            draw_circle(
                coin.pos.x as f32 * CELL_SIZE.x + CELL_SIZE.x / 2.0,
                coin.pos.y as f32 * CELL_SIZE.y + CELL_SIZE.y / 2.0,
                CELL_SIZE.x * 0.4,
                ORANGE,
            );

            draw_circle(
                coin.pos.x as f32 * CELL_SIZE.x + CELL_SIZE.x / 2.0,
                coin.pos.y as f32 * CELL_SIZE.y + CELL_SIZE.y / 2.0,
                CELL_SIZE.x * 0.2,
                YELLOW,
            );
        }

        next_frame().await;

        clear_background(BLACK);
    }
}
