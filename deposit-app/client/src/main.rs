// use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::{
    commitment_config::CommitmentConfig,
    // signature::{read_keypair_file, Keypair, Signer},
    signature::read_keypair_file,
    system_program,
};
// use anchor_client::{Client, Cluster, Program};
use anchor_client::{Client, Cluster};
// use anyhow::Result;
use solana_sdk::declare_id;
use std::error::Error;
use std::rc::Rc;

declare_id!("63QPWD9JifxukoYhdJJLBP3jzZqAt45hfGoNaMvVafFF");

#[derive(
    Debug, Clone, anchor_lang::prelude::AnchorSerialize, anchor_lang::prelude::AnchorDeserialize,
)]
pub struct UserAccount {
    pub balance: u64,
}

// fn main() -> Result<()> {
fn main() -> Result<(), Box<dyn Error>> {
    // 🔐 Загрузка ключей пользователя
    let payer = read_keypair_file("~/.config/solana/id.json")?;
    let client = Client::new_with_options(
        Cluster::Devnet,
        Rc::new(payer),
        CommitmentConfig::processed(),
    );
    let program = client.program(id())?;

    // 📦 Генерация адреса аккаунта пользователя (PDA)
    let user = program.payer();
    let (user_account_pda, _bump) =
        anchor_lang::solana_program::pubkey::Pubkey::find_program_address(
            &[user.to_bytes().as_ref()],
            &program.id(),
        );

    println!("User account PDA: {}", user_account_pda);

    // ✅ Инициализация аккаунта
    let _sig = program
        .request()
        .accounts(deposit_program::accounts::Initialize {
            user_account: user_account_pda,
            user,
            system_program: system_program::ID,
        })
        .args(deposit_program::instruction::Initialize {})
        .send()?;

    println!("User account initialized");

    // 💰 Внести депозит
    let deposit_amount = 1_000_000; // 0.001 SOL
    let _sig = program
        .request()
        .accounts(deposit_program::accounts::Deposit {
            user_account: user_account_pda,
        })
        .args(deposit_program::instruction::Deposit {
            amount: deposit_amount,
        })
        .send()?;

    println!("Deposited {} lamports", deposit_amount);

    // 💸 Вывод средств
    let withdraw_amount = 500_000; // 0.0005 SOL
    let _sig = program
        .request()
        .accounts(deposit_program::accounts::Withdraw {
            user_account: user_account_pda,
        })
        .args(deposit_program::instruction::Withdraw {
            amount: withdraw_amount,
        })
        .send()?;

    println!("Withdrew {} lamports", withdraw_amount);

    Ok(())
}
