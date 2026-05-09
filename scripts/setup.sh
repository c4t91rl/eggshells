#!/bin/bash
set -euo pipefail

echo "🔧 Setting up KryptoUpdate development environment..."

# Check prerequisites
command -v cargo >/dev/null 2>&1 || { echo "❌ Rust not found. Install from https://rustup.rs"; exit 1; }
command -v node >/dev/null 2>&1 || { echo "❌ Node.js not found. Install from https://nodejs.org"; exit 1; }
command -v npm >/dev/null 2>&1 || { echo "❌ npm not found"; exit 1; }

echo "✓ Prerequisites checked"

# Install Rust tools
echo "📦 Installing Rust tools..."
cargo install cargo-audit cargo-deny tauri-cli || true

# Setup UI
echo "📦 Installing UI dependencies..."
cd ../crates/update-client/ui
npm install vite@^7.0.0 --save-dev
npm install
#npm audit fix --force
cd ../../..

# Create data directories
mkdir -p data/packages data/manifests data/downloads keys reports

# Generate default publisher keys
echo "🔑 Generating default publisher keys..."
cargo run --bin keygen -- generate \
    --id "demo-publisher" \
    --name "Demo Publisher" \
    --url "http://localhost:8443" \
    --algorithm hybrid \
    --output keys

echo ""
echo "✅ Setup complete!"
echo ""
echo "To start the update server:"
echo "  cargo run --bin update-server"
echo ""
echo "To start the client (dev mode):"
echo "  cd ../crates/update-client && cargo tauri dev && cd ../../scripts"
echo ""
echo "To run security analysis:"
echo "  bash ./run-sast.sh"
echo ""
echo "To run attack simulator:"
echo "  cargo run --bin attack-simulator -- tamper-manifest -s http://localhost:8443 -p demo-app"