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
    let mut debug_info = Vec::new();

    // Method 1: airport コマンドを使用
    println!("Trying airport command...");  
    let airport_output = Command::new("/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport")
        .args(&["-s"])
        .output();

    match airport_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                debug_info.push(format!("airport -s output: {}", stdout));
                println!("airport command succeeded");
                
                for (i, line) in stdout.lines().enumerate() {
                    println!("Line {}: {}", i, line);
                    if i == 0 {
                        continue; // ヘッダーをスキップ
                    }
                    
                    // スペースで分割して BSSID を取得
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let potential_bssid = parts[1];
                        println!("Potential BSSID: {}", potential_bssid);
                        
                        // BSSID の形式チェック (XX:XX:XX:XX:XX:XX)
                        if potential_bssid.len() == 17 && potential_bssid.matches(':').count() == 5 {
                            // 各部分が16進数かチェック
                            let hex_parts: Vec<&str> = potential_bssid.split(':').collect();
                            if hex_parts.len() == 6 && hex_parts.iter().all(|part| part.len() == 2 && part.chars().all(|c| c.is_ascii_hexdigit())) {
                                bssids.push(potential_bssid.to_string());
                                println!("Added BSSID: {}", potential_bssid);
                            }
                        }
                    }
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                debug_info.push(format!("airport command failed: {}", stderr));
                println!("airport command failed: {}", stderr);
            }
        }
        Err(e) => {
            debug_info.push(format!("airport command error: {}", e));
            println!("airport command error: {}", e);
        }
    }

    // Method 2: ターミナルから直接 airport コマンドを実行
    if bssids.is_empty() {
        println!("Trying airport via sh...");
        let sh_output = Command::new("sh")
            .args(&["-c", "/System/Library/PrivateFrameworks/Apple80211.framework/Versions/Current/Resources/airport -s"])
            .output();

        if let Ok(output) = sh_output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                debug_info.push(format!("sh airport output: {}", stdout));
                
                for line in stdout.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let potential_bssid = parts[1];
                        if potential_bssid.len() == 17 && potential_bssid.matches(':').count() == 5 {
                            let hex_parts: Vec<&str> = potential_bssid.split(':').collect();
                            if hex_parts.len() == 6 && hex_parts.iter().all(|part| part.len() == 2 && part.chars().all(|c| c.is_ascii_hexdigit())) {
                                bssids.push(potential_bssid.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Method 3: system_profiler を試す
    if bssids.is_empty() {
        println!("Trying system_profiler...");
        let output = Command::new("system_profiler")
            .args(&["SPAirPortDataType"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                debug_info.push(format!("system_profiler output: {}", stdout));
                
                // より詳細なパース
                for line in stdout.lines() {
                    if line.contains("BSSID") || line.contains("Address") {
                        // コロンで分割して BSSID を探す
                        if let Some(colon_pos) = line.find(':') {
                            let after_colon = &line[colon_pos + 1..];
                            let potential_bssid = after_colon.trim();
                            
                            if potential_bssid.len() == 17 && potential_bssid.matches(':').count() == 5 {
                                let hex_parts: Vec<&str> = potential_bssid.split(':').collect();
                                if hex_parts.len() == 6 && hex_parts.iter().all(|part| part.len() == 2 && part.chars().all(|c| c.is_ascii_hexdigit())) {
                                    bssids.push(potential_bssid.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Method 4: iwlist がインストールされている場合
    if bssids.is_empty() {
        println!("Trying iwlist...");
        let iwlist_output = Command::new("iwlist")
            .args(&["scan"])
            .output();

        if let Ok(output) = iwlist_output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                debug_info.push(format!("iwlist output: {}", stdout));
                
                for line in stdout.lines() {
                    if line.contains("Address:") {
                        if let Some(address_part) = line.split("Address:").nth(1) {
                            let potential_bssid = address_part.trim();
                            if potential_bssid.len() == 17 && potential_bssid.matches(':').count() == 5 {
                                bssids.push(potential_bssid.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // デバッグ情報を含める
    if bssids.is_empty() {
        bssids.push("=== DEBUG INFO ===".to_string());
        bssids.extend(debug_info);
        bssids.push("=== END DEBUG ===".to_string());
        bssids.push("No BSSID found - check permissions".to_string());
    }

    Ok(bssids)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_bssids])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}