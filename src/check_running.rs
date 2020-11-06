use anyhow::Result;
use web3::types::{BlockId, BlockNumber};
use tokio::time::{delay_for, Duration};
use async_recursion::async_recursion;
use serde_json::Value;
use reqwest::Client;

/// CHECK RUNNING
/// compare the count with ethereum height,
/// alert if the difference between the two exceeds the threshold
#[async_recursion]
pub async fn start(shadow_url: &str, ethereum_url: &str, alert_manager: &str, gap_threshold: u64) -> Result<()> {
    log::info!("checking running...");
    match calc_gap(shadow_url, ethereum_url).await {
        Ok(gap) => {
            if gap > gap_threshold {
                log::error!("shadow may have stopped, gap: {}", gap);
                alert_shadow_may_stopped(alert_manager, shadow_url, gap).await;
                delay_for(Duration::from_millis(600000)).await; // 10 minutes
                start(shadow_url, ethereum_url, alert_manager, gap_threshold).await
            } else {
                delay_for(Duration::from_millis(60000)).await; // 1 minute
                start(shadow_url, ethereum_url, alert_manager, gap_threshold).await
            }
        }
        Err(e) => {
            log::error!("check_running error: {:?}, wait 10 minutes to retry", e);
            delay_for(Duration::from_millis(600000)).await; // 10 minutes
            start(shadow_url, ethereum_url, alert_manager, gap_threshold).await
        }
    }
}

async fn calc_gap(shadow_url: &str, ethereum_url: &str) -> Result<u64> {
    let last_mmr_leaf = latest_mmr_leaf(&shadow_url).await?;
    let latest_block_number = latest_block_number(&ethereum_url).await?;

    let gap = latest_block_number - last_mmr_leaf;
    Ok(gap)
}

async fn latest_mmr_leaf(shadow_url: &str) -> Result<u64> {
    let resp = reqwest::get(format!("{}/ethereum/count", shadow_url).as_str()).await?;
    let content = resp.text().await?;
    let last_mmr_leaf = content.parse::<u64>()?;
    Ok(last_mmr_leaf)
}

async fn latest_block_number(ethereum_url: &str) -> Result<u64> {
    let transport = web3::transports::Http::new(ethereum_url)?;
    let web3 = web3::Web3::new(transport);
    let block = web3
        .eth()
        .block(BlockId::Number(BlockNumber::Latest))
        .await?
        .unwrap();
    Ok(block.number.unwrap().as_u64())
}

async fn alert_shadow_may_stopped(alert_manager: &str, shadow_url: &str, gap: u64) {
    if let Err(e) = do_alert_shadow_may_stopped(alert_manager, shadow_url, gap).await {
        log::error!("alert fail by: {:?}", e)
    }
}

async fn do_alert_shadow_may_stopped(alert_manager: &str, shadow_url: &str, gap: u64) -> Result<()> {
    let string = format! {
        "[{{{}}}]", vec![
            r#""labels": {"#,
            r#""alertname": "ShadowMayStopped","#,
            &format!(r#""shadow": "{}","#, shadow_url),
            &format!(r#""gap": "{}""#, gap),
            r#"},"#,
            r#""generatorUrl": "https://github.com/darwinia-network/postman_rs""#,
        ].concat(),
    };
    let map: Value = serde_json::from_str(&string)?;
    let client = Client::new();
    let _text = client.post(alert_manager).json(&map).send().await?.text().await?;
    Ok(())
}