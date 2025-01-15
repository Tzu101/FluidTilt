use tauri::{AppHandle, Emitter, Manager};
use serde::Serialize;

use std::{sync::Arc, sync::Mutex, thread};
use rand::Rng;

// Events
#[derive(Clone, Serialize)]
struct FluidGrid<'a> {
    data: &'a Vec<Vec<u8>>,
}
const UPDATE_GRID_EVENT: &str = "update_grid";

// Invokes
#[tauri::command(rename_all = "snake_case")]
fn start_fluid_simulation(app: AppHandle, rows: usize, cols: usize) {
    println!("Starting fluid simulation with {} rows and {} cols", rows, cols);

    let state = app.state::<Arc<Mutex<State>>>();
    state.lock().unwrap().running = true;
    let state = Arc::clone(&state);

    thread::spawn(move || {
        let mut rng = rand::thread_rng();
        while state.lock().unwrap().running {
            thread::sleep(std::time::Duration::from_secs(1));

            let mut grid = vec![vec![0; cols]; rows];
            for row in 0..rows {
                for col in 0..cols {
                    grid[row][col] = rng.gen_range(100..=250);
                }
            }
    
            app.emit(UPDATE_GRID_EVENT, FluidGrid { data: &grid }).unwrap();
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
