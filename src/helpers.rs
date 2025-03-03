use colored::Colorize;
use regex::Regex;
use std::env;

use crate::api::get_latest_crate_version;

const UPGRADE_INDEXER_URL: &str = "https://indexer.upgrade.thegraph.com/status";

pub fn get_graft_values(yaml_str: &str) -> Option<(String, u64)> {
    let base_re = Regex::new(r"graft:\s*\n\s*base:\s*(\S+)").ok()?;
    let block_re = Regex::new(r"block:\s*(\d+)").ok()?;

    let base = base_re.captures(yaml_str)?.get(1)?.as_str().to_string();
    let block = block_re
        .captures(yaml_str)?
        .get(1)?
        .as_str()
        .parse::<u64>()
        .ok()?;

    Some((base, block))
}

pub fn get_status_url(local: &bool) -> String {
    if *local {
        return "http://localhost:8030/graphql".to_string();
    }

    let status_url =
        env::var("SUBGRAPH_STATUS_URL").unwrap_or_else(|_| UPGRADE_INDEXER_URL.to_string());

    match status_url.as_str() {
        url if url.ends_with("/status") => url.to_string(),
        url if url.ends_with('/') => format!("{}status", url),
        url => format!("{}/status", url),
    }
}

pub fn get_start_block(manifest: &String) -> String {
    let re = Regex::new(r"startBlock:\s*(\d+)").unwrap();
    let start_blocks: Vec<u64> = re
        .captures_iter(manifest)
        .filter_map(|cap| cap[1].parse::<u64>().ok())
        .collect();

    start_blocks
        .iter()
        .min()
        .map(|min| min.to_string())
        .unwrap_or_else(|| String::from("0"))
}

pub fn get_sync_percentage(start_block: i64, latest_block: i64, chain_head_block: i64) -> String {
    if latest_block == 0 {
        return String::from("N/A");
    }
    let blocks_processed = latest_block - start_block;
    let total_blocks = chain_head_block - start_block;
    let synced = (blocks_processed * 100) / total_blocks;
    if synced > 100 {
        return String::from("100%");
    }
    return synced.to_string() + "%";
}

pub fn capitalize_first_letter(word: &String) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first_char) => first_char.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub async fn check_for_updates() -> Result<(), reqwest::Error> {
    let latest_version = get_latest_crate_version().await?;
    // get current version
    let current_version = env!("CARGO_PKG_VERSION");

    if current_version != latest_version {
        println!(
            "🚨 Warning: subgraph-status update available from {} to {}",
            current_version.yellow(),
            latest_version.green()
        );
        println!(
            "🚀 Run {} to update.",
            "cargo install subgraph-status".green()
        );
    }
    Ok(())
}
