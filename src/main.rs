use serde_json::Value;
use std::process::Output;
use std::{error::Error, time::Duration};
use tokio::process::Command;
use tokio::time::sleep;

async fn change_nft(ip: &str, wan: &str) -> Result<(), String> {
    const TABLE: &str = "inet mangle";
    const WAN1_SET: &str = "wan1_hosts";
    const WAN2_SET: &str = "wan2_hosts";

    let (to_set, from_set) = match wan {
        "wan1" => (WAN1_SET, WAN2_SET),
        "wan2" => (WAN2_SET, WAN1_SET),
        _ => {
            return Err(format!(
                "無効なNICです: '{}'。'wan1' または 'wan2' を使用してください。",
                wan
            ))
        }
    };

    println!("{}: {} -> {}", ip, from_set, to_set);

    let delete_args = &["delete", "element", TABLE, from_set, "{", ip, "}"];
    match run_nft_command(delete_args).await {
        Ok(_) => {}
        Err(e) => {
            if !e.contains("No such file or directory") {
                return Err(format!(
                    "{}からのIP削除中に予期せぬエラーが発生しました: {}",
                    from_set, e
                ));
            }
            println!("{} cannot be found in {}", ip, from_set);
        }
    }

    let add_args = &["add", "element", TABLE, to_set, "{", ip, "}"];
    match run_nft_command(add_args).await {
        Ok(_) => {}
        Err(e) => {
            if !e.contains("File exists") {
                return Err(format!(
                    "{}へのIP追加中に予期せぬエラーが発生しました: {}",
                    to_set, e
                ));
            }
            println!("{} cannot be found in {}", ip, from_set);
        }
    }

    Ok(())
}

async fn run_nft_command(args: &[&str]) -> Result<Output, String> {
    let output = Command::new("nft")
        .args(args)
        .output()
        .await
        .map_err(|e| format!("コマンドの実行に失敗しました: {}", e))?;

    if output.status.success() {
        Ok(output)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(stderr.to_string())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // match change_nft("192.168.1.100", "wan1").await {
    //     Ok(_) => {
    //         println!("   永続化するには 'sudo nft list ruleset > /etc/nftables.conf' を実行してください。");
    //     }
    //     Err(e) => {
    //         eprintln!("\n Err: {}", e);
    //     }
    // }
    loop {
        let result = get_nftables_config().await?;
        println!(
            "{}",
            get_ip_nic_list(result.clone())
        );

        // let ip_nic_list = get_ip_nic_list(result);

        // let nic_list: Vec<String> = ["eth0", "eth1"].iter().map(|s| s.to_string()).collect();

        // let ip_addresses: Vec<String> = ip_nic_list
        //     .as_object()
        //     .map(|obj| obj.keys().cloned().collect())
        //     .unwrap_or_default();

        // let _nft_id: Vec<u64> = ip_nic_list
        //     .as_object()
        //     .map(|obj| {
        //         obj.values()
        //             .filter_map(|v| v.get("id").and_then(|id| id.as_u64()))
        //             .collect()
        //     })
        //     .unwrap_or_default();

        // let mut packetloss_list = get_packetloss().await.expect("REASON");

        // for item in &packetloss_list {
        //     if ip_addresses.iter().any(|ip| item.contains(ip)) {
        //         if let Some((ip, packetloss_str)) = item.split_once('=') {
        //             // println!("IP: {}", ip);
        //             // println!("Packetloss: {}", packetloss_str);
        //             if let Ok(packetloss) = packetloss_str.parse::<f64>() {
        //                 if packetloss >= 10000.0 {
        //                     if let Some(nic) = ip_nic_list
        //                         .get(ip)
        //                         .and_then(|v| v.get("nic"))
        //                         .and_then(|n| n.as_str())
        //                     {
        //                         println!(
        //                             "<Packetloss too high> IP: {} Packetloss: {} Interface: {}",
        //                             ip, packetloss, nic
        //                         );

        //                         let other_nics: Vec<_> =
        //                             nic_list.iter().filter(|&n| *n != nic).collect();
        //                         println!("Other available interfaces: {:?}", other_nics);
        //                         println!("Use available interfaces: {:?}", nic_list.iter().filter(|&n| *n == nic).collect::<Vec<_>>());

        //                         // ip_nic_listから現在のnicに関連するIPアドレスを取得
        //                         let current_nic_ips: Vec<String> = ip_nic_list
        //                             .as_object()
        //                             .map(|obj| {
        //                                 obj.iter()
        //                                     .filter(|(_, v)| {
        //                                         v.get("nic")
        //                                             .and_then(|n| n.as_str())
        //                                             .map(|n| n == nic)
        //                                             .unwrap_or(false)
        //                                     })
        //                                     .map(|(ip, _)| ip.clone())
        //                                     .collect()
        //                             })
        //                             .unwrap_or_default();
        //                         println!("IPs using interface {}: {:?}", nic, current_nic_ips);
        //                     }
        //                 }
        //             }
        //         }
        //     }
        // }
        // packetloss_list.clear();

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
        .args(["--json", "list", "table", "inet", "mangle"])
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

fn get_ip_nic_list(result: Value) -> Value {
    let mut ip_map = serde_json::Map::new();
    
    // First, collect the sets and their elements
    let mut sets = std::collections::HashMap::new();
    
    if let Some(entries) = result.get("nftables").and_then(|n| n.as_array()) {
        // Parse sets first
        for entry in entries {
            if let Some(set) = entry.get("set") {
                let set_name = set.get("name").and_then(|n| n.as_str());
                let elements = set.get("elem").and_then(|e| e.as_array());
                
                if let (Some(name), Some(elems)) = (set_name, elements) {
                    let ip_list: Vec<String> = elems
                        .iter()
                        .filter_map(|ip| ip.as_str().map(|s| s.to_string()))
                        .collect();
                    sets.insert(name.to_string(), ip_list);
                }
            }
        }
        
        // Then parse rules
        for entry in entries {
            if let Some(rule) = entry.get("rule") {
                if let Some(exprs) = rule.get("expr").and_then(|e| e.as_array()) {
                    let mut nic = None;
                    let mut set_reference = None;
                    let mut mark_value = None;

                    for expr in exprs {
                        if let Some(m) = expr.get("match") {
                            // Check for interface name (iifname)
                            if let Some(meta) = m.get("left").and_then(|l| l.get("meta")) {
                                if meta.get("key").and_then(|k| k.as_str()) == Some("iifname") {
                                    nic = m.get("right").and_then(|r| r.as_str());
                                }
                            }
                            // Check for set reference in saddr matching
                            else if let Some(payload) = m.get("left").and_then(|l| l.get("payload")) {
                                if payload.get("field").and_then(|f| f.as_str()) == Some("saddr") {
                                    set_reference = m.get("right").and_then(|r| r.as_str());
                                }
                            }
                        }
                        // Check for mangle (mark) value
                        else if let Some(mangle) = expr.get("mangle") {
                            mark_value = mangle.get("value").and_then(|v| v.as_u64());
                        }
                    }

                    // Map mark values to WAN interfaces
                    let wan_interface = match mark_value {
                        Some(1) => Some("wan1"),
                        Some(2) => Some("wan2"),
                        _ => None,
                    };

                    if let (Some(nic_name), Some(set_ref), Some(wan)) =
                        (nic, set_reference, wan_interface) {
                        // Remove '@' prefix from set reference
                        let set_name = set_ref.trim_start_matches('@');
                        
                        // Get IPs from the referenced set
                        if let Some(ip_list) = sets.get(set_name) {
                            for ip_addr in ip_list {
                                let nic_info = serde_json::json!({
                                    "nic": wan,
                                    "input_interface": nic_name,
                                    // "set": set_name,
                                    // "mark": mark_value,
                                });
                                ip_map.insert(ip_addr.clone(), nic_info);
                            }
                        }
                    }
                }
            }
        }
    }
    serde_json::Value::Object(ip_map)
}
