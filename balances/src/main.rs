use futures::future::join_all;
use serde::Deserialize;
use serde_json::json;
use std::fs::File;
use std::io::BufReader;

#[derive(Debug, Deserialize)]
struct Config {
    wallets: Vec<String>,
    rpc_url: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Загрузка конфигурации
    let config: Config = load_config("config.yaml")?;

    // Параллельные запросы балансов
    let client = reqwest::Client::new();

    let balance_futures = config
        .wallets
        .iter()
        .map(|wallet| get_balance(&client, &config.rpc_url, wallet.to_string()));

    let balances = join_all(balance_futures).await;

    for (wallet, balance) in config.wallets.iter().zip(balances) {
        match balance {
            Ok(lamports) => println!("{} => {} lamports", wallet, lamports),
            Err(e) => eprintln!("Ошибка для {}: {}", wallet, e),
        }
    }

    Ok(())
}

fn load_config(path: &str) -> Result<Config, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let config = serde_yaml::from_reader(reader)?;
    Ok(config)
}

async fn get_balance(
    client: &reqwest::Client,
    rpc_url: &str,
    pubkey: String,
) -> Result<u64, reqwest::Error> {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getBalance",
        "params": [pubkey]
    });

    let res = client
        .post(rpc_url)
        .json(&body)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    Ok(res["result"]["value"].as_u64().unwrap_or(0))
}
