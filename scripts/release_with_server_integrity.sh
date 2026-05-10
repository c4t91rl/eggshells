#!/bin/bash
set -e

echo "═══════════════════════════════════════════════"
echo "  Release: build + register hash on server"
echo "═══════════════════════════════════════════════"
echo ""

# 1. Build release
echo "1. Building release binary..."
cargo build --release -p secure-update-client

BINARY="target/release/secure-update-client"
if [ ! -f "$BINARY" ]; then
    echo "❌ Build failed"
    exit 1
fi

# 2. Hash binarki
echo ""
echo "2. Computing SHA3-256 of binary..."
HASH=$(python3 -c "
import hashlib
with open('$BINARY', 'rb') as f:
    print(hashlib.sha3_256(f.read()).hexdigest())
")
echo "   Hash: $HASH"

# 3. Wykryj platformę
PLATFORM_KEY="linux-x86_64"
case "$(uname -s)" in
    Linux*)
        case "$(uname -m)" in
            x86_64) PLATFORM_KEY="linux-x86_64" ;;
            aarch64) PLATFORM_KEY="linux-aarch64" ;;
        esac
        ;;
    MINGW*|MSYS*|CYGWIN*)
        PLATFORM_KEY="windows-x86_64"
        ;;
esac

echo "   Platform: $PLATFORM_KEY"

# 4. Wczytaj/utwórz plik z hashami
HASHES_FILE="./server_data/client_hashes.json"
mkdir -p ./server_data

if [ ! -f "$HASHES_FILE" ]; then
    echo "{}" > "$HASHES_FILE"
fi

# 5. Zaktualizuj plik
echo ""
echo "3. Updating $HASHES_FILE..."
NOW=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

python3 << PYEOF
import json

with open("$HASHES_FILE", "r") as f:
    data = json.load(f)

data["$PLATFORM_KEY"] = {
    "sha3_256": "$HASH",
    "version": "0.1.0",
    "released_at": "$NOW"
}

with open("$HASHES_FILE", "w") as f:
    json.dump(data, f, indent=2)

print("   File updated:")
print(json.dumps(data, indent=2))
PYEOF

echo ""
echo "✅ Done!"
echo ""
echo "Next steps:"
echo "  1. Restart server (Ctrl+C, then cargo run --release -p secure-update-server)"
echo "  2. Run client: target/release/secure-update-client"
echo "  3. Integrity check should pass ✅"