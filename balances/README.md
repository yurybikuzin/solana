# Chat GPT

Используя Solana JSON RPC API, реализовать на языке программирования Rust функцию для одновременного получения текущего баланса указанных кошельков

Кошельки задается через config.yaml

# Советы

Можешь улучшить get_balance, чтобы обрабатывать возможные ошибки JSON (result может отсутствовать).

Для больших списков кошельков стоит добавить ограничение на параллелизм (tokio::sync::Semaphore или futures::stream::FuturesUnordered).

Используй кэш или rate limiting, если работаешь с публичным RPC.
