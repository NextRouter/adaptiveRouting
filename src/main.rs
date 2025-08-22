use core::time;
use std::error::Error;
use tokio::time::sleep;
use serde_json::Value;

#[tokio::main]
async fn main() {
    loop{
        println!("{:?}", get_packetloss().await);
        sleep(time::Duration::from_secs(1)).await;
    }
}
async fn get_packetloss() -> Result<String, Box<dyn Error>> {
    let prometheus_url = "http://localhost:9090/api/v1/query";
    // let query = r#"{job="packetloss",__name__="tcp_monitor_packet_loss_missing_per_second"}"#;
    let query = r#"{job="networktraffic",__name__="local_ip_rx_bytes_rate"}"#;

    let client = reqwest::Client::new();
    
    let response = client
        .get(prometheus_url)
        .query(&[("query", query)])
        .send()
        .await?;

    // Parse the JSON response into a serde_json::Value
    let body = response.text().await?;
    let json: Value = serde_json::from_str(&body)?;

    // Extract the value from the JSON structure
    // The path is data.result[0].value[1]
    let value = json["data"]["result"][0]["value"][1]
        .as_str()
        .ok_or("Value not found or not a string")?
        .to_string();

    return Ok(value);
}
