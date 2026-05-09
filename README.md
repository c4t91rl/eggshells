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

# 4. Rejestracja publishera
cargo run --release -p secure-update-publisher -- register \
    --keys ./keys/my-publisher.keys.json \
    --server http://127.0.0.1:8443 \
    --name "My Publisher"

# 5. Publikacja pakietu
echo "App v1.1.0 content" > /tmp/update.bin
cargo run --release -p secure-update-publisher -- publish \
    --keys ./keys/my-publisher.keys.json \
    --server http://127.0.0.1:8443 \
    --app-id "example-app" \
    --version "1.1.0" \
    --file /tmp/update.bin \
    --description "Security update"

# 6. Klient GUI (Terminal 3)
cargo run --release -p secure-update-client

# 7. Testy
cargo test --all

# 8. Analiza bezpieczeństwa
bash scripts/run_sast.sh