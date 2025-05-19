# Chat GPT

## Main.rs

## Request

Реализовать CLI-программу на языке Rust, которая позволяет:

Отправить  SOL с массива кошельков одновременно на другой массив кошельков (ключи и адреса предоставляются в файле config.yaml).

Проверить статусы всех транзакций после выполнения. Вывести хеши транзакций и статистику времени обработки.

### Solution

CLI-программа на Rust, которая:
- Загружает config.yaml с массивом ключей и адресов.
- Отправляет SOL параллельно от отправителей к получателям.
- Проверяет статусы транзакций.
- Выводит хэши транзакций и статистику по времени выполнения.

### Дополнительно

Вы можете добавить опцию для выбора кластера (devnet, testnet).

Возможна отправка всех с одного кошелька на множество адресов (если нужно — уточните).

Можно добавить логирование, retries и хранение хэшей транзакций в файл.

## load_keypair_from_file

## Request

write Rust function which loads solana keypairs from json-file

### Solution

Simple and idiomatic Rust function to load Solana keypairs from a JSON file using the solana-sdk crate

### Notes

The JSON file is expected to contain an array of 64 bytes representing the private key (like id.json created by solana-keygen).

