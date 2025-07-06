// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use tauri::command;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub bssid: String,
    pub signal: String,
}

#[command]
fn get_bssids(building: String, room: String) -> Result<Vec<WifiNetwork>, String> {
    println!("Building: {}, Room: {}", building, room);
    
    // macOS専用でWi-Fiネットワーク情報を取得
    let networks = get_wifi_networks_macos();

    match networks {
        Ok(mut networks) => {
            // BSSIDで重複を削除
            networks.sort_by(|a, b| a.bssid.cmp(&b.bssid));
            networks.dedup_by(|a, b| a.bssid == b.bssid);
            Ok(networks)
        }
        Err(e) => Err(format!("Wi-Fi ネットワークの取得に失敗しました: {}", e)),
    }
}

fn get_wifi_networks_macos() -> Result<Vec<WifiNetwork>, String> {
    let mut networks = Vec::new();
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
                    
                    // airport -s の出力形式を解析
                    // 例: "NetworkName aa:bb:cc:dd:ee:ff  -50  1       CC/WPA2(PSK) CC"
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        let ssid = parts[0].to_string();
                        let bssid = parts[1].to_string();
                        let signal = if parts.len() >= 3 { parts[2].to_string() } else { "N/A".to_string() };
                        
                        println!("Potential - SSID: {}, BSSID: {}, Signal: {}", ssid, bssid, signal);
                        
                        // BSSID の形式チェック (XX:XX:XX:XX:XX:XX)
                        if bssid.len() == 17 && bssid.matches(':').count() == 5 {
                            // 各部分が16進数かチェック
                            let hex_parts: Vec<&str> = bssid.split(':').collect();
                            if hex_parts.len() == 6 && hex_parts.iter().all(|part| part.len() == 2 && part.chars().all(|c| c.is_ascii_hexdigit())) {
                                networks.push(WifiNetwork {
                                    ssid,
                                    bssid,
                                    signal,
                                });
                                println!("Added network: SSID={}, BSSID={}", parts[0], parts[1]);
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
    if networks.is_empty() {
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
                    if parts.len() >= 3 {
                        let ssid = parts[0].to_string();
                        let bssid = parts[1].to_string();
                        let signal = parts[2].to_string();
                        
                        if bssid.len() == 17 && bssid.matches(':').count() == 5 {
                            let hex_parts: Vec<&str> = bssid.split(':').collect();
                            if hex_parts.len() == 6 && hex_parts.iter().all(|part| part.len() == 2 && part.chars().all(|c| c.is_ascii_hexdigit())) {
                                networks.push(WifiNetwork {
                                    ssid,
                                    bssid,
                                    signal,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    // Method 3: system_profiler を試す（SSID取得は困難だがフォールバック）
    if networks.is_empty() {
        println!("Trying system_profiler...");
        let output = Command::new("system_profiler")
            .args(&["SPAirPortDataType"])
            .output();

        if let Ok(output) = output {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                debug_info.push(format!("system_profiler output: {}", stdout));
                
                // system_profilerからはSSIDとBSSIDの対応が取りにくいため、
                // 見つかったBSSIDにダミーSSIDを設定
                for line in stdout.lines() {
                    if line.contains("BSSID") || line.contains("Address") {
                        if let Some(colon_pos) = line.find(':') {
                            let after_colon = &line[colon_pos + 1..];
                            let potential_bssid = after_colon.trim();
                            
                            if potential_bssid.len() == 17 && potential_bssid.matches(':').count() == 5 {
                                let hex_parts: Vec<&str> = potential_bssid.split(':').collect();
                                if hex_parts.len() == 6 && hex_parts.iter().all(|part| part.len() == 2 && part.chars().all(|c| c.is_ascii_hexdigit())) {
                                    networks.push(WifiNetwork {
                                        ssid: "Unknown SSID".to_string(),
                                        bssid: potential_bssid.to_string(),
                                        signal: "N/A".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // デバッグ情報を含める
    if networks.is_empty() {
        networks.push(WifiNetwork {
            ssid: "=== DEBUG INFO ===".to_string(),
            bssid: "debug".to_string(),
            signal: "N/A".to_string(),
        });
        
        for info in debug_info {
            networks.push(WifiNetwork {
                ssid: info,
                bssid: "debug".to_string(),
                signal: "N/A".to_string(),
            });
        }
        
        networks.push(WifiNetwork {
            ssid: "=== END DEBUG ===".to_string(),
            bssid: "debug".to_string(),
            signal: "N/A".to_string(),
        });
        
        networks.push(WifiNetwork {
            ssid: "No networks found - check permissions".to_string(),
            bssid: "error".to_string(),
            signal: "N/A".to_string(),
        });
    }

    Ok(networks)
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_bssids])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}