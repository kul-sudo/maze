#![recursion_limit = "256"]

use macroquad::prelude::{
    clear_background, draw_circle, draw_line, is_key_down, is_key_pressed, is_mouse_button_down,
    mouse_position, next_frame, screen_height, screen_width, set_fullscreen, u16vec2, vec2, Conf,
    KeyCode, MouseButton, U16Vec2, Vec2, BLACK, DARKGREEN, WHITE, YELLOW,
};

#[cfg(feature = "mode1")]
use rand::prelude::{IteratorRandom, SliceRandom};

#[cfg(feature = "mode2")]
use rand::prelude::SliceRandom;

use rand::{rngs::StdRng, thread_rng, Rng, SeedableRng};
use std::{
    collections::{HashSet, VecDeque},
    sync::LazyLock,
    time::Duration,
};

const DEFAULT_CELLS_AREA: f32 = 53.0 * 30.0;

const WALL_CHANGE_CHANCE: f32 = 1.0;

const CELLS_ROWS: usize = 40;

static CELLS_COLUMNS: LazyLock<usize> =
    LazyLock::new(|| (CELLS_ROWS as f32 * (screen_width() / screen_height())) as usize);

static CELL_SIZE: LazyLock<Vec2> = LazyLock::new(|| {
    vec2(
        screen_width() / *CELLS_COLUMNS as f32,
        screen_height() / CELLS_ROWS as f32,
    )
});

static CELLS_AREA: LazyLock<f32> = LazyLock::new(|| *CELLS_COLUMNS as f32 * CELLS_ROWS as f32);

const WALL_WIDTH_CONST: f32 = 5.0;
static WALL_WIDTH: LazyLock<f32> =
    LazyLock::new(|| WALL_WIDTH_CONST / (*CELLS_AREA / DEFAULT_CELLS_AREA).powf(1.0 / 4.0));

#[derive(Clone, Default, Debug)]
struct Cell {
    visited: bool,
    /// Clockwise: North, East, South, West.
    walls: [bool; 4],
}

#[derive(Clone, Debug)]
struct Cells {
    cells: Vec<Vec<Cell>>,
}

struct Player {
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

    // Mode 1.
    #[cfg(feature = "mode1")]
    /// The function adds a new wall between 2 random cells.
    fn add_wall(&mut self, rng: &mut StdRng) -> (U16Vec2, U16Vec2) {
        let random_pos = u16vec2(
            rng.gen_range(0..*CELLS_COLUMNS as u16),
            rng.gen_range(0..CELLS_ROWS as u16),
        );

        let neighbors = self.get_neighbors(random_pos);
        let filtered_neighbors = neighbors.iter().filter(|neighbor| {
            !self.cells[random_pos.y as usize][random_pos.x as usize].walls
                [Cells::wall_from_pos(random_pos, **neighbor)]
        });

        let random_neighbor = *filtered_neighbors.choose(rng).unwrap();

        // Add the wall
        let (lhs_wall, rhs_wall) = Cells::mutual_walls_from_pos(random_pos, random_neighbor);
        self.cells[random_pos.y as usize][random_pos.x as usize].walls[lhs_wall] = true;
        self.cells[random_neighbor.y as usize][random_neighbor.x as usize].walls[rhs_wall] = true;

        (random_pos, random_neighbor)
    }

    #[cfg(feature = "mode1")]
    /// The function collects the neighbors on the border of a self.lake().
    fn collect_lake_shore(
        &self,
        new_wall: &(U16Vec2, U16Vec2),
        rng: &mut StdRng,
    ) -> HashSet<(U16Vec2, U16Vec2)> {
        let mut lake_shore = HashSet::new();

        let lake = self.lake(rng);

        for pos in lake.iter() {
            let neighbors = self.get_neighbors(*pos);

            for neighbor in neighbors {
                if *new_wall != (*pos, neighbor) && !lake.contains(&neighbor) {
                    lake_shore.insert((*pos, neighbor));
                }
            }
        }

        lake_shore
    }

    #[cfg(feature = "mode1")]
    /// The function gets the cells of one of the 2 possible lakes of a non-perfect maze.
    /// It doesn't matter which one to get.
    fn lake(&self, rng: &mut StdRng) -> HashSet<U16Vec2> {
        let mut lake_cells = HashSet::new();

        self.lake_recursion(
            u16vec2(
                rng.gen_range(0..*CELLS_COLUMNS as u16),
                rng.gen_range(0..CELLS_ROWS as u16),
            ),
            &mut lake_cells,
        );

        lake_cells
    }

    #[cfg(feature = "mode1")]
    fn lake_recursion(&self, pos: U16Vec2, lake_cells: &mut HashSet<U16Vec2>) {
        if lake_cells.contains(&pos) {
            return;
        }

        lake_cells.insert(pos);

        for neighbor in self.get_neighbors(pos) {
            if self.cells[pos.y as usize][pos.x as usize].walls[Cells::wall_from_pos(pos, neighbor)]
            {
                continue;
            }

            self.lake_recursion(neighbor, lake_cells);
        }
    }

    #[cfg(feature = "mode1")]
    /// The function combines all the operations needed for changing a wall.
    fn change_wall(&mut self, rng: &mut StdRng) {
        let new_wall = self.add_wall(rng);
        let shore = self.collect_lake_shore(&new_wall, rng);

        let (pos, neighbor) = shore.iter().choose(rng).unwrap();

        let (lhs_wall, rhs_wall) = Cells::mutual_walls_from_pos(*pos, *neighbor);
        self.cells[pos.y as usize][pos.x as usize].walls[lhs_wall] = false;
        self.cells[neighbor.y as usize][neighbor.x as usize].walls[rhs_wall] = false;
    }

    fn wall_from_pos(lhs: U16Vec2, rhs: U16Vec2) -> usize {
        2 * ((lhs.y < rhs.y) as usize + (lhs.x > rhs.x) as usize) + (lhs.x != rhs.x) as usize
    }

    fn mutual_walls_from_pos(lhs: U16Vec2, rhs: U16Vec2) -> (usize, usize) {
        let lhs_wall = Cells::wall_from_pos(lhs, rhs);

        (lhs_wall, (lhs_wall + 2) % 4)
    }

    /// The function builds the maze.
    fn build(&mut self, pos: U16Vec2, rng: &mut StdRng) {
        self.cells[pos.y as usize][pos.x as usize].visited = true;

        let mut neighbors = self.get_neighbors(pos);
        neighbors.shuffle(rng);

        for neighbor in neighbors {
            if self.cells[neighbor.y as usize][neighbor.x as usize].visited {
                continue;
            }

            let (lhs_wall, rhs_wall) = Cells::mutual_walls_from_pos(pos, neighbor);
            self.cells[pos.y as usize][pos.x as usize].walls[lhs_wall] = false;
            self.cells[neighbor.y as usize][neighbor.x as usize].walls[rhs_wall] = false;

            self.build(neighbor, rng);
        }
    }

    fn get_path(&self, lhs: U16Vec2, rhs: U16Vec2) -> VecDeque<U16Vec2> {
        let mut have_been_here = HashSet::new();

        self.recursive_get_path(lhs, rhs, &mut have_been_here)
            .unwrap()
    }

    fn recursive_get_path(
        &self,
        lhs: U16Vec2,
        rhs: U16Vec2,
        have_been_here: &mut HashSet<U16Vec2>,
    ) -> Option<VecDeque<U16Vec2>> {
        if lhs == rhs {
            return Some(VecDeque::from([lhs]));
        }

        if have_been_here.contains(&lhs) {
            return None;
        }

        have_been_here.insert(lhs);

        for neighbor in self.get_neighbors(lhs) {
            if !self.cells[lhs.y as usize][lhs.x as usize].walls
                [Cells::wall_from_pos(lhs, neighbor)]
            {
                if let Some(mut path) = self.recursive_get_path(neighbor, rhs, have_been_here) {
                    path.push_front(lhs);

                    return Some(path);
                }
            }
        }

        None
    }

    // Mode 2
    #[cfg(feature = "mode2")]
    /// The function adds a new wall between 2 random cells.
    fn change_wall(&mut self, rng: &mut StdRng) {
        loop {
            let random_pos = u16vec2(
                rng.gen_range(0..*CELLS_COLUMNS as u16),
                rng.gen_range(0..CELLS_ROWS as u16),
            );

            let neighbors = self.get_neighbors(random_pos);
            let filtered_neighbors = neighbors
                .iter()
                .filter(|neighbor| {
                    self.cells[random_pos.y as usize][random_pos.x as usize].walls
                        [Cells::wall_from_pos(random_pos, **neighbor)]
                })
                .collect::<Vec<_>>();

            if filtered_neighbors.is_empty() {
                continue;
            }

            let random_neighbor = *filtered_neighbors.choose(rng).unwrap();

            let path = self.get_path(random_pos, *random_neighbor);

            {
                let (lhs_wall, rhs_wall) =
                    Cells::mutual_walls_from_pos(random_pos, *random_neighbor);
                self.cells[random_pos.y as usize][random_pos.x as usize].walls[lhs_wall] = false;
                self.cells[random_neighbor.y as usize][random_neighbor.x as usize].walls
                    [rhs_wall] = false;
            }

            let lhs = rng.gen_range(0..path.len() - 1);

            let lhs_cell = path[lhs];
            let rhs_cell = path[lhs + 1];

            {
                let (lhs_wall, rhs_wall) = Cells::mutual_walls_from_pos(lhs_cell, rhs_cell);
                self.cells[lhs_cell.y as usize][lhs_cell.x as usize].walls[lhs_wall] = true;
                self.cells[rhs_cell.y as usize][rhs_cell.x as usize].walls[rhs_wall] = true;
            }

            break;
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

    // Rng
    let mut rng = StdRng::from_rng(&mut thread_rng()).unwrap();

    // Player
    let mut player = Player { pos: U16Vec2::ZERO };

    // Cells
    let mut cells = Cells {
        cells: vec![
            vec![
                Cell {
                    visited: false,
                    walls: [true; 4]
                };
                *CELLS_COLUMNS
            ];
            CELLS_ROWS
        ],
    };

    cells.build(U16Vec2::ZERO, &mut rng);

    // Path
    let mut path = cells.get_path(
        player.pos,
        u16vec2(*CELLS_COLUMNS as u16 - 1, CELLS_ROWS as u16 - 1),
    );

    loop {
        // Interactions
        if (is_key_down(KeyCode::W) || is_key_down(KeyCode::Up))
            && player.pos.y > 0
            && !cells.cells[player.pos.y as usize][player.pos.x as usize].walls[0]
        {
            player.pos.y -= 1;
        }

        if (is_key_down(KeyCode::D) || is_key_down(KeyCode::Right))
            && player.pos.x < *CELLS_COLUMNS as u16 - 1
            && !cells.cells[player.pos.y as usize][player.pos.x as usize].walls[1]
        {
            player.pos.x += 1;
        }

        if (is_key_down(KeyCode::S) || is_key_down(KeyCode::Down))
            && player.pos.y < CELLS_ROWS as u16 - 1
            && !cells.cells[player.pos.y as usize][player.pos.x as usize].walls[2]
        {
            player.pos.y += 1;
        }

        if (is_key_down(KeyCode::A) || is_key_down(KeyCode::Left))
            && player.pos.x > 0
            && !cells.cells[player.pos.y as usize][player.pos.x as usize].walls[3]
        {
            player.pos.x -= 1;
        }

        if is_key_pressed(KeyCode::Space) {
            cells = Cells {
                cells: vec![
                    vec![
                        Cell {
                            visited: false,
                            walls: [true; 4]
                        };
                        *CELLS_COLUMNS
                    ];
                    CELLS_ROWS
                ],
            };

            cells.build(U16Vec2::ZERO, &mut rng);

            path = cells.get_path(
                player.pos,
                u16vec2(*CELLS_COLUMNS as u16 - 1, CELLS_ROWS as u16 - 1),
            );
        }

        if is_mouse_button_down(MouseButton::Left) {
            let (x, y) = mouse_position();

            player.pos = u16vec2((x / CELL_SIZE.x) as u16, (y / CELL_SIZE.y) as u16);
        }

        for (pos_index, pos) in path.iter().enumerate() {
            if let Some(next_pos) = path.get(pos_index + 1) {
                draw_line(
                    pos.x as f32 * CELL_SIZE.x + CELL_SIZE.x / 2.0,
                    pos.y as f32 * CELL_SIZE.y + CELL_SIZE.y / 2.0,
                    next_pos.x as f32 * CELL_SIZE.x + CELL_SIZE.x / 2.0,
                    next_pos.y as f32 * CELL_SIZE.y + CELL_SIZE.y / 2.0,
                    *WALL_WIDTH,
                    WHITE,
                )
            }
        }

        // Drawing
        for i in 0..CELLS_ROWS {
            for j in 0..*CELLS_COLUMNS {
                let cell = &mut cells.cells[i][j];

                if cell.walls[0] {
                    draw_line(
                        j as f32 * CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y,
                        j as f32 * CELL_SIZE.x + CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y,
                        *WALL_WIDTH,
                        DARKGREEN,
                    );
                }

                if cell.walls[1] {
                    draw_line(
                        j as f32 * CELL_SIZE.x + CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y,
                        j as f32 * CELL_SIZE.x + CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y + CELL_SIZE.y,
                        *WALL_WIDTH,
                        DARKGREEN,
                    );
                }

                if cell.walls[2] {
                    draw_line(
                        j as f32 * CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y + CELL_SIZE.y,
                        j as f32 * CELL_SIZE.x + CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y + CELL_SIZE.y,
                        *WALL_WIDTH,
                        DARKGREEN,
                    );
                }

                if cell.walls[3] {
                    draw_line(
                        j as f32 * CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y,
                        j as f32 * CELL_SIZE.x,
                        i as f32 * CELL_SIZE.y + CELL_SIZE.y,
                        *WALL_WIDTH,
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

        next_frame().await;

        clear_background(BLACK);

        // Change walls in needed
        if WALL_CHANGE_CHANCE > 0.0
            && (WALL_CHANGE_CHANCE == 1.0 || rng.gen_range(0.0..1.0) <= WALL_CHANGE_CHANCE)
        {
            cells.change_wall(&mut rng);

            path = cells.get_path(
                player.pos,
                u16vec2(*CELLS_COLUMNS as u16 - 1, CELLS_ROWS as u16 - 1),
            );
        }
    }
}
