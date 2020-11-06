use anyhow::{
    anyhow, Result
};
use regex::Regex;
use tokio::time::{delay_for, Duration};
use async_recursion::async_recursion;
use serde_json::Value;
use reqwest::Client;

/// CHECK BLOCK HASH
/// Check if the block hash are equal
/// between shadow and etherscan one by one
#[async_recursion]
pub async fn start(shadow_url: &str, etherscan_apikey: &str, alert_manager: &str, block: u64) -> Result<()> {
    println!("check block hash: {}", block);

    match block_hash_is_same(shadow_url, etherscan_apikey,  block).await {
        Ok(is_same) => {
            if !is_same {
                println!("block {} is not same", block);
                alert_block_hash_is_not_same(alert_manager, shadow_url, block).await;
            }

            delay_for(Duration::from_millis(500)).await;
            start(shadow_url, etherscan_apikey, alert_manager, block + 1).await
        }
        Err(e) => {
            println!("check_block_hash error: {:?}, wait 10 seconds to retry", e);
            delay_for(Duration::from_millis(10000)).await;
            start(shadow_url, etherscan_apikey, alert_manager, block + 1).await
        }
    }
}

async fn block_hash_is_same(shadow_url: &str, etherscan_apikey: &str, block_number: u64) -> Result<bool> {
    let hash_from_etherscan = block_hash_from_etherscan(&etherscan_apikey, block_number).await?;
    let hash_from_shadow = block_hash_from_shadow(&shadow_url, block_number).await?;

    Ok(hash_from_etherscan == hash_from_shadow)
}

async fn block_hash_from_etherscan(etherscan_apikey: &str, block_number: u64) -> Result<String> {
    let etherscan_url = format!(
        "https://api.etherscan.io/api?module=proxy&action=eth_getBlockByNumber&tag={}&boolean=true&apikey={}",
        format!("0x{:x}", block_number),
        etherscan_apikey
    );
    let resp = reqwest::get(&etherscan_url).await?;
    let content = resp.text().await?;

    let re = Regex::new(r#""hash":"([0-9a-fx]{66})""#).unwrap();
    let caps = re.captures(&content);
    match caps {
        Some(c) => {
            Ok(c[1].to_string())
        },
        None => Err(anyhow!("No hash captured: {}", content))
    }
}

async fn block_hash_from_shadow(shadow_url: &str, block_number: u64) -> Result<String> {
    let resp = reqwest::get(format!("{}/ethereum/parcel/{}", shadow_url, block_number).as_str()).await?;
    let content = resp.text().await?;

    let re = Regex::new(r#""hash":"([0-9a-fx]{66})""#).unwrap();
    let caps = re.captures(&content);
    match caps {
        Some(c) => {
            Ok(c[1].to_string())
        },
        None => Err(anyhow!("No hash captured: {}", content))
    }
}
async fn alert_block_hash_is_not_same(alert_manager: &str, shadow_url: &str, block: u64) {
    if let Err(e) = do_alert_block_hash_is_not_same(alert_manager, shadow_url, block).await {
        println!("alert fail by: {:?}", e)
    }
}
async fn do_alert_block_hash_is_not_same(alert_manager: &str, shadow_url: &str, block: u64) -> Result<()> {
    let map: Value = serde_json::from_str(&format! {
        "[{{{}}}]", vec![
            r#""labels": {"#,
            r#"  "alertname": "ShadowBlockHashNotSameAsEtherscan","#,
            &format!(r#"  "shadow": "{}","#, shadow_url),
            &format!(r#"  "block": "{}""#, block),
            r#"},"#,
            r#""generatorUrl": "https://github.com/darwinia-network/postman_rs""#,
        ].concat(),
    })?;

    let client = Client::new();

    let text = client.post(alert_manager).json(&map).send().await?.text().await?;
    println!("{}", text);
    Ok(())
}