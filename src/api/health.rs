use axum::response::IntoResponse;
use axum::Json;
use axum::extract::Request;
use serde_json::json;
use std::process::Command;
use regex::Regex;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, error, instrument};

fn calculate_uptime(start_timestamp: &str) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Parse the timestamp (format: "Sun 2025-06-08 08:13:27 BST")
    let parts: Vec<&str> = start_timestamp.split_whitespace().collect();
    if parts.len() < 3 {
        return "Unknown".to_string();
    }
    
    let date_time = format!("{} {} {}", parts[1], parts[2], parts[3]);
    let start_time = chrono::NaiveDateTime::parse_from_str(&date_time, "%Y-%m-%d %H:%M:%S %Z")
        .unwrap_or_default()
        .and_utc()
        .timestamp() as u64;
    
    let uptime_secs = now - start_time;
    let days = uptime_secs / 86400;
    let hours = (uptime_secs % 86400) / 3600;
    let minutes = (uptime_secs % 3600) / 60;
    
    format!("{}d {}h {}m", days, hours, minutes)
}

#[instrument]
pub async fn get_health(request: Request) -> impl IntoResponse {
    let uri = request.uri().clone();
    let result = match Command::new("systemctl")
        .args(["show", "noisebell"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let output_str = String::from_utf8_lossy(&output.stdout);
                let re = Regex::new(r"^([^=]+)=(.*)$").unwrap();
                
                let mut service_info = serde_json::Map::new();
                let mut start_timestamp = String::new();
                
                for line in output_str.lines() {
                    if let Some(caps) = re.captures(line) {
                        let key = caps.get(1).unwrap().as_str();
                        let value = caps.get(2).unwrap().as_str();
                        
                        // Only collect important metrics
                        match key {
                            "ActiveState" | "SubState" | "MainPID" | "TasksCurrent" | 
                            "CPUUsageNSec" | "MemoryCurrent" | "ExecMainStartTimestamp" => {
                                if key == "ExecMainStartTimestamp" {
                                    start_timestamp = value.to_string();
                                }
                                
                                if let Ok(num) = value.parse::<i64>() {
                                    service_info.insert(key.to_string(), json!(num));
                                } else if value == "infinity" {
                                    service_info.insert(key.to_string(), json!(null));
                                } else if value == "yes" {
                                    service_info.insert(key.to_string(), json!(true));
                                } else if value == "no" {
                                    service_info.insert(key.to_string(), json!(false));
                                } else {
                                    service_info.insert(key.to_string(), json!(value));
                                }
                            }
                            _ => continue,
                        }
                    }
                }
                
                // Add uptime calculation
                if !start_timestamp.is_empty() {
                    service_info.insert("Uptime".to_string(), json!(calculate_uptime(&start_timestamp)));
                }
                
                // Convert CPU usage to seconds
                if let Some(cpu_usage) = service_info.get("CPUUsageNSec") {
                    if let Some(cpu_secs) = cpu_usage.as_i64() {
                        service_info.insert("CPUUsageSeconds".to_string(), json!(cpu_secs / 1_000_000_000));
                    }
                }
                
                info!("Health check successful at {} - Service info: {:?}", uri, service_info);
                json!({
                    "status": "success",
                    "data": service_info
                })
            } else {
                let error_msg = String::from_utf8_lossy(&output.stderr).to_string();
                error!("Health check failed at {} - Error: {}", uri, error_msg);
                json!({
                    "status": "error",
                    "error": error_msg
                })
            }
        }
        Err(e) => {
            error!("Health check failed at {} - Failed to execute systemctl command: {}", uri, e);
            json!({
                "status": "error",
                "error": "Failed to execute systemctl command"
            })
        },
    };

    Json(result)
} 