// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#[tauri::command]
fn simple_command() {
    println!("I was invoked from JS!");
}
fn main() {
    WiFiAnalyzer2nd_lib::run()
}
