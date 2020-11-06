use anyhow::Result;
use config::*;

mod check_running;
mod check_block_hash;

#[tokio::main]
async fn main() -> Result<()> {
    // read config
    let mut settings = Config::default();
    settings.merge(File::with_name("config.toml"))?;
    let shadow_url = settings.get_str("shadow")?;
    let alert_manager = settings.get_str("alert_manager")?;

    let ethereum_url = settings.get_str("ethereum")?;
    let gap_threshold = settings.get_int("gap_threshold")? as u64;

    let etherscan_apikey = settings.get_str("etherscan_apikey")?;
    let start_block = settings.get_int("start_block")? as u64;

    // checking
    // TODO: check mmr_root between two shadow instance
    let (first, second) = tokio::join!(
        check_running::start(&shadow_url, &ethereum_url, &alert_manager, gap_threshold),
        check_block_hash::start(&shadow_url, &etherscan_apikey, &alert_manager, start_block),
    );

    first.and(second)
}

