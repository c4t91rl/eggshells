#!/bin/bash
# scripts/demo.sh
# Pełna demonstracja systemu

set -e

echo "═══════════════════════════════════════════════════"
echo "  Secure Update System - Full Demo"
echo "═══════════════════════════════════════════════════"
echo ""

# 1. Budowanie
echo "1. Building all components..."
cargo build --release
echo "     Build complete"
echo ""

# 2. Uruchom serwer w tle
echo "2. Starting update server..."
cargo run --release -p secure-update-server &
SERVER_PID=$!
sleep 2
echo "     Server started (PID: $SERVER_PID)"
echo ""

# 3. Generowanie kluczy publishera
echo "3. Generating publisher keys..."
cargo run --release -p secure-update-publisher -- generate-keys \
    --publisher-id "demo-publisher" \
    --output ./demo-keys/
echo ""

# 4. Rejestracja publishera
echo "4. Registering publisher..."
cargo run --release -p secure-update-publisher -- register \
    --keys ./demo-keys/demo-publisher.keys.json \
    --server http://127.0.0.1:8443 \
    --name "Demo Publisher Inc."
echo ""

# 5. Tworzenie testowego pakietu
echo "5. Creating test package..."
echo "This is demo application v1.1.0 with important security fixes." > /tmp/demo-app-v1.1.0.bin
echo ""

# 6. Podpisywanie i publikacja
echo "6. Signing and publishing package..."
cargo run --release -p secure-update-publisher -- publish \
    --keys ./demo-keys/demo-publisher.keys.json \
    --server http://127.0.0.1:8443 \
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
echo "   (The client will check for updates from the server)"
cargo run --release -p secure-update-client &
CLIENT_PID=$!

echo ""
echo "═══════════════════════════════════════════════════"
echo "  Demo running!"
echo "  Server PID: $SERVER_PID"
echo "  Client PID: $CLIENT_PID"
echo ""
echo "  Press Enter to stop demo..."
echo "═══════════════════════════════════════════════════"

read
kill $SERVER_PID 2>/dev/null || true
kill $CLIENT_PID 2>/dev/null || true
rm -rf ./demo-keys /tmp/demo-app-v1.1.0.bin
echo "Demo stopped."
