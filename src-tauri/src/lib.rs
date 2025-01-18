use tauri::{AppHandle, Emitter, Manager};
use serde::Serialize;

use std::cmp;
use std::hash::Hash;
use std::thread;
use std::sync::{Arc, Mutex};
use std::collections::HashSet;

use rand::Rng;

// Events
#[derive(Clone, Serialize)]
struct FluidGrid {
    data: Vec<Vec<u8>>,
}
const UPDATE_GRID_EVENT: &str = "update_grid";

// Simulation
#[derive(PartialEq, Eq, Hash, Clone)]
struct Vec2<T> {
    x: T,
    y: T,
}

impl<T> Vec2<T> {
    fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
}

impl Particle {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y, vx: 0.0, vy: 0.0 }
    }

    fn set_velocity(&mut self, vx: f32, vy: f32) {
        self.vx = vx;
        self.vy = vy;
    }

    fn add_velocity(&mut self, vx: f32, vy: f32) {
        self.vx += vx;
        self.vy += vy;
    }

    fn apply_velocity(&mut self, dt: f32) {
        self.x += self.vx * dt;
        self.y += self.vy * dt;
    }

    fn apply_bounds(&mut self, rows: usize, cols: usize) {
        if self.x < 0.0 {
            self.x = 0.0;
            self.vx = 0.0;
        } else if self.x >= cols as f32 {
            self.x = cols as f32 - 0.01;
            self.vx = 0.0;
        }

        if self.y < 0.0 {
            self.y = 0.0;
            self.vy = 0.0;
        } else if self.y >= rows as f32 {
            self.y = rows as f32 - 0.01;
            self.vy = 0.0;
        }
    }
}

struct Simulation {
    rows: usize,
    cols: usize,

    particles: Vec<Particle>,
    //velocity_x: Vec<Vec<f32>>,
    //velocity_y: Vec<Vec<f32>>,
    fluid_cells: HashSet<Vec2<usize>>,
}

impl Simulation {
    fn empty(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            particles: Vec::new(),
            fluid_cells: HashSet::new(),
        }
    }

    fn rand(rows: usize, cols: usize, count: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut particles = Vec::new();
        let mut fluid_cells = HashSet::new();

        for _ in 0..count {
            let x = rng.gen_range(0..cols);
            let y = rng.gen_range(0..rows);

            particles.push(Particle::new(x as f32, y as f32));
            fluid_cells.insert(Vec2::new(x as usize, y as usize));
        }

        Self {
            rows,
            cols,
            particles,
            fluid_cells,
        }
    }

    fn add_particle(&mut self, particle: Particle) {
        self.fluid_cells.insert(Vec2::<usize>::new(particle.x as usize, particle.y as usize));
        self.particles.push(particle);
    }

    fn simulate_step(&mut self, dt: f32) -> Vec<Vec<u8>> {
        self.fluid_cells.clear();
        let mut velocity_x = vec![vec![0.0f32; self.cols + 1]; self.rows];
        let mut velocity_y = vec![vec![0.0f32; self.cols]; self.rows + 1];
        let mut grid: Vec<Vec<u8>> = vec![vec![0; self.cols]; self.rows];

        const CELL_SIZE: f32 = 1.0;

        // Apply forces + Particle -> Velocity
        for particle in self.particles.iter_mut() {
            const GRAVITY: f32 = 9.81;
            particle.add_velocity(0.0, GRAVITY * dt);
            particle.apply_velocity(dt);
            particle.apply_bounds(self.rows, self.cols);

            // TODO push particles out of solids

            let x_cell = particle.x.floor() as usize;
            let x_offset = (particle.x - x_cell as f32) / CELL_SIZE;

            let y_cell = particle.y.floor() as usize;
            let y_offset = (particle.y - y_cell as f32) / CELL_SIZE;

            self.fluid_cells.insert(Vec2::<usize>::new(x_cell, y_cell));
            grid[y_cell][x_cell] += 1;

            let wx0 = if x_cell <= 0 { 0.0 } else { x_offset };
            let wx1 = if x_cell >= self.cols { 0.0 } else { 1.0 - x_offset };
            let wy0 = if y_cell <= 0 { 0.0 } else { y_offset };
            let wy1 = if y_cell >= self.rows { 0.0 } else { 1.0 - y_offset };

            velocity_x[y_cell][x_cell] += particle.vx * wx0;
            velocity_x[y_cell][x_cell + 1] += particle.vx * wx1;
            velocity_y[y_cell][x_cell] += particle.vy * wy0;
            velocity_y[y_cell + 1][x_cell] += particle.vy * wy1;
        }

        // Incompressible
        let mut incompressed_velocity_x = vec![vec![0.0f32; self.cols + 1]; self.rows];
        let mut incompressed_velocity_y = vec![vec![0.0f32; self.cols]; self.rows + 1];
        for row in 0..self.rows {
            for col in 0..self.cols {
                if !self.fluid_cells.contains(&Vec2::new(col, row)) {
                    continue;
                }

                let x0 = col;
                let x1 = col + 1;
                let y0 = row;
                let y1 = row + 1;

                // Ignore solids
                let left_solid = if x0 == 0 { 0.0 } else { 1.0 };
                let right_solid = if x1 == self.cols { 0.0 } else { 1.0 };
                let top_solid = if y0 == 0 { 0.0 } else { 1.0 };
                let bottom_solid = if y1 == self.rows { 0.0 } else { 1.0 };

                let fluid_neighbours: f32 = left_solid + right_solid + top_solid + bottom_solid;

                let dx = velocity_x[y0][x1] - velocity_x[y0][x0];
                let dy = velocity_y[y1][x0] - velocity_y[y0][x0];
                let divergence = dx + dy;
                
                incompressed_velocity_x[y0][x0] = velocity_x[y0][x0] + left_solid * divergence / fluid_neighbours;
                incompressed_velocity_x[y0][x1] = velocity_x[y0][x1] - right_solid * divergence / fluid_neighbours;
                incompressed_velocity_y[y0][x0] = velocity_y[y0][x0] + top_solid * divergence / fluid_neighbours;
                incompressed_velocity_y[y1][x0] = velocity_y[y1][x0] - bottom_solid * divergence / fluid_neighbours;
            }
        }

        // Velocity -> Particle
        for particle in self.particles.iter_mut() {
            let x_cell = particle.x.floor() as usize;
            let x_offset = (particle.x - x_cell as f32) / CELL_SIZE;

            let y_cell = particle.y.floor() as usize;
            let y_offset = (particle.y - y_cell as f32) / CELL_SIZE;

            let wx0 = if x_cell <= 0 { 0.0 } else { x_offset };
            let wx1 = if x_cell >= self.cols { 0.0 } else { 1.0 - x_offset };
            let wy0 = if y_cell <= 0 { 0.0 } else { y_offset };
            let wy1 = if y_cell >= self.rows { 0.0 } else { 1.0 - y_offset };

            let vx = incompressed_velocity_x[y_cell][x_cell] * wx0 + incompressed_velocity_x[y_cell][x_cell + 1] * wx1;
            let vy = incompressed_velocity_y[y_cell][x_cell] * wy0 + incompressed_velocity_y[y_cell + 1][x_cell] * wy1;

            particle.set_velocity(vx, vy);
        }

        grid
    }
}

// Invokes
#[tauri::command(rename_all = "snake_case")]
fn start_fluid_simulation(app: AppHandle, rows: usize, cols: usize) {
    println!("Starting fluid simulation with {} rows and {} cols", rows, cols);

    let state = app.state::<Arc<Mutex<State>>>();
    state.lock().unwrap().running = true;
    let state = Arc::clone(&state);

    thread::spawn(move || {
        let mut simulation = Simulation::rand(rows, cols, 50);

        const FPS: i32 = 30;
        const DT: f32 = 1.0 / FPS as f32;
        const WAIT_TIME: std::time::Duration = std::time::Duration::from_millis(1000 / FPS as u64);

        while state.lock().unwrap().running {
            let grid = simulation.simulate_step(DT);
            
            app.emit(UPDATE_GRID_EVENT, FluidGrid { data: grid }).unwrap();
            thread::sleep(WAIT_TIME);
        }
    });
}

#[tauri::command(rename_all = "snake_case")]
fn stop_fluid_simulation(app: AppHandle) {
    println!("Stopping fluid simulation");

    let state = app.state::<Arc<Mutex<State>>>();
    state.lock().unwrap().running = false;
}

// App state
#[derive(Default)]
struct State {
    running: bool,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            app.manage(Arc::new(Mutex::new(State::default())));
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_fluid_simulation, stop_fluid_simulation])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
