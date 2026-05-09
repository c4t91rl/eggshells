Write-Host "🔧 Setting up KryptoUpdate development environment..." -ForegroundColor Cyan

# Check prerequisites
if (!(Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Rust not found. Install from https://rustup.rs" -ForegroundColor Red
    exit 1
}
if (!(Get-Command node -ErrorAction SilentlyContinue)) {
    Write-Host "❌ Node.js not found. Install from https://nodejs.org" -ForegroundColor Red
    exit 1
}

Write-Host "✓ Prerequisites checked" -ForegroundColor Green

# Install Rust tools
Write-Host "📦 Installing Rust tools..."
cargo install cargo-audit tauri-cli 2>$null

# Setup UI
Write-Host "📦 Installing UI dependencies..."
Push-Location crates/update-client/ui
npm install
Pop-Location

# Create directories
New-Item -ItemType Directory -Force -Path data/packages, data/manifests, data/downloads, keys, reports | Out-Null

# Generate keys
Write-Host "🔑 Generating default publisher keys..."
cargo run --bin keygen -- generate `
    --id "demo-publisher" `
    --name "Demo Publisher" `
    --url "http://localhost:8443" `
    --algorithm hybrid `
    --output keys

Write-Host ""
Write-Host "✅ Setup complete!" -ForegroundColor Green
Write-Host ""
Write-Host "To start the update server:"
Write-Host "  cargo run --bin update-server" -ForegroundColor Yellow
Write-Host ""
Write-Host "To start the client (dev mode):"
Write-Host "  cd crates/update-client; cargo tauri dev" -ForegroundColor Yellow