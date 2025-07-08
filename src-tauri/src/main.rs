// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use tauri::command;

#[command]
fn get_bssids(building: String, room: String) -> Result<Vec<String>, String> {
    println!("Building: {}, Room: {}", building, room);
    
    // Linux専用でBSSIDを取得
    let bssids = get_bssids_linux();

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

fn get_bssids_linux() -> Result<Vec<String>, String> {
    let mut bssids = Vec::new();
    let mut debug_info = Vec::new();

    // Method 1: nmcli コマンドを使用 (NetworkManager)
    println!("Trying nmcli command...");  
    let nmcli_output = Command::new("nmcli")
        .args(&["device", "wifi", "list"])
        .output();

    match nmcli_output {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                debug_info.push(format!("nmcli output: {}", stdout));
                println!("nmcli command succeeded");
                
                for (i, line) in stdout.lines().enumerate() {
                    println!("Line {}: {}", i, line);
                    if i == 0 {
                        continue; // ヘッダーをスキップ
                    }
                    
                    // nmcli の出力をパース
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let potential_bssid = parts[0]; // nmcliでは最初の列がBSSID
                        println!("Potential BSSID: {}", potential_bssid);
                        
                        // BSSID の形式チェック (XX:XX:XX:XX:XX:XX)
                        if is_valid_bssid(potential_bssid) {
                            bssids.push(potential_bssid.to_string());
                            println!("Added BSSID: {}", potential_bssid);
                        }
                    }
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                debug_info.push(format!("nmcli command failed: {}", stderr));
                println!("nmcli command failed: {}", stderr);
            }
        }
        Err(e) => {
            debug_info.push(format!("nmcli command error: {}", e));
            println!("nmcli command error: {}", e);
        }
    }

    // Method 2: iwlist コマンドを使用 (wireless-tools)
    if bssids.is_empty() {
        println!("Trying iwlist command...");
        
        // まずワイヤレスインターフェースを取得
        let interfaces = get_wireless_interfaces();
        
        for interface in interfaces {
            println!("Scanning interface: {}", interface);
            let iwlist_output = Command::new("iwlist")
                .args(&[&interface, "scan"])
                .output();

            if let Ok(output) = iwlist_output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    debug_info.push(format!("iwlist {} scan output: {}", interface, stdout));
                    
                    for line in stdout.lines() {
                        if line.trim().starts_with("Address:") {
                            if let Some(address_part) = line.split("Address:").nth(1) {
                                let potential_bssid = address_part.trim();
                                if is_valid_bssid(potential_bssid) {
                                    bssids.push(potential_bssid.to_string());
                                }
                            }
                        }
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    debug_info.push(format!("iwlist {} failed: {}", interface, stderr));
                }
            }
        }
    }

    // Method 3: iw コマンドを使用 (nl80211)
    if bssids.is_empty() {
        println!("Trying iw command...");
        
        let interfaces = get_wireless_interfaces();
        
        for interface in interfaces {
            println!("Scanning with iw on interface: {}", interface);
            let iw_output = Command::new("iw")
                .args(&[&interface, "scan"])
                .output();

            if let Ok(output) = iw_output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    debug_info.push(format!("iw {} scan output: {}", interface, stdout));
                    
                    for line in stdout.lines() {
                        if line.trim().starts_with("BSS ") {
                            let parts: Vec<&str> = line.split_whitespace().collect();
                            if parts.len() >= 2 {
                                let potential_bssid = parts[1].trim_end_matches("(on");
                                if is_valid_bssid(potential_bssid) {
                                    bssids.push(potential_bssid.to_string());
                                }
                            }
                        }
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    debug_info.push(format!("iw {} failed: {}", interface, stderr));
                }
            }
        }
    }

    // Method 4: /proc/net/wireless を確認
    if bssids.is_empty() {
        println!("Trying /proc/net/wireless...");
        match std::fs::read_to_string("/proc/net/wireless") {
            Ok(content) => {
                debug_info.push(format!("/proc/net/wireless content: {}", content));
                // このファイルからはBSSIDは取得できませんが、
                // ワイヤレスインターフェースの存在確認に使用
            }
            Err(e) => {
                debug_info.push(format!("/proc/net/wireless read error: {}", e));
            }
        }
    }

    // デバッグ情報を含める
    if bssids.is_empty() {
        bssids.push("=== DEBUG INFO ===".to_string());
        bssids.extend(debug_info);
        bssids.push("=== END DEBUG ===".to_string());
        bssids.push("No BSSID found - check permissions and wireless tools".to_string());
        bssids.push("Required tools: nmcli, iwlist, or iw".to_string());
        bssids.push("You may need to run with sudo for some commands".to_string());
    }

    Ok(bssids)
}

fn get_wireless_interfaces() -> Vec<String> {
    let mut interfaces = Vec::new();
    
    // /sys/class/net からワイヤレスインターフェースを取得
    if let Ok(entries) = std::fs::read_dir("/sys/class/net") {
        for entry in entries.flatten() {
            let interface_name = entry.file_name().to_string_lossy().to_string();
            let wireless_path = format!("/sys/class/net/{}/wireless", interface_name);
            
            if std::path::Path::new(&wireless_path).exists() {
                interfaces.push(interface_name);
            }
        }
    }
    
    // 一般的なワイヤレスインターフェース名も試す
    if interfaces.is_empty() {
        let common_interfaces = vec![
            "wlan0", "wlan1", "wlp2s0", "wlp3s0", "wlp4s0", "wlp1s0",
            "wlo1", "wlx", "wifi0", "ath0", "ra0"
        ];
        
        for interface in common_interfaces {
            let wireless_path = format!("/sys/class/net/{}/wireless", interface);
            if std::path::Path::new(&wireless_path).exists() {
                interfaces.push(interface.to_string());
            }
        }
    }
    
    // 最後の手段として ip link show を使用
    if interfaces.is_empty() {
        if let Ok(output) = Command::new("ip").args(&["link", "show"]).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.contains("wlan") || line.contains("wlp") || line.contains("wlo") {
                        if let Some(interface) = extract_interface_name(line) {
                            interfaces.push(interface);
                        }
                    }
                }
            }
        }
    }
    
    interfaces
}

fn extract_interface_name(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() >= 2 {
        let interface_with_colon = parts[1];
        let interface = interface_with_colon.trim_end_matches(':');
        if interface.starts_with("wl") || interface.starts_with("wifi") {
            return Some(interface.to_string());
        }
    }
    None
}

fn is_valid_bssid(bssid: &str) -> bool {
    // BSSID の形式チェック (XX:XX:XX:XX:XX:XX)
    if bssid.len() == 17 && bssid.matches(':').count() == 5 {
        // 各部分が16進数かチェック
        let hex_parts: Vec<&str> = bssid.split(':').collect();
        if hex_parts.len() == 6 && hex_parts.iter().all(|part| part.len() == 2 && part.chars().all(|c| c.is_ascii_hexdigit())) {
            return true;
        }
    }
    false
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_bssids])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}