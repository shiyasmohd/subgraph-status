use crate::{
    api::{get_manifest_as_string, Health, SubgraphData},
    helpers::{capitalize_first_letter, get_graft_values, get_start_block, get_sync_percentage},
};
use colored::Colorize;
use prettytable::color::*;
use prettytable::format::Alignment;
use prettytable::{Attr, Cell, Row, Table};

pub fn display_status(subgraph_data: &SubgraphData) {
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

    let health_status_txt_clr: u32;

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

    if let Some(fatal_error) = subgraph_data.indexingStatuses[0].fatalError.as_ref() {
        println!("\n{}", "Fatal Errors".bright_yellow());
        println!("\nMessage: {}", fatal_error.message.red());
    }

    if subgraph_data.indexingStatuses[0].nonFatalErrors.len() > 0 {
        let non_fatal_error = &subgraph_data.indexingStatuses[0].nonFatalErrors[0];
        println!("\n{}", "Non Fatal Errors".bright_yellow());
        println!("\nMessage: {}", non_fatal_error.message.red());
    }
}
