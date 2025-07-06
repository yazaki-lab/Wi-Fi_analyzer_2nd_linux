// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[tauri::command]
fn simple_command() {
    println!("I was invoked from JS!");
}
// fn main() {
//     WiFiAnalyzer2nd_lib::run()
// }
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            simple_command,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
