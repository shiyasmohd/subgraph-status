<div align="center" style="font-family:'Montserrat', sans-serif;">

## 📊 Subgraph Status
🖥️  &nbsp;CLI Application to Check Your Subgraph's Status 
<br/>
<br/>
[![Crates.io](https://img.shields.io/crates/v/subgraph-status?style=flat-square)](https://crates.io/crates/subgraph-status)
[![Crates.io](https://img.shields.io/crates/d/subgraph-status?style=flat-square)](https://crates.io/crates/subgraph-status)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](LICENSE-MIT)
[![Contributors](https://img.shields.io/github/contributors/shiyasmohd/subgraph-status?style=flat-square)](https://github.com/shiyasmohd/subgraph-status/graphs/contributors)

</div>

<video controls autoplay loop src='https://github.com/user-attachments/assets/eaec9da2-3ace-46a3-b076-e96a1aeb9330' width="60%"></video>
</div>

## Prequsites 🛠️
- Rust - [Install Rust](https://doc.rust-lang.org/book/ch01-01-installation.html)

## Installation 💻
```
cargo install subgraph-status
```

## Usage
```
subgraph-status <DEPLOYMENT_ID>
```

Example
```
subgraph-status QmUfcf55AoqVs3zuyfu9TwYh8G4LxRnY5DpxjVCea3RSWe
```

## How to fetch details from a specific indexer ❓

1. Visit the [Graph Network Arbitrum Subgraph](https://thegraph.com/explorer/subgraphs/DZz4kDTdmzWLWsV373w2bSmoar3umKKH9y82SUKr5qmp?view=Query&chain=arbitrum-one).

2. Execute the following query with the desired indexer address:
    ```graphql
    {
      indexer(id: "INDEXER_ADDRESS") {
        url
      }
    }
    ```

3. Set the resulting URL as an environment variable:
    ```sh
    export SUBGRAPH_STATUS_URL="resulting_url"
    ```

4. You're all set! The package will now fetch the status from the specified indexer.

## How to run locally ❓
1. Clone repository & change directory
```
git clone https://github.com/Shiyasmohd/subgraph-status.git
cd subgraph-status
```
2. Copy Deployment ID of the Subgraph (Starts with Qm..)

3. Run Program
```
cargo run <DEPLOYMENT_ID>
```
Example Command
```
cargo run QmUfcf55AoqVs3zuyfu9TwYh8G4LxRnY5DpxjVCea3RSWe
```
