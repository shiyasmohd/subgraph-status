use api::get_subgraph_status;
use clap::Parser;
use colored::Colorize;
use helpers::{check_for_updates, get_status_url};
use output::display_status;
use std::collections::BTreeSet;

mod api;
mod helpers;
mod output;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Deployment ID of the subgraph (starts with Qm)
    deployments: Vec<String>,

    /// Fetch status from local graph-node
    #[clap(long, short)]
    local: bool,
}

fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        if let Err(e) = check_for_updates().await {
            println!("Failed to check for updates: {}", e);
        }
    });

    let args = Args::parse();

    let (valid_deployment_ids, invalid_deployment_ids): (BTreeSet<_>, BTreeSet<_>) = args
        .deployments
        .into_iter()
        .partition(|id| id.starts_with("Qm") && id.len() == 46);

    if !invalid_deployment_ids.is_empty() {
        println!(
            "Invalid deployment IDs: {}",
            invalid_deployment_ids
                .into_iter()
                .collect::<Vec<_>>()
                .join(", ")
                .red()
        );
    }

    for deployment_id in valid_deployment_ids {
        let url = get_status_url(&args.local);
        match get_subgraph_status(url, &deployment_id) {
            Ok(res) => display_status(&res),
            Err(err) => {
                println!("Failed to fetch status: {}", err);
            }
        }
    }
}
