// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use tauri::command;

#[command]
fn get_bssids(building: String, room: String) -> Result<Vec<String>, String> {
    println!("Building: {}, Room: {}", building, room);
    
    // macOS専用でBSSIDを取得
    let bssids = get_bssids_macos();

    match bssids {
        Ok(mut bssids) => {
            // 重複を削除
            bssids.sort();
            bssids.dedup();
            Ok(bssids)
        }
        Err(e) => Err(format!("BSSID の取得に失敗しました: {}", e)),
    }
}

fn get_bssids_macos() -> Result<Vec<String>, String> {
    let mut bssids = Vec::new();

    // airport コマンドを使用（最も確実な方法）
    let airport_output = Command::new("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport")
        .args(&["-s"])
        .output();

    if let Ok(airport_output) = airport_output {
        if airport_output.status.success() {
            let airport_stdout = String::from_utf8_lossy(&airport_output.stdout);
            for line in airport_stdout.lines().skip(1) { // ヘッダーをスキップ
                let fields: Vec<&str> = line.split_whitespace().collect();
                if fields.len() >= 2 {
                    let bssid = fields[1].to_string();
                    if bssid.len() == 17 && bssid.matches(':').count() == 5 {
                        bssids.push(bssid);
                    }
                }
            }
        }
    }

    // airport コマンドが失敗した場合、system_profiler を試す
    if bssids.is_empty() {
        let output = Command::new("system_profiler")
            .args(&["SPAirPortDataType"])
            .output()
            .map_err(|e| format!("コマンドの実行に失敗しました: {}", e))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // BSSID を抽出
            for line in stdout.lines() {
                if line.contains("BSSID") {
                    if let Some(bssid) = line.split(':').nth(1) {
                        let bssid = bssid.trim().to_string();
                        if !bssid.is_empty() {
                            bssids.push(bssid);
                        }
                    }
                }
            }
        }
    }

    if bssids.is_empty() {
        bssids.push("no BSSID found (macOS)".to_string());
    }

    Ok(bssids)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_bssids])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}