use tauri::{AppHandle, Emitter};
use serde::Serialize;

use std::thread;
use rand::Rng;

// Events
#[derive(Clone, Serialize)]
struct FluidGrid {
    data: Vec<Vec<u8>>,
}
const UPDATE_GRID_EVENT: &str = "update_grid";

// Invokes
#[tauri::command(rename_all = "snake_case")]
fn start_fluid_simulation(app: AppHandle, rows: usize, cols: usize) {
    println!("Starting fluid simulation with {} rows and {} cols", rows, cols);

    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        loop {
            thread::sleep(std::time::Duration::from_secs(1));

            let mut grid = vec![vec![0; cols]; rows];
            for row in 0..rows {
                for col in 0..cols {
                    grid[row][col] = rng.gen_range(100..=250);
                }
            }
    
            app.emit(UPDATE_GRID_EVENT, FluidGrid {data: grid }).unwrap();
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![start_fluid_simulation])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
