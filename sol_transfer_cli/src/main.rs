use std::str::FromStr;
use std::{fs::File, time::Instant};

// use anyhow::{Context, Result};
use anyhow::Result;
use clap::Parser;
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::sol_to_lamports,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    system_instruction,
    transaction::Transaction,
};
use tokio::task;

use std::io::Read;
use std::path::Path;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, default_value = "config.yaml")]
    config: String,
}

#[derive(Debug, Deserialize)]
struct WalletConfig {
    keypair_path: String,
}

#[derive(Debug, Deserialize)]
struct RecipientConfig {
    address: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    senders: Vec<WalletConfig>,
    recipients: Vec<RecipientConfig>,
    amount_sol: f64,
}

pub fn load_keypair_from_file<P: AsRef<Path>>(path: P) -> Result<Keypair> {
    let mut file = File::open(path)?;
    let mut json_data = String::new();
    file.read_to_string(&mut json_data)?;

    let secret_bytes: Vec<u8> = serde_json::from_str(&json_data)?;
    let keypair = Keypair::from_bytes(&secret_bytes)?;

    Ok(keypair)
}

async fn send_transaction(
    client: &RpcClient,
    sender: Keypair,
    recipient: Pubkey,
    lamports: u64,
) -> Result<Signature> {
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[system_instruction::transfer(
            &sender.pubkey(),
            &recipient,
            lamports,
        )],
        Some(&sender.pubkey()),
        &[&sender],
        recent_blockhash,
    );
    let signature = client.send_and_confirm_transaction(&tx)?;
    Ok(signature)
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config_text = std::fs::read_to_string(&cli.config)?;
    let config: Config = serde_yaml::from_str(&config_text)?;

    let client = std::sync::Arc::new(RpcClient::new_with_commitment(
        "https://api.mainnet-beta.solana.com".to_string(),
        CommitmentConfig::confirmed(),
    ));

    let lamports = sol_to_lamports(config.amount_sol);
    let mut tasks = vec![];
    let start_time = Instant::now();

    for (sender_conf, recipient_conf) in config.senders.iter().zip(config.recipients.iter()) {
        let client = client.clone();
        let sender_keypair = load_keypair_from_file(&sender_conf.keypair_path)?;
        let recipient_pubkey = Pubkey::from_str(&recipient_conf.address)?;

        tasks.push(task::spawn(async move {
            let start = Instant::now();
            match send_transaction(&client, sender_keypair, recipient_pubkey, lamports).await {
                Ok(sig) => Ok((sig, start.elapsed())),
                Err(e) => Err(e),
            }
        }));
    }

    let mut success = 0;
    let mut failure = 0;

    for task in tasks {
        match task.await? {
            Ok((sig, duration)) => {
                success += 1;
                println!("✅ Transaction sent: {} ({} ms)", sig, duration.as_millis());
            }
            Err(e) => {
                failure += 1;
                eprintln!("❌ Transaction failed: {:?}", e);
            }
        }
    }

    println!("\n📊 Transaction summary:");
    println!("Successful: {}", success);
    println!("Failed: {}", failure);
    println!("Total time: {} ms", start_time.elapsed().as_millis());

    Ok(())
}
