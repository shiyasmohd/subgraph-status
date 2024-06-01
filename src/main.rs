#![allow(warnings)]
use colored::Colorize;
use prettytable::color::*;
use prettytable::format::Alignment;
use prettytable::{Attr, Cell, Row, Table};
use serde::Deserialize;
use serde::Serialize;
use std::env;
use std::io;

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
    apiVersion: String,
    dataSources: Vec<String>,
    features: Vec<String>,
    specVersion: String,
    handlers: Vec<String>,
    network: String,
}

#[derive(Deserialize, Debug)]
struct IndexingStatus {
    subgraph: String,
    health: String,
    entityCount: String,
    node: String,
    paused: bool,
    synced: bool,
    historyBlocks: i64,
    fatalError: Option<SubgraphError>,
    nonFatalErrors: Vec<Option<SubgraphError>>,
    chains: Vec<ChainIndexingStatus>,
}
#[derive(Deserialize, Debug)]
struct SubgraphError {
    message: String,
    block: Block,
    handlers: Option<String>,
    deterministic: bool,
}
#[derive(Deserialize, Debug)]
struct Block {
    hash: String,
    number: String,
}
#[derive(Deserialize, Debug)]
struct ChainIndexingStatus {
    network: String,
    chainHeadBlock: Block,
    earliestBlock: Block,
    latestBlock: Block,
    lastHealthyBlock: Option<Block>,
}
#[derive(Serialize)]
struct GraphqlQuery<'a> {
    query: &'a str,
}

fn main() {
    // let deployment_id = "QmRQUYU2HNXDQdWCbcif8iLCxnoNcz8jdtJiVJJXAyKgjk";
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        // The first argument (args[0]) is the program name, so we take the second one
        let deployment_id = &args[1];
        match get_subgraph_status(deployment_id.to_string()) {
            Ok(res) => display_status(res),
            Err(err) => {
                println!("Failed to fetch status: {}", err);
            }
        }
    } else {
        println!("{}", "Please provide Deployment ID of subgraph".red());
    }
}

#[tokio::main]
async fn get_subgraph_status(deployment_id: String) -> Result<SubgraphData, reqwest::Error> {
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
                block {{
                    hash
                    number
                }}
                deterministic
            }}
            chains {{
                chainHeadBlock {{
                    hash
                    number
                }}
                latestBlock {{
                    hash
                    number
                }}
                earliestBlock {{
                    hash
                    number
                }}
                lastHealthyBlock{{
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
                    hash
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
    let response_json: Response = response.json().await?;

    Ok(response_json.data)
}

fn display_status(subgraph_data: SubgraphData) {
    let mut table = Table::new();

    table.add_row(Row::new(vec![Cell::new_align(
        "Subgraph Status",
        Alignment::CENTER,
    )
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

    table.add_row(Row::new(vec![
        Cell::new("Health"),
        Cell::new(&subgraph_data.indexingStatuses[0].health),
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

    let latest_block: i64 = subgraph_data.indexingStatuses[0].chains[0]
        .latestBlock
        .number
        .parse()
        .expect("Not a valid number");

    let chain_head_block: i64 = subgraph_data.indexingStatuses[0].chains[0]
        .chainHeadBlock
        .number
        .parse()
        .expect("Not a valid number");

    let blocks_behind = chain_head_block - latest_block;

    table.add_row(Row::new(vec![
        Cell::new("Blocks Behind"),
        Cell::new(&blocks_behind.to_string()),
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
        Cell::new("History Blocks"),
        Cell::new(&subgraph_data.indexingStatuses[0].historyBlocks.to_string()),
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
    .with_hspan(2)]));

    table.add_row(Row::new(vec![
        Cell::new("Spec Version"),
        Cell::new(&subgraph_data.subgraphFeatures.specVersion),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("API Version"),
        Cell::new(&subgraph_data.subgraphFeatures.apiVersion),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("Event Handler"),
        Cell::new(
            if subgraph_data
                .subgraphFeatures
                .handlers
                .contains(&"event".to_string())
            {
                "✅"
            } else {
                "❌"
            },
        ),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("Call Handler"),
        Cell::new(
            if subgraph_data
                .subgraphFeatures
                .handlers
                .contains(&"call".to_string())
            {
                "✅"
            } else {
                "❌"
            },
        ),
    ]));

    table.add_row(Row::new(vec![
        Cell::new("Block Handler"),
        Cell::new(
            if subgraph_data
                .subgraphFeatures
                .handlers
                .contains(&"block".to_string())
            {
                "✅"
            } else {
                "❌"
            },
        ),
    ]));

    if !subgraph_data.subgraphFeatures.dataSources.is_empty() {
        table.add_row(Row::new(vec![
            Cell::new("Data Sources"),
            Cell::new(&subgraph_data.subgraphFeatures.dataSources[0]),
        ]));
        for i in subgraph_data.subgraphFeatures.dataSources.iter().skip(1) {
            table.add_row(Row::new(vec![Cell::new(""), Cell::new(i)]));
        }
    }

    table.printstd();

    if subgraph_data.indexingStatuses[0].fatalError.is_some() {
        println!("\n{}", "Fatal Errors".bright_yellow());
        println!(
            "\nMessage: {}",
            subgraph_data.indexingStatuses[0]
                .fatalError
                .as_ref()
                .unwrap()
                .message
                .red()
        );
        println!(
            "Block: {}",
            subgraph_data.indexingStatuses[0]
                .fatalError
                .as_ref()
                .unwrap()
                .block
                .number
                .bright_yellow()
        );
    }
    if !subgraph_data.indexingStatuses[0].nonFatalErrors.is_empty() {
        println!("\n{}", "Non Fatal Errors".bright_yellow());
        println!(
            "\nMessage: {}",
            subgraph_data.indexingStatuses[0].nonFatalErrors[0]
                .as_ref()
                .unwrap()
                .message
                .red()
        );
        println!(
            "Block: {}",
            subgraph_data.indexingStatuses[0].nonFatalErrors[0]
                .as_ref()
                .unwrap()
                .block
                .number
                .bright_yellow()
        );
    }
}
