#!/bin/bash
# scripts/run_sast.sh
# Skrypt analizy statycznej kodu (SAST)

echo "═══════════════════════════════════════"
echo "  SAST Analysis - Secure Update System"
echo "═══════════════════════════════════════"

echo ""
echo "1. Running cargo clippy (linter)..."
echo "─────────────────────────────────────"
cargo clippy --all-targets --all-features -- -W clippy::all -W clippy::pedantic 2>&1 | head -100

echo ""
echo "2. Running cargo audit (SCA - dependency vulnerabilities)..."
echo "────────────────────────────────────────────────────────────"
if command -v cargo-audit &> /dev/null; then
    cargo audit
else
    echo "cargo-audit not installed. Install with: cargo install cargo-audit"
fi

echo ""
echo "3. Running cargo deny (license & advisory check)..."
echo "───────────────────────────────────────────────────"
if command -v cargo-deny &> /dev/null; then
    cargo deny check
else
    echo "cargo-deny not installed. Install with: cargo install cargo-deny"
fi

echo ""
echo "4. Running cargo geiger (unsafe code detection)..."
echo "──────────────────────────────────────────────────"
if command -v cargo-geiger &> /dev/null; then
    cargo geiger --all-features --all-targets
else
    echo "cargo-geiger not installed. Install with: cargo install cargo-geiger"
fi

echo ""
echo "5. Checking for common security issues..."
echo "─────────────────────────────────────────"
echo "   Searching for unwrap() calls (potential panics)..."
grep -rn "\.unwrap()" crates/ --include="*.rs" | grep -v "test" | grep -v "#\[cfg(test)\]" | head -20
echo ""
echo "   Searching for unsafe blocks..."
grep -rn "unsafe" crates/ --include="*.rs" | head -20

echo ""
echo "═══════════════════════════════════════"
echo "  SAST Analysis Complete"
echo "═══════════════════════════════════════"