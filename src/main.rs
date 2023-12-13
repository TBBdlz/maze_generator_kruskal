extern crate rand;
extern crate clap;

use rand::{Rng, seq::SliceRandom};
use std::collections::HashSet;
use std::fs::File;
use std::io::{self, Write};
use clap::{App, Arg};

type Cell = (usize, usize);
type Wall = (Cell, Cell);

struct Maze {
    width: usize,
    height: usize,
    walls: HashSet<Wall>,
    stickiness: Vec<Vec<u8>>,
    open_walls: HashSet<Wall>,
}

impl Maze {
    fn new(width: usize, height: usize) -> Self {
        let mut walls = HashSet::new();
        let mut rng = rand::thread_rng();
        let mut stickiness = vec![vec![0; width + 2]; height + 2];

        for x in 0..width + 2 {
            for y in 0..height + 2 {
                if x == 0 || y == 0 || x == width + 1 || y == height + 1 {
                    stickiness[y][x] = 0; // Outer walls
                } else {
                    if x < width + 1 {
                        walls.insert(((x, y), (x + 1, y)));
                    }
                    if y < height + 1 {
                        walls.insert(((x, y), (x, y + 1)));
                    }
                    stickiness[y][x] = rng.gen_range(1..=9); // Inner cells
                }
            }
        }

        Maze { width, height, walls, stickiness, open_walls: HashSet::new() }
    }

    fn generate(&mut self) {
        let mut sets = UnionFind::new((self.width + 2) * (self.height + 2));
        let mut wall_list: Vec<Wall> = self.walls.iter().cloned().collect();
        let mut rng = rand::thread_rng();

        wall_list.shuffle(&mut rng);

        for wall in wall_list {
            let (cell1, cell2) = wall;

            // Skip if it's an outer wall
            if cell1.0 == 0 || cell1.1 == 0 || cell2.0 == self.width + 1 || cell2.1 == self.height + 1 {
                continue;
            }

            let set1 = sets.find(self.cell_to_id(cell1));
            let set2 = sets.find(self.cell_to_id(cell2));

            if set1 != set2 {
                self.open_walls.insert(wall);
                sets.union(set1, set2);
            }
        }
    }

    fn cell_to_id(&self, cell: Cell) -> usize {
        cell.0 + cell.1 * (self.width + 2)
    }

    fn add_map(&mut self) {
        let mut rng = rand::thread_rng();

        // Collect non-wall cell coordinates separately
        let mut non_wall_cells: Vec<Cell> = Vec::new();
        for y in 1..=self.height {
            for x in 1..=self.width {
                if self.stickiness[y][x] != 0 {
                    non_wall_cells.push((x, y));
                }
            }
        }

        // Shuffle and select positions for 'S' and 'G'
        non_wall_cells.shuffle(&mut rng);

        if non_wall_cells.len() >= 1 {
            if let Some((start_x, start_y)) = non_wall_cells.pop() {
                self.stickiness[start_y][start_x] = b'S';
            }
        }

        if non_wall_cells.len() >= 1 {
            if let Some((goal_x, goal_y)) = non_wall_cells.pop() {
                self.stickiness[goal_y][goal_x] = b'G';
            }
        }

        if non_wall_cells.len() < 2 {
            eprintln!("Warning: Only one non-wall cell available. Only 'S' or 'G' was placed.");
        }
    }
}

struct UnionFind {
    parent: Vec<usize>,
}

impl UnionFind {
    fn new(size: usize) -> Self {
        UnionFind {
            parent: (0..size).collect(),
        }
    }

    fn find(&mut self, mut node: usize) -> usize {
        while node != self.parent[node] {
            node = self.parent[node];
        }
        node
    }

    fn union(&mut self, a: usize, b: usize) {
        let root_a = self.find(a);
        let root_b = self.find(b);
        if root_a != root_b {
            self.parent[root_a] = root_b;
        }
    }
}

fn print_maze(maze: &Maze) {
    for y in 0..maze.height + 2 {
        for x in 0..maze.width + 2 {
            if maze.stickiness[y][x] == 0 || (!maze.open_walls.contains(&((x, y), (x + 1, y))) && !maze.open_walls.contains(&((x, y), (x, y + 1)))) {
                print!("X");
            } else {
                print!("{}", if maze.stickiness[y][x] == b'S' || maze.stickiness[y][x] == b'G' {
                    maze.stickiness[y][x] as char
                } else {
                    char::from_digit(maze.stickiness[y][x] as u32, 10).unwrap()
                });
            }
        }
        println!();
    }
}

fn save_to_file(maze: &Maze, file_name: &str) -> io::Result<()> {
    let mut file = File::create(file_name)?;

    for y in 0..maze.height + 2 {
        for x in 0..maze.width + 2 {
            if maze.stickiness[y][x] == 0 || (!maze.open_walls.contains(&((x, y), (x + 1, y))) && !maze.open_walls.contains(&((x, y), (x, y + 1)))) {
                write!(file, "X")?;
            } else {
                write!(file, "{}", if maze.stickiness[y][x] == b'S' || maze.stickiness[y][x] == b'G' {
                    maze.stickiness[y][x] as char
                } else {
                    char::from_digit(maze.stickiness[y][x] as u32, 10).unwrap()
                })?;
            }
        }
        writeln!(file)?;
    }

    Ok(())
}

fn main() {
    let matches = App::new("Maze Generator")
        .version("1.0")
        .author("Metee Yingyongwatthanakit <metee.ying@gmail.com>")
        .about("Generates a maze with Kruskal's algorithm, assigns stickiness to each cell, and can mark a start and goal")
        .arg(Arg::with_name("width")
            .short('w')
            .long("width")
            .help("Sets the width of the maze")
            .takes_value(true))
        .arg(Arg::with_name("height")
            .short('h')
            .long("height")
            .help("Sets the height of the maze")
            .takes_value(true))
        .arg(Arg::with_name("output")
            .short('o')
            .long("output")
            .help("Output file name (optional, prints to console if not provided)")
            .takes_value(true))
        .arg(Arg::with_name("map")
            .short('m')
            .long("map")
            .help("Include a start (S) and goal (G) in the maze"))
        .get_matches();

    let width = matches.value_of("width").unwrap_or("10").parse().unwrap_or(10);
    let height = matches.value_of("height").unwrap_or("10").parse().unwrap_or(10);
    let output_file = matches.value_of("output");
    let include_map = matches.is_present("map");

    let mut maze = Maze::new(width, height);
    maze.generate();

    if include_map {
        maze.add_map();
    }

    match output_file {
        Some(file_name) => {
            if let Err(e) = save_to_file(&maze, file_name) {
                eprintln!("Error saving to file: {}", e);
            }
        },
        None => print_maze(&maze),
    }
}
