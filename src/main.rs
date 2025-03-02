#![allow(warnings)]
use clap::Parser;
use colored::Colorize;
use core::fmt;
use prettytable::color::*;
use prettytable::format::Alignment;
use prettytable::{Attr, Cell, Row, Table};
use regex::Regex;
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::io;

const UPGRADE_INDEXER_URL: &str = "https://indexer.upgrade.thegraph.com/status";

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Deployment ID of the subgraph (starts with Qm)
    deployment: String,
}

#[derive(Deserialize, Debug)]
enum Health {
    healthy,
    unhealthy,
    failed,
}

#[derive(Deserialize, Debug)]
struct Response {
    data: SubgraphData,
}

#[derive(Deserialize, Debug)]
struct SubgraphData {
    subgraphFeatures: SubgraphFeatures,
    indexingStatuses: Vec<IndexingStatus>,
}

#[derive(Deserialize, Debug)]
struct SubgraphFeatures {
    apiVersion: Option<String>,
    dataSources: Vec<String>,
    features: Vec<String>,
    specVersion: String,
    handlers: Vec<String>,
    network: String,
}

#[derive(Deserialize, Debug)]
struct IndexingStatus {
    subgraph: String,
    health: Health,
    entityCount: String,
    node: Option<String>,
    paused: Option<bool>,
    synced: bool,
    historyBlocks: i32,
    fatalError: Option<SubgraphError>,
    nonFatalErrors: Vec<SubgraphError>,
    chains: Vec<ChainIndexingStatus>,
}
#[derive(Deserialize, Debug)]
struct SubgraphError {
    message: String,
    block: Option<Block>,
    handlers: Option<String>,
    deterministic: bool,
}
#[derive(Clone, Deserialize, Debug)]
struct Block {
    number: String,
}
#[derive(Deserialize, Debug)]
struct ChainIndexingStatus {
    network: String,
    chainHeadBlock: Block,
    earliestBlock: Block,
    latestBlock: Option<Block>,
}
#[derive(Serialize)]
struct GraphqlQuery<'a> {
    query: &'a str,
}

impl fmt::Display for Health {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

fn main() {
    let args = Args::parse();

    let deployment_id = &args.deployment;
    if (deployment_id.starts_with("Qm") && deployment_id.len() == 46) {
        match get_subgraph_status(deployment_id) {
            Ok(res) => display_status(&res),
            Err(err) => {
                println!("Failed to fetch status: {}", err);
            }
        }
    } else {
        println!("{} is not a valid deployment ID.", deployment_id.yellow());
    }
}

fn get_graft_values(yaml_str: &str) -> Option<(String, u64)> {
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

fn get_status_url() -> String {
    let status_url =
        env::var("SUBGRAPH_STATUS_URL").unwrap_or_else(|_| UPGRADE_INDEXER_URL.to_string());

    match status_url.as_str() {
        url if url.ends_with("/status") => url.to_string(),
        url if url.ends_with('/') => format!("{}status", url),
        url => format!("{}/status", url),
        _ => {
            println!("{}", "Invalid SUBGRAPH_STATUS_URL".red());
            std::process::exit(1);
        }
    }
}

#[tokio::main]
async fn get_subgraph_status(deployment_id: &String) -> Result<SubgraphData, reqwest::Error> {
    let url = get_status_url();

    let client = reqwest::Client::new();

    let query = format!(
        r#"
    {{
        subgraphFeatures(subgraphId:"{}"){{
            apiVersion
            specVersion
            network
            handlers
            dataSources
            features
        }}
        indexingStatuses(subgraphs: ["{}"]){{
            subgraph
            synced
            health
            entityCount
            historyBlocks
            node
            paused
            fatalError {{
                message
                handler
                deterministic
            }}
            chains {{
                chainHeadBlock {{
                    number
                }}
                latestBlock {{
                    number
                }}
                earliestBlock {{
                    number
                }}
                network
            }}
            nonFatalErrors{{
                message
                deterministic
                handler
                block{{
                    number
                }}
            }}
        }}
    }}
"#,
        deployment_id, deployment_id
    );

    let req_body: GraphqlQuery = GraphqlQuery { query: &query };

    let response = client.post(url).json(&req_body).send().await?;
    let mut response_json: Response = response.json().await?;

    Ok(response_json.data)
}

#[tokio::main]
async fn get_manifest_as_string(deployment_id: &String) -> Result<String, reqwest::Error> {
    let manifest_url = format!(
        "https://api.thegraph.com/ipfs/api/v0/cat?arg={}",
        deployment_id
    );
    let client = reqwest::Client::new();
    let manifest_response = client.get(manifest_url).send().await?;
    let manifest = manifest_response.text().await?;
    Ok(manifest)
}

fn get_start_block(manifest: &String) -> String {
    let re = Regex::new(r"startBlock:\s*(\d+)").unwrap();
    let mut start_blocks: Vec<u64> = re
        .captures_iter(manifest)
        .filter_map(|cap| cap[1].parse::<u64>().ok())
        .collect();

    start_blocks
        .iter()
        .min()
        .map(|min| min.to_string())
        .unwrap_or_else(|| String::from("0"))
}

fn display_status(subgraph_data: &SubgraphData) {
    if subgraph_data.indexingStatuses.len() == 0 {
        println!("{}", "No Matches for Deployment ID found".bright_red());
        return;
    }

    let manifest = get_manifest_as_string(&subgraph_data.indexingStatuses[0].subgraph).unwrap();

    let start_block: i64 = get_start_block(&manifest)
        .parse()
        .expect("start_block is Not a valid number");

    let graft_values = get_graft_values(&manifest);

    let mut table = Table::new();

    table.add_row(Row::new(vec![Cell::new_align(
        "Subgraph Status",
        Alignment::CENTER,
    )
    .with_style(Attr::ForegroundColor(BRIGHT_YELLOW))
    .with_hspan(2)]));

    table.add_row(Row::new(vec![
        Cell::new("Deployment ID"),
        Cell::new(&subgraph_data.indexingStatuses[0].subgraph),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("Synced"),
        Cell::new(if subgraph_data.indexingStatuses[0].synced {
            "✅"
        } else {
            "❌"
        }),
    ]));

    let mut health_status_txt_clr: u32;

    match subgraph_data.indexingStatuses[0].health {
        Health::healthy => {
            health_status_txt_clr = GREEN;
        }
        Health::unhealthy => {
            health_status_txt_clr = YELLOW;
        }
        Health::failed => {
            health_status_txt_clr = RED;
        }
    }

    table.add_row(Row::new(vec![
        Cell::new("Health"),
        Cell::new(&subgraph_data.indexingStatuses[0].health.to_string())
            .with_style(Attr::ForegroundColor(health_status_txt_clr)),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("Entity Count"),
        Cell::new(&subgraph_data.indexingStatuses[0].entityCount),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("Paused"),
        Cell::new(
            if let Some(paused) = subgraph_data.indexingStatuses[0].paused {
                if paused {
                    "✅"
                } else {
                    "❌"
                }
            } else {
                "N/A"
            },
        ),
    ]));

    let latest_block: i64 = if subgraph_data.indexingStatuses[0].chains[0]
        .latestBlock
        .is_some()
    {
        subgraph_data.indexingStatuses[0].chains[0]
            .latestBlock
            .as_ref()
            .unwrap()
            .number
            .parse()
            .expect("Not a valid number")
    } else {
        0
    };

    let chain_head_block: i64 = subgraph_data.indexingStatuses[0].chains[0]
        .chainHeadBlock
        .number
        .parse()
        .expect("Not a valid number");

    let earliest_block: i64 = subgraph_data.indexingStatuses[0].chains[0]
        .earliestBlock
        .number
        .parse()
        .expect("Not a valid number");

    let blocks_behind = chain_head_block - latest_block;

    table.add_row(Row::new(vec![
        Cell::new("Synced"),
        Cell::new(&get_sync_percentage(
            start_block,
            latest_block,
            chain_head_block,
        )),
    ]));

    let blocks_behind_txt_clr: u32;

    match blocks_behind {
        blocks_behind if blocks_behind < 30 => {
            blocks_behind_txt_clr = GREEN;
        }
        blocks_behind if blocks_behind < 1000 => {
            blocks_behind_txt_clr = YELLOW;
        }
        _ => {
            blocks_behind_txt_clr = RED;
        }
    }

    table.add_row(Row::new(vec![
        Cell::new("Blocks Behind"),
        Cell::new(&blocks_behind.to_string())
            .with_style(Attr::ForegroundColor(blocks_behind_txt_clr)),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("Chain Head Block"),
        Cell::new(&chain_head_block.to_string()),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("Latest Block"),
        Cell::new(&latest_block.to_string()),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("Start Block"),
        Cell::new(&start_block.to_string()),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("Earliest Block"),
        Cell::new(&earliest_block.to_string()),
    ]));

    let pruning = if subgraph_data.indexingStatuses[0].historyBlocks == i32::MAX {
        "❌"
    } else {
        &subgraph_data.indexingStatuses[0].historyBlocks.to_string()
    };

    table.add_row(Row::new(vec![Cell::new("Pruning"), Cell::new(pruning)]));

    table.add_row(Row::new(vec![
        Cell::new("Network"),
        Cell::new(&subgraph_data.indexingStatuses[0].chains[0].network),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("Node"),
        Cell::new(
            if let Some(node) = &subgraph_data.indexingStatuses[0].node {
                &node
            } else {
                "N/A"
            },
        ),
    ]));

    if graft_values.is_some() {
        table.add_row(Row::new(vec![
            Cell::new("Graft Base"),
            Cell::new(&graft_values.as_ref().unwrap().0),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Graft Block"),
            Cell::new(&graft_values.as_ref().unwrap().1.to_string()),
        ]));
    }

    table.add_row(Row::new(vec![Cell::new_align(
        "Subgraph Features",
        Alignment::CENTER,
    )
    .with_style(Attr::ForegroundColor(BRIGHT_YELLOW))
    .with_hspan(2)]));

    table.add_row(Row::new(vec![
        Cell::new("Spec Version"),
        Cell::new(&subgraph_data.subgraphFeatures.specVersion),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("API Version"),
        Cell::new(
            &subgraph_data
                .subgraphFeatures
                .apiVersion
                .clone()
                .unwrap_or(String::from("N/A")),
        ),
    ]));

    let mut handlers_cell = String::new();

    for handler in &subgraph_data.subgraphFeatures.handlers {
        handlers_cell
            .push_str(&(capitalize_first_letter(handler).to_string() + " Handler " + "\n"));
    }

    table.add_row(Row::new(vec![
        Cell::new("Handlers Used"),
        Cell::new(&handlers_cell),
    ]));

    let mut data_source_cell = String::new();
    for data_souce in &subgraph_data.subgraphFeatures.dataSources {
        data_source_cell.push_str(&(data_souce.to_string() + "\n"));
    }

    if !subgraph_data.subgraphFeatures.dataSources.is_empty() {
        table.add_row(Row::new(vec![
            Cell::new("Data Sources"),
            Cell::new(&data_source_cell),
        ]));
    }

    let mut features = subgraph_data.subgraphFeatures.features.clone();

    if graft_values.is_some() && !features.contains(&"grafting".to_string()) {
        features.push("grafting".to_string());
    }

    if graft_values.is_none() && features.contains(&"grafting".to_string()) {
        features.retain(|x| x != "grafting");
    }

    let features_cell = if features.is_empty() {
        "N/A".to_string()
    } else {
        features.join("\n")
    };

    table.add_row(Row::new(vec![
        Cell::new("Features Used"),
        Cell::new(&features_cell),
    ]));

    table.printstd();

    if let Some(fatalError) = subgraph_data.indexingStatuses[0].fatalError.as_ref() {
        println!("\n{}", "Fatal Errors".bright_yellow());
        println!("\nMessage: {}", fatalError.message.red());
    }

    if subgraph_data.indexingStatuses[0].nonFatalErrors.len() > 0 {
        let nonFatalError = &subgraph_data.indexingStatuses[0].nonFatalErrors[0];
        println!("\n{}", "Non Fatal Errors".bright_yellow());
        println!("\nMessage: {}", nonFatalError.message.red());
    }
}

fn get_sync_percentage(start_block: i64, latest_block: i64, chain_head_block: i64) -> String {
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

fn capitalize_first_letter(word: &String) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first_char) => first_char.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
