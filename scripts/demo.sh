#!/bin/bash
# scripts/demo.sh
# Pełna demonstracja systemu

set -e

# Kill stale demo server/client processes and clean previous demo state.
echo "Cleaning demo state..."
pkill -f "secure-update-server" >/dev/null 2>&1 || true
pkill -f "secure-update-client" >/dev/null 2>&1 || true
sleep 1
if lsof -iTCP:8443 -sTCP:LISTEN -t >/dev/null 2>&1; then
    echo "Port 8443 is still in use after killing stale processes."
    echo "Please stop the running process or choose a different port."
    exit 1
fi
rm -rf ./server_data/db ./server_data/packages ./demo-keys /tmp/demo-app-v1.1.0.bin

echo "═══════════════════════════════════════════════════"
echo  "Secure Update System - Full Demo"
echo "═══════════════════════════════════════════════════"
echo ""

# 1. Budowanie
echo "1. Building all components..."
cargo build --release
echo   " Build complete"
echo ""

# 2. Uruchom serwer w tle
echo "2. Starting update server..."
cargo run --release -p secure-update-server &
SERVER_PID=$!
sleep 2
echo   " Server started (PID: $SERVER_PID)"
echo ""

# 3. Generowanie kluczy publishera
echo "3. Generating publisher keys..."
cargo run --release -p secure-update-publisher -- generate-keys \
    --publisher-id "demo-publisher" \
    --output ./demo-keys/
echo ""

# 4. Tworzenie konta i logowanie
USERNAME="demo-user"
PASSWORD="demo-password"

echo "4. Creating publisher account..."
cargo run --release -p secure-update-publisher -- create-account \
    --username "$USERNAME" \
    --password "$PASSWORD" \
    --publisher-id "demo-publisher" \
    --display-name "Demo Publisher Inc." \
    --server https://127.0.0.1:8443
echo ""

echo "5. Logging in..."
TOKEN=$(cargo run --release -p secure-update-publisher -- login \
    --username "$USERNAME" \
    --password "$PASSWORD" \
    --server https://127.0.0.1:8443 | awk '/token:/ {print $2}')

if [ -z "$TOKEN" ]; then
    echo "Failed to obtain auth token"
    kill $SERVER_PID 2>/dev/null || true
    exit 1
fi

echo " Token: $TOKEN"
echo ""

# 6. Rejestracja publishera
echo "6. Registering publisher..."
cargo run --release -p secure-update-publisher -- register \
    --keys ./demo-keys/demo-publisher.keys.json \
    --server https://127.0.0.1:8443 \
    --name "Demo Publisher Inc." \
    --token "$TOKEN"
echo ""

# 7. Tworzenie testowego pakietu
echo "7. Creating test package..."
echo "This is demo application v1.1.0 with important security fixes." > /tmp/demo-app-v1.1.0.bin
echo ""

# 8. Podpisywanie i publikacja
echo "8. Signing and publishing package..."
cargo run --release -p secure-update-publisher -- publish \
    --keys ./demo-keys/demo-publisher.keys.json \
    --server https://127.0.0.1:8443 \
    --token "$TOKEN" \
    --app-id "example-app" \
    --version "1.1.0" \
    --file /tmp/demo-app-v1.1.0.bin \
    --description "Security update with critical fixes" \
    --changelog "Fixed CVE-2024-0001" \
    --changelog "Improved crypto implementation" \
    --changelog "Performance optimizations"
echo ""

# 7. Uruchom klienta GUI
echo "7. Starting update client GUI..."
echo  " (The client will check for updates from the server)"
cargo run --release -p secure-update-client &
CLIENT_PID=$!

echo ""
echo "═══════════════════════════════════════════════════"
echo  "Demo running!"
echo  "Server PID: $SERVER_PID"
echo  "Client PID: $CLIENT_PID"
echo ""
echo  "Press Enter to stop demo..."
echo "═══════════════════════════════════════════════════"

read
kill $SERVER_PID 2>/dev/null || true
kill $CLIENT_PID 2>/dev/null || true
rm -rf ./demo-keys ./server_data/db ./server_data/packages /tmp/demo-app-v1.1.0.bin
echo "Demo stopped."
