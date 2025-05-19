use clap::Parser;
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signature, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::{fs::File, sync::Arc};
use yellowstone_grpc_client::GeyserGrpcClient;
// use tokio::sync::Mutex;

use futures::{sink::SinkExt, stream::StreamExt};
use yellowstone_grpc_client::ClientTlsConfig;
use yellowstone_grpc_proto::prelude::{
    subscribe_update::UpdateOneof, CommitmentLevel, SubscribeRequest, SubscribeRequestFilterSlots,
    SubscribeRequestPing, SubscribeUpdatePong, SubscribeUpdateSlot,
};

use tokio::time::{interval, Duration};

#[derive(Parser, Debug)]
#[command(name = "Geyser GRPC Listener")]
struct Cli {
    #[arg(short, long, default_value = "config.yaml")]
    config: String,
}

#[derive(Debug, Deserialize)]
struct Config {
    private_key_path: String,
    recipient_address: String,
    rpc_url: String,
    grpc_url: String,
    x_token: Option<String>,
}

// Заглушка Geyser-клиента (реализуется отдельно по протоколу GRPC)
// async fn subscribe_to_blocks<F>(mut on_block: F)
// where
//     F: FnMut() + Send + 'static,
// {
//     // Здесь будет GRPC соединение и подписка на новые блоки
//     // Например, используя tonic::transport::Channel и сгенерированные protobuf-клиенты
//     loop {
//         tokio::time::sleep(std::time::Duration::from_secs(15)).await;
//         on_block(); // вызывается при получении нового блока
//     }
// }

fn send_transaction(
    client: &RpcClient,
    payer: &Keypair,
    recipient: &Pubkey,
    lamports: u64,
) -> Result<Signature, Box<dyn std::error::Error>> {
    let instruction = system_instruction::transfer(&payer.pubkey(), recipient, lamports);
    let recent_blockhash = client.get_latest_blockhash()?;
    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );
    let sig = client.send_and_confirm_transaction(&tx)?;
    Ok(sig)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let config_file = File::open(cli.config)?;
    let config: Config = serde_yaml::from_reader(config_file)?;

    let payer = read_keypair_file(&config.private_key_path)?;
    let recipient: Pubkey = config.recipient_address.parse()?;
    let rpc_client = Arc::new(RpcClient::new_with_commitment(
        config.rpc_url.clone(),
        CommitmentConfig::confirmed(),
    ));

    let rpc_clone = rpc_client.clone();
    let payer = Arc::new(payer);
    let payer_clone = payer.clone();

    let recipient_clone = recipient.clone();
    let lamports_to_send = 10000; // 0.00001 SOL

    let mut client = GeyserGrpcClient::build_from_shared(config.grpc_url)?
        .x_token(config.x_token)?
        .tls_config(ClientTlsConfig::new().with_native_roots())?
        .connect()
        .await?;

    let (mut subscribe_tx, mut stream) = client.subscribe().await?;

    futures::try_join!(
        async move {
            subscribe_tx
                .send(SubscribeRequest {
                    slots: maplit::hashmap! {
                        "".to_owned() => SubscribeRequestFilterSlots {
                            filter_by_commitment: Some(true),
                            interslot_updates: Some(false)
                        }
                    },
                    commitment: Some(CommitmentLevel::Processed as i32),
                    ..Default::default()
                })
                .await?;

            let mut timer = interval(Duration::from_secs(3));
            let mut id = 0;
            loop {
                timer.tick().await;
                id += 1;
                subscribe_tx
                    .send(SubscribeRequest {
                        ping: Some(SubscribeRequestPing { id }),
                        ..Default::default()
                    })
                    .await?;
            }
            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        },
        async move {
            while let Some(message) = stream.next().await {
                match message?.update_oneof.expect("valid message") {
                    UpdateOneof::Slot(SubscribeUpdateSlot { slot: _, .. }) => {
                        // info!("slot received: {slot}");
                        let client = rpc_clone.clone();
                        let payer = payer_clone.clone();
                        let recipient = recipient_clone.clone();

                        tokio::spawn(async move {
                            println!("Новый блок обнаружен. Отправка SOL...");

                            match send_transaction(&client, &payer, &recipient, lamports_to_send) {
                                Ok(sig) => println!("Транзакция отправлена: {}", sig),
                                Err(e) => eprintln!("Ошибка отправки: {}", e),
                            }
                        });
                    }
                    UpdateOneof::Ping(_msg) => {
                        println!("ping received");
                    }
                    UpdateOneof::Pong(SubscribeUpdatePong { id }) => {
                        println!("pong received: id#{id}");
                    }
                    msg => anyhow::bail!("received unexpected message: {msg:?}"),
                }
            }
            Ok::<(), anyhow::Error>(())
        }
    )?;

    // subscribe_to_blocks(move || {
    //     let client = rpc_clone.clone();
    //     let payer = payer_clone.clone();
    //     let recipient = recipient_clone.clone();
    //
    //     tokio::spawn(async move {
    //         println!("Новый блок обнаружен. Отправка SOL...");
    //
    //         match send_transaction(&client, &payer, &recipient, lamports_to_send) {
    //             Ok(sig) => println!("Транзакция отправлена: {}", sig),
    //             Err(e) => eprintln!("Ошибка отправки: {}", e),
    //         }
    //     });
    // })
    // .await;

    Ok(())
}
