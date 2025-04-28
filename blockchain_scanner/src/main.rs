use reqwest::Error;
use reqwest::header::{HeaderMap, HeaderValue};
use tokio::time::{sleep, Duration};
use serde::Deserialize;
use dotenv::dotenv;
use std::env;
extern crate diesel;
mod schema;
mod model;
use crate::model::establish_connection;
use diesel::prelude::*;
use diesel::sql_query;

#[derive(Debug, Deserialize)]
struct BlockchainInfoRequest {
    result: BlockchainInfo,
}

#[derive(Debug, Deserialize)]
struct BlockchainInfo {
    blocks: u64,
    bestblockhash: String,
}

#[derive(Debug, Deserialize)]
struct BlockRequest {
    result: Block,
}

#[derive(Debug, Deserialize)]
struct Block {
    confirmations: u64,
    nTx: u64,
    nextblockhash: Option<String>,
    tx: Vec<Tx>
}

#[derive(Debug, Deserialize)]
struct Tx {
    txid: String,
    vout: Vec<Vout>
}

#[derive(Debug, Deserialize)]
struct Vout {
    scriptPubKey: ScriptPubKey
}

#[derive(Debug, Deserialize)]
struct ScriptPubKey {
    asm: String,
    hex: String,
}

#[derive(Insertable)]
#[diesel(table_name = schema::dns_mapping)]
pub struct NewDnsMapping {
    pub tx_id: String,
    pub key: String,
    pub nostr_address: String,
    pub active: bool,
}

#[derive(Debug, QueryableByName)]
#[diesel(table_name = schema::dns_mapping)]
pub struct DnsMapping {
    pub id: i32,
    pub tx_id: String,
    pub key: String,
    pub nostr_address: String,
    pub active: bool,
}

async fn scan_latest() -> Result<(), Error> {
    let data = "{\"method\":\"getblockchaininfo\"}";
    let headers = setup_headers();

    let rpc_user = env::var("RPC_USER").expect("RPC_USER must be set");
    let rpc_password = env::var("RPC_PASSWORD").expect("RPC_PASSWORD must be set");

    let client = reqwest::Client::new();
    let url = env::var("BASE_URL").expect("BASE_URL must be set");

    let response = client.post(url).body(data).headers(headers).basic_auth(rpc_user, Some(rpc_password)).send().await?;
    let data = response.text().await?;

    let parsed_data: BlockchainInfoRequest = serde_json::from_str(&data).expect("JSON was not well-formatted");

    scan_block(parsed_data.result.bestblockhash).await?;

    Ok(())
}

async fn scan_block(block_hash: String) -> Result<(), Error> {
    let data = format!("{{\"method\":\"getblock\",\"params\":[\"{block_hash}\", 2]}}");
    let headers = setup_headers();

    let rpc_user = env::var("RPC_USER").expect("RPC_USER must be set");
    let rpc_password = env::var("RPC_PASSWORD").expect("RPC_PASSWORD must be set");

    let client = reqwest::Client::new();
    let url = env::var("BASE_URL").expect("BASE_URL must be set");

    let response = client.post(url).body(data).headers(headers).basic_auth(rpc_user, Some(rpc_password)).send().await?;
    let data = response.text().await?;

    let parsed_data: BlockRequest = serde_json::from_str(&data).expect("JSON was not well-formatted");

    for tx in parsed_data.result.tx {
        let asm = tx.vout[0].scriptPubKey.asm.split(" ").collect::<Vec<&str>>();

        // Standard size of dns mapping transaction
        if asm.len() == 12 {
            if asm[3] == "444e53" {
                create_new_dns_mapping(tx.txid, asm[5].to_string(), asm[7].to_string());
            }
        }
    }

    Ok(())
}

fn create_new_dns_mapping(tx_id: String, key: String, nostr_address: String) {
    let mut connection = establish_connection();

    let active = check_existing_dns_mapping(key.clone());

    let new_dns_mapping = NewDnsMapping {
        tx_id,
        key,
        nostr_address,
        active,
    };

    diesel::insert_into(schema::dns_mapping::table)
        .values(&new_dns_mapping)
        .execute(&mut connection)
        .expect("Error saving new dns mapping");
}

fn check_existing_dns_mapping(key: String) -> bool {
    let mut connection = establish_connection();

    let dns_mappings: Vec<DnsMapping> = sql_query("SELECT * FROM dns_mapping WHERE key = $1 AND active = TRUE")
        .bind::<diesel::sql_types::Text, _>(key)
        .load(&mut connection)
        .expect("Error loading dns mapping");

    if dns_mappings.len() > 0 {
        return false;
    }

    return true;
}

fn setup_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    let content_type = HeaderValue::from_static("application/json");
    let cache_control = HeaderValue::from_static("no-cache");
    headers.insert("content-type", content_type);
    headers.insert("cache-control", cache_control);

    headers
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    loop {
        if let Err(e) = scan_latest().await {
            eprintln!("Error: {}", e);
        }
        sleep(Duration::from_secs(600)).await;
    }
}


