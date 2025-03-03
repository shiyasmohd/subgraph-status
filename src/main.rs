use api::get_subgraph_status;
use clap::Parser;
use colored::Colorize;
use helpers::{check_for_updates, get_status_url};
use output::display_status;

mod api;
mod helpers;
mod output;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Deployment ID of the subgraph (starts with Qm)
    deployment: String,

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

    let deployment_id = &args.deployment;
    if deployment_id.starts_with("Qm") && deployment_id.len() == 46 {
        let url = get_status_url(&args.local);
        match get_subgraph_status(url, deployment_id) {
            Ok(res) => display_status(&res),
            Err(err) => {
                println!("Failed to fetch status: {}", err);
            }
        }
    } else {
        println!("{} is not a valid deployment ID.", deployment_id.yellow());
    }
}
