#!/bin/bash
set -euo pipefail

echo "════════════════════════════════════════════"
echo "  KryptoUpdate Security Analysis Pipeline  "
echo "════════════════════════════════════════════"
echo ""

# 1. SAST - Static Analysis with Clippy
echo "📋 [1/4] Running SAST (cargo clippy)..."
cargo clippy --workspace --all-targets -- \
    -W clippy::all \
    -W clippy::pedantic \
    -W clippy::nursery \
    -D clippy::unwrap_used \
    2>&1 | tee reports/sast-clippy.txt
echo "   ✓ SAST complete"

# 2. SCA - Dependency Audit
echo ""
echo "📦 [2/4] Running SCA (cargo audit)..."
cargo audit 2>&1 | tee reports/sca-audit.txt
echo "   ✓ SCA complete"

# 3. Unsafe code detection
echo ""
echo "🔒 [3/4] Checking for unsafe code..."
grep -rn "unsafe" crates/ --include="*.rs" | grep -v "// SAFETY:" | tee reports/unsafe-check.txt || echo "   ✓ No unsafe code found"

# 4. Secret scanning
echo ""
echo "🔑 [4/4] Scanning for potential secrets..."
grep -rn "password\|secret\|api_key\|private_key" crates/ --include="*.rs" | grep -v "test\|example\|comment\|doc" | tee reports/secrets-scan.txt || echo "   ✓ No secrets found"

echo ""
echo "════════════════════════════════════════════"
echo "  Analysis complete! Reports in reports/   "
echo "════════════════════════════════════════════"