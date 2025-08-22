use serde_json::{Value, json};
use std::{error::Error, sync::OnceLock, time::Duration};
use tokio::process::Command;
use tokio::time::sleep;

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn http_client() -> &'static reqwest::Client {
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .expect("failed to build reqwest client")
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    loop {
        let result = get_nftables_config().await?;
        println!("{:?}", get_ip_nic_list(result.clone()));

        let ip_nic_list = get_ip_nic_list(result);
        let ip_addresses: Vec<String> = ip_nic_list
            .iter()
            .filter_map(|item| {
                // コロンで分割してIPアドレス部分を取得
                item.split(':').nth(1).map(|ip| ip.to_string())
            })
            .collect();

        let mut packetloss_list = get_packetloss().await.expect("REASON");
        for item in &packetloss_list {
            if ip_addresses.iter().any(|ip| item.contains(ip)) {
                println!("{:?}", item);
            }
        }

        packetloss_list.clear();

        sleep(Duration::from_secs(1)).await;
    }
}
async fn get_packetloss() -> Result<Vec<String>, Box<dyn Error>> {
    let prometheus_url = "http://localhost:9090/api/v1/query";
    let query = r#"{job="localpacketdump",__name__=~"network_ip_window_size_changes_per_sec"}"#;

    let client = http_client();

    let response = client
        .get(prometheus_url)
        .query(&[("query", query)])
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await?
        .error_for_status()?;

    // Parse the JSON response into a serde_json::Value
    let body = response.text().await?;

    // Collect pairs of ip_address and value from: data.result[].{metric.ip_address, value[1]}
    let mut ip_packetloss = vec![];

    if let Ok(json) = serde_json::from_str::<Value>(&body) {
        if let Some(results) = json
            .get("data")
            .and_then(|d| d.get("result"))
            .and_then(|r| r.as_array())
        {
            for entry in results {
                let ip = entry
                    .get("metric")
                    .and_then(|m| m.get("ip_address"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("unknown");
                let val = entry
                    .get("value")
                    .and_then(|v| v.get(1))
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                ip_packetloss.push(format!("{}={}", ip, val));
            }
        }
    }
    Ok(ip_packetloss)
}

async fn get_nftables_config() -> Result<Value, Box<dyn Error>> {
    let output = Command::new("nft")
        .args(["--json", "list", "table", "inet", "nat"])
        .output()
        .await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("nft command failed: {}", stderr).into());
    }

    let stdout = String::from_utf8(output.stdout)?;
    let ruleset: Value = serde_json::from_str(&stdout)?;

    Ok(ruleset)
}

fn get_ip_nic_list(result: Value) -> Vec<String> {
    let mut ip_nic_list = Vec::new();

    if let Some(entries) = result["nftables"].as_array() {
        for entry in entries {
            if let Some(rule) = entry.get("rule") {
                if let Some(exprs) = rule.get("expr").and_then(|e| e.as_array()) {
                    let mut nic = None;
                    let mut ip = None;

                    for expr in exprs {
                        if let Some(m) = expr.get("match") {
                            // Check for oifname match
                            if let Some(meta) = m.get("left").and_then(|l| l.get("meta")) {
                                if meta.get("key").and_then(|k| k.as_str()) == Some("oifname") {
                                    nic = m.get("right").and_then(|r| r.as_str());
                                }
                            }
                            // Check for saddr match
                            else if let Some(payload) =
                                m.get("left").and_then(|l| l.get("payload"))
                            {
                                if payload.get("field").and_then(|f| f.as_str()) == Some("saddr") {
                                    ip = m.get("right").and_then(|r| r.as_str());
                                }
                            }
                        }
                    }

                    if let (Some(nic_name), Some(ip_addr)) = (nic, ip) {
                        ip_nic_list.push(format!("{}:{}", nic_name, ip_addr));
                    }
                }
            }
        }
    }
    ip_nic_list
}
