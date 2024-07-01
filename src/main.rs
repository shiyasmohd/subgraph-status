#![allow(warnings)]
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
    node: String,
    paused: bool,
    synced: bool,
    historyBlocks: i64,
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
    // let deployment_id = "QmRQUYU2HNXDQdWCbcif8iLCxnoNcz8jdtJiVJJXAyKgjk";
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        // The first argument (args[0]) is the program name, so we take the second one
        let deployment_id = &args[1];
        if (deployment_id.starts_with("Qm") && deployment_id.len() == 46) {
            match get_subgraph_status(deployment_id) {
                Ok(res) => display_status(&res),
                Err(err) => {
                    println!("Failed to fetch status: {}", err);
                }
            }
        } else {
            println!("{}", "Please enter correct Deployment ID".red());
        }
    } else {
        println!("{}", "Please provide Deployment ID of subgraph".red());
    }
}

fn get_lowest_start_block(data: &str) -> String {
    let re = Regex::new(r"startBlock:\s*(\d+)").unwrap();

    let mut start_blocks: Vec<u64> = re
        .captures_iter(data)
        .filter_map(|cap| cap[1].parse::<u64>().ok())
        .collect();

    start_blocks
        .iter()
        .min()
        .map(|min| min.to_string())
        .unwrap_or_else(|| String::from("0"))
}

#[tokio::main]
async fn get_subgraph_status(deployment_id: &String) -> Result<SubgraphData, reqwest::Error> {
    const URL: &str = "https://api.thegraph.com/index-node/graphql";
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

    // Send the POST request
    let response = client.post(URL).json(&req_body).send().await?;
    let mut response_json: Response = response.json().await?;

    let manifest_url = format!(
        "https://api.thegraph.com/ipfs/api/v0/cat?arg={}",
        deployment_id
    );
    let manifest_response = client.get(manifest_url).send().await?;
    let manifest = manifest_response.text().await?;

    response_json.data.indexingStatuses[0].chains[0].earliestBlock = Block {
        number: get_lowest_start_block(&manifest),
    };

    Ok(response_json.data)
}

fn display_status(subgraph_data: &SubgraphData) {
    if subgraph_data.indexingStatuses.len() == 0 {
        println!("{}", "No Matches for Deployment ID found".bright_red());
        return;
    }

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
        Cell::new(if subgraph_data.indexingStatuses[0].paused {
            "✅"
        } else {
            "❌"
        }),
    ]));

    let earliest_block: i64 = subgraph_data.indexingStatuses[0].chains[0]
        .earliestBlock
        .number
        .parse()
        .expect("Not a valid number");

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

    let blocks_behind = chain_head_block - latest_block;

    table.add_row(Row::new(vec![
        Cell::new("Synced"),
        Cell::new(&get_sync_percentage(
            earliest_block,
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
        Cell::new("Earliest Block"),
        Cell::new(
            &subgraph_data.indexingStatuses[0].chains[0]
                .earliestBlock
                .number,
        ),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("History Blocks"),
        Cell::new(&subgraph_data.indexingStatuses[0].historyBlocks.to_string()),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("Network"),
        Cell::new(&subgraph_data.indexingStatuses[0].chains[0].network),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("Node"),
        Cell::new(&subgraph_data.indexingStatuses[0].node),
    ]));

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

    let mut features_cell = String::new();
    if subgraph_data.subgraphFeatures.features.len() == 0 {
        features_cell = String::from("N/A");
    } else {
        for feature in &subgraph_data.subgraphFeatures.features {
            features_cell.push_str(&(feature.to_string() + "\n"));
        }
    }
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
        String::from("N/A");
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
