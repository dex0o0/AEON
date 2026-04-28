#[allow(unused_imports)]
use std::io::{Read, Write};
#[allow(unused_imports)]
use std::net::TcpStream;

pub async fn test(port: u32) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!("http://127.0.0.1:{}/status", port);

    let response = reqwest::get(&url).await?;

    let json_data: serde_json::Value = response.json().await?;

    if let Some(cpu) = json_data["cpu"]["usage_percent"].as_f64() {
        println!("CPU Usage:{:.2}", cpu);
    }
    if let Some(mem) = json_data["memory"]["usage_percent"].as_f64() {
        println!("MEM Usage:{:.2}", mem);
    }

    Ok(())
}
