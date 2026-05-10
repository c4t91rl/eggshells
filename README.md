# eggshell
### Kościuszkon 2026 – Honeywell Theme #1

Bezpieczny system aktualizacji oprogramowania z kryptografią postkwantową.

## Szybki przegląd

| Komponent | Technologia | Opis |
|---|---|---|
| Update Server | Rust + Actix-web | REST API, SQLite, multi-publisher |
| Update Client | Rust + egui | Cross-platform GUI, weryfikacja podpisów |
| Publisher Tool | Rust CLI | Generowanie kluczy, podpisywanie, publikacja |
| Kryptografia | Dilithium3 + Ed25519 + SHA3-256 | Hybrydowe podpisy postkwantowe |

## Uruchomienie

```bash
# 1. Build
cargo build --release

# 2. Serwer (Terminal 1)
cargo run --release -p secure-update-server

# 3. Generuj klucze publishera (Terminal 2)
cargo run --release -p secure-update-publisher -- generate-keys \
    --publisher-id "my-publisher" --output ./keys/

cargo run --release -p secure-update-publisher-gui

# 4. Klient GUI (Terminal 3)
cargo run --release -p secure-update-client

# 5. Testy
cargo test --all

# 6. Analiza bezpieczeństwa
bash scripts/run_sast.sh