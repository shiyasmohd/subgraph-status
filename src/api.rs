#![allow(warnings)]
use core::fmt;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Debug)]
pub enum Health {
    healthy,
    unhealthy,
    failed,
}

#[derive(Deserialize, Debug)]
struct Response {
    data: SubgraphData,
}
#[derive(Deserialize, Debug)]
pub struct SubgraphData {
    pub subgraphFeatures: SubgraphFeatures,
    pub indexingStatuses: Vec<IndexingStatus>,
}

#[derive(Deserialize, Debug)]
pub struct SubgraphFeatures {
    pub apiVersion: Option<String>,
    pub dataSources: Vec<String>,
    pub features: Vec<String>,
    pub specVersion: String,
    pub handlers: Vec<String>,
    pub network: String,
}

#[derive(Deserialize, Debug)]
pub struct IndexingStatus {
    pub subgraph: String,
    pub health: Health,
    pub entityCount: String,
    pub node: Option<String>,
    pub paused: Option<bool>,
    pub synced: bool,
    pub historyBlocks: i32,
    pub fatalError: Option<SubgraphError>,
    pub nonFatalErrors: Vec<SubgraphError>,
    pub chains: Vec<ChainIndexingStatus>,
}
#[derive(Deserialize, Debug)]
pub struct SubgraphError {
    pub message: String,
    pub block: Option<Block>,
    pub handlers: Option<String>,
    pub deterministic: bool,
}
#[derive(Clone, Deserialize, Debug)]
pub struct Block {
    pub number: String,
}
#[derive(Deserialize, Debug)]
pub struct ChainIndexingStatus {
    pub network: String,
    pub chainHeadBlock: Block,
    pub earliestBlock: Block,
    pub latestBlock: Option<Block>,
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

#[tokio::main]
pub async fn get_subgraph_status(
    url: String,
    deployment_id: &String,
) -> Result<SubgraphData, reqwest::Error> {
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
    let response_json: Response = response.json().await?;

    Ok(response_json.data)
}

#[tokio::main]
pub async fn get_manifest_as_string(deployment_id: &String) -> Result<String, reqwest::Error> {
    let manifest_url = format!(
        "https://api.thegraph.com/ipfs/api/v0/cat?arg={}",
        deployment_id
    );
    let client = reqwest::Client::new();
    let manifest_response = client.get(manifest_url).send().await?;
    let manifest = manifest_response.text().await?;
    Ok(manifest)
}

#[tokio::main]
pub async fn get_subgraph_id(deployment_id: &String) -> Result<String, reqwest::Error> {
    let manifest_url = format!(
        "https://subgraph-status-server.vercel.app/get-subgraph-id?deploymentId={}",
        deployment_id
    );
    let client = reqwest::Client::new();
    let response = client.get(manifest_url).send().await?;
    let subgraph_id = response.text().await?;
    Ok(subgraph_id)
}

pub async fn get_latest_crate_version() -> Result<String, reqwest::Error> {
    let url = String::from("https://subgraph-status-server.vercel.app/get-crate-latest-verision");
    let client = reqwest::Client::new();
    let response = client.get(url).send().await?;
    let subgraph_id = response.text().await?;
    Ok(subgraph_id)
}
