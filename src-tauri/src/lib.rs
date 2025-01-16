use tauri::{AppHandle, Emitter, Manager};
use serde::Serialize;

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
#[derive(PartialEq, Eq, Hash)]
struct Cell {
    x: usize,
    y: usize,
}

impl Cell {
    fn new(x: usize, y: usize) -> Self {
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

    velocity: Vec<Vec<f32>>,
    particles: Vec<Particle>,
    fluid_cells: HashSet<Cell>,
}

impl Simulation {
    fn empty(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            velocity: vec![vec![0.0; cols + 1]; rows + 1],
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
            fluid_cells.insert(Cell::new(x as usize, y as usize));
        }

        Self {
            rows,
            cols,
            velocity: vec![vec![0.0; cols + 1]; rows + 1],
            particles,
            fluid_cells,
        }
    }

    fn add_particle(&mut self, particle: Particle) {
        self.fluid_cells.insert(Cell::new(particle.x as usize, particle.y as usize));
        self.particles.push(particle);
    }

    fn simulate_step(&mut self, dt: f32) -> Vec<Vec<u8>> {
        let mut new_fluid_cells = HashSet::new();
        let mut grid: Vec<Vec<u8>> = vec![vec![0; self.cols]; self.rows];

        for particle in self.particles.iter_mut() {
            // Apply forces
            const GRAVITY: f32 = 9.81;
            particle.add_velocity(0.0, GRAVITY * dt);
            particle.apply_velocity(dt);
            particle.apply_bounds(self.rows, self.cols);

            // TODO push particles out of solids

            // Particle -> Velocity
            const CELL_SIZE: f32 = 1.0;

            let x_cell = particle.x.floor() as usize;
            let x_offset = (particle.x - x_cell as f32) / CELL_SIZE;

            let y_cell = particle.y.floor() as usize;
            let y_offset = (particle.y - y_cell as f32) / CELL_SIZE;

            let w1 = (1.0 - x_offset) * (1.0 - y_offset);
            let w2 = x_offset * (1.0 - y_offset);   
            let w3 = x_offset * y_offset;
            let w4 = (1.0 - x_offset) * y_offset;

            new_fluid_cells.insert(Cell::new(x_cell, y_cell));
            grid[y_cell][x_cell] += 1;

            /*let q1 = self.velocity[y_cell][x_cell];
            let q2 = self.velocity[y_cell][x_cell + 1];
            let q3 = self.velocity[y_cell + 1][x_cell + 1];
            let q4 = self.velocity[y_cell + 1][x_cell];*/

            // Incompressible

            // Velocity -> Particle
        }

        for row in 0..self.rows{
            for col in 0..self.cols {
                // U, V velocity components
                let u0 = col;
                let u1 = col + 1;
                let v0 = row;
                let v1 = row + 1;

                // Ignore solids
                let left_solid = if u0 == 0 { 0.0 } else { 1.0 };
                let right_solid = if u1 == self.cols { 0.0 } else { 1.0 };
                let top_solid = if v0 == 0 { 0.0 } else { 1.0 };
                let bottom_solid = if v1 == self.rows { 0.0 } else { 1.0 };

                let fluid_neighbours: f32 = 4.0 - left_solid - right_solid - top_solid - bottom_solid;
            }
        }

        self.fluid_cells = new_fluid_cells;
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
        let mut simulation = Simulation::empty(rows, cols);
        let mut particle = Particle::new(10.0, 10.0);
        particle.add_velocity(0.0, 0.1);
        simulation.add_particle(particle);

        while state.lock().unwrap().running {
            let grid = simulation.simulate_step(0.1);
            
            app.emit(UPDATE_GRID_EVENT, FluidGrid { data: grid }).unwrap();
            thread::sleep(std::time::Duration::from_secs(1));
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
