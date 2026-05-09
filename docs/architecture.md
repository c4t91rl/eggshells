# Architektura Systemu

## Przegląd

System składa się z trzech głównych komponentów:
┌─────────────────────────────────────────────────────────────────┐
│ ARCHITEKTURA SYSTEMU │
├─────────────────────────────────────────────────────────────────┤
│ │
│ ┌──────────────┐ HTTP/TLS ┌──────────────────────┐ │
│ │ │◄──────────────────►│ │ │
│ │ UPDATE │ │ UPDATE SERVER │ │
│ │ CLIENT │ 1. Check version │ (Rust/Actix-web) │ │
│ │ (Rust+egui) │ 2. Get metadata │ │ │
│ │ │ 3. Download pkg │ ┌────────────────┐ │ │
│ │ ┌────────┐ │ 4. Verify sig │ │ Publisher A │ │ │
│ │ │PQ Verify│ │ │ │ (keys + pkgs) │ │ │
│ │ └────────┘ │ │ ├────────────────┤ │ │
│ │ ┌────────┐ │ │ │ Publisher B │ │ │
│ │ │Version │ │ │ │ (keys + pkgs) │ │ │
│ │ │Check │ │ │ ├────────────────┤ │ │
│ │ └────────┘ │ │ │ Publisher C │ │ │
│ │ ┌────────┐ │ │ │ (keys + pkgs) │ │ │
│ │ │Anti- │ │ │ └────────────────┘ │ │
│ │ │Tamper │ │ │ │ │
│ │ └────────┘ │ │ ┌────────────────┐ │ │
│ │ │ │ │ Metadata DB │ │ │
│ └──────────────┘ │ │ (SQLite) │ │ │
│ │ └────────────────┘ │ │
│ ┌──────────────┐ └──────────────────────┘ │
│ │ PUBLISHER │ │
│ │ TOOL (CLI) │ │
│ │ │ │
│ │ 1. Keygen │ │
│ │ 2. Sign │──────────────────────────────────────► │
│ │ 3. Upload │ │
│ └──────────────┘ │
└─────────────────────────────────────────────────────────────────┘

text


## Komponenty

### Update Server
- Framework: Rust + Actix-web 4
- Baza danych: SQLite (via rusqlite)
- Storage: lokalny filesystem
- Port: 8443 (HTTP w prototypie, HTTPS w produkcji)

**Endpointy REST API:**

| Metoda | Endpoint | Opis |
|--------|----------|------|
| GET | `/api/health` | Health check |
| POST | `/api/publishers` | Rejestracja publishera |
| GET | `/api/publishers` | Lista publisherów |
| POST | `/api/packages/metadata` | Publikacja metadanych |
| POST | `/api/packages/upload/{pub}/{app}/{ver}` | Upload pliku |
| POST | `/api/check/{app_id}` | Sprawdzenie aktualizacji |
| GET | `/api/download/{app_id}/{version}` | Pobranie pakietu |

### Update Client
- Framework: Rust + egui/eframe 0.28
- Cross-platform: Windows x86_64, Linux x86_64/aarch64
- Konfiguracja: JSON (lokalny plik)
- Key pinning: TOFU (Trust On First Use)

**Zakładki GUI:**
- **Dashboard** – status aplikacji, quick actions
- **Update** – sprawdzanie, pobieranie, weryfikacja, instalacja
- **Security** – hardening report, pinned keys, threat model
- **Settings** – konfiguracja serwera, wersja, key management
- **Logs** – historia operacji

### Publisher Tool
- Interfejs: CLI (clap 4)
- Komendy: `generate-keys`, `register`, `publish`, `sign`, `verify`
- Klucze prywatne: JSON z uprawnieniami 600 (Linux)

## Przepływ aktualizacji
Client Server
│ │
│──POST /api/check/{app_id}─────────►│
│ {current_version, platform} │
│◄──{update_available, metadata,─────│
│ publisher_public_key} │
│ │
│ [Sprawdź: nowa > obecna?] │
│ [Sprawdź: key pinning] │
│ │
│──GET /api/download/{app}/{ver}────►│
│◄──[binary data]────────────────────│
│ │
│ [Oblicz SHA3-256] │
│ [Porównaj z hash z metadata] │
│ [Weryfikuj podpis Dilithium3] │
│ [Weryfikuj podpis Ed25519] │
│ [Oba OK? → Zastosuj aktualizację] │
│ │

text


## Model wieloserwera / wielu publisherów
┌─────────────────────────────────────────┐
│ MULTI-PUBLISHER MODEL │
├─────────────────────────────────────────┤
│ │
│ Publisher A Publisher B │
│ ┌──────────┐ ┌──────────┐ │
│ │ keygen │ │ keygen │ │
│ │ Dil + Ed │ │ Dil + Ed │ │
│ └────┬─────┘ └────┬─────┘ │
│ │ register │ register │
│ ▼ ▼ │
│ ┌─────────────────────────────────┐ │
│ │ UPDATE SERVER │ │
│ │ publishers table: │ │
│ │ ├── A: pub_key_A │ │
│ │ └── B: pub_key_B │ │
│ └─────────────────────────────────┘ │
│ ▲ ▲ │
│ │ upload signed pkg │ upload │
│ ┌────┴─────┐ ┌────┴─────┐ │
│ │ sign pkg │ │ sign pkg │ │
│ │ with A │ │ with B │ │
│ └──────────┘ └──────────┘ │
│ │
│ Klient weryfikuje każdy pakiet │
│ kluczem odpowiedniego publishera │
└─────────────────────────────────────────┘

text


## Struktura danych SQLite

```sql
CREATE TABLE publishers (
    id TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    public_key_json TEXT NOT NULL,   -- HybridPublicKey (Dilithium + Ed25519)
    registered_at TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE packages (
    package_id TEXT PRIMARY KEY,
    app_id TEXT NOT NULL,
    version_major INTEGER NOT NULL,
    version_minor INTEGER NOT NULL,
    version_patch INTEGER NOT NULL,
    publisher_id TEXT NOT NULL,
    sha3_256_hash TEXT NOT NULL,
    file_size INTEGER NOT NULL,
    filename TEXT NOT NULL,
    description TEXT NOT NULL,
    target_platforms_json TEXT NOT NULL,
    signature_json TEXT NOT NULL,    -- HybridSignature (Dilithium + Ed25519)
    published_at TEXT NOT NULL,
    min_upgrade_from_json TEXT,
    changelog_json TEXT NOT NULL,
    FOREIGN KEY (publisher_id) REFERENCES publishers(id)
);
Wybory technologiczne
Decyzja	Alternatywy	Uzasadnienie
Rust	Go, C++, Python	Memory safety, zero unsafe w naszym kodzie, cross-platform
egui	Qt, GTK, Tauri	Pure Rust, brak external deps, cross-platform out-of-the-box
SQLite	PostgreSQL, flat files	Prostota prototypu, brak serwera DB
Actix-web	Axum, Warp	Dojrzały, wysoka wydajność, duże community
Dilithium3	Dilithium2/5, SPHINCS+	Balans: bezpieczeństwo NIST Level 3 vs rozmiar
Ed25519	RSA, ECDSA P-256	Szybki, małe klucze, dobrze zbadany
SHA3-256	SHA-256, BLAKE3	Odporny na length-extension, solidna podstawa NIST