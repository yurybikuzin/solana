# Chat GPT

## Request 

Add a Rust client example.

## Solution

Вот пример Rust-клиента, который взаимодействует с программой Solana, описанной выше (через Anchor framework). Этот клиент:

1. Инициализирует аккаунт пользователя.

2. Вносит депозит.

3. Выводит средства.

## Подготовка к запуску

Установи Anchor CLI: cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked

Убедись, что Solana CLI установлен и настроен (например, на Devnet):

bash
Копировать
Редактировать
solana config set --url https://api.devnet.solana.com
Запусти Anchor-деплой и скопируй ID программы в declare_id! клиента.

Запусти клиент:

bash
Копировать
Редактировать
cargo run
