# Architektura Systemu

## Przegląd

System składa się z trzech głównych komponentów:
┌──────────────────────────────────────────────────────────────────────────────┐
│                             ARCHITEKTURA SYSTEMU                             │
├──────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌────────────────────┐       HTTPS (TLS)        ┌────────────────────────┐  │
│  │   UPDATE CLIENT    │<────────────────────────>│     UPDATE SERVER      │  │
│  │    (Rust + egui)   │                          │    (Rust/Actix-web)    │  │
│  ├────────────────────┤    1. Check version      ├────────────────────────┤  │
│  │ ┌────────────────┐ │    2. Get metadata       │ ┌────────────────────┐ │  │
│  │ │   PQ Verify    │ │    3. Download pkg       │ │   PUBLISHERS DATA  │ │  │
│  │ └────────────────┘ │    4. Verify signature   │ │ (Keys + Packages)  │ │  │
│  │ ┌────────────────┐ │                          │ ├────────────────────┤ │  │
│  │ │ Version Check  │ │                          │ │ - Publisher A      │ │  │
│  │ └────────────────┘ │                          │ │ - Publisher B      │ │  │
│  │ ┌────────────────┐ │                          │ │ - Publisher C      │ │  │
│  │ │  Anti-Tamper   │ │                          │ └────────────────────┘ │  │
│  │ └────────────────┘ │                          │ ┌────────────────────┐ │  │
│  │                    │                          │ │    METADATA DB     │ │  │
│  └────────────────────┘                          │ │      (SQLite)      │ │  │
│            ▲                                     │ └────────────────────┘ │  │
│            │                                     └────────────────────────┘  │
│            │                                                 ▲               │
│            │                                                 │               │
│  ┌─────────┴──────────┐                                      │               │
│  │   PUBLISHER TOOL   │          Upload (Auth)               │               │
│  │       (CLI)        ├──────────────────────────────────────┘               │
│  ├────────────────────┤                                                      │
│  │  1. Key Generation │                                                      │
│  │  2. Sign Package   │                                                      │
│  │  3. Upload Assets  │                                                      │
│  └────────────────────┘                                                      │
│                                                                              │
└──────────────────────────────────────────────────────────────────────────────┘

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
     CLIENT                                           SERVER
        │                                                │
        │─── POST /api/check/{app_id} ──────────────────►│
        │    Body: {current_version, platform}           │
        │                                                │
        │◄── 200 OK ─────────────────────────────────────│
        │    Body: {update_available, metadata,          │
        │           publisher_public_key}                │
        │                                                │
        ▼                                                │
  ┌──────────────────────────┐                           │
  │ Logika Klienta:          │                           │
  │ 1. Nowa > obecna?        │                           │
  │ 2. Sprawdź Key Pinning   │                           │
  └──────────────────────────┘                           │
        │                                                │
        │─── GET /api/download/{app}/{ver} ─────────────►│
        │                                                │
        │◄── [binary data] ──────────────────────────────│
        │                                                │
        ▼                                                │
  ┌─────────────────────────────────┐                    │
  │ Weryfikacja Integralności:      │                    │
  │ 1. Oblicz SHA3-256 pliku        │                    │
  │ 2. Porównaj z hash z metadata   │                    │
  └─────────────────────────────────┘                    │
        │                                                │
        ▼                                                │
  ┌─────────────────────────────────┐                    │
  │ Weryfikacja Kryptograficzna:    │                    │
  │ 1. Verify Dilithium3 signature  │                    │
  │ 2. Verify Ed25519 signature     │                    │
  └─────────────────────────────────┘                    │
        │                                                │
        ▼                                                │
  ┌─────────────────────────────────┐                    │
  │ Decyzja końcowa:                │                    │
  │ Oba OK? → Zastosuj aktualizację │                    │
  └─────────────────────────────────┘                    │
        │                                                │

## Model wieloserwera / wielu publisherów
┌─────────────────────────────────────────────────────────────────┐
│                      MULTI-PUBLISHER MODEL                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  PUBLISHER A                           PUBLISHER B              │
│  ┌──────────┐                          ┌──────────┐             │
│  │  Keygen  │                          │  Keygen  │             │
│  │ Dil + Ed │                          │ Dil + Ed │             │
│  └────┬─────┘                          └────┬─────┘             │
│       │ 1. Register Public Key              │                   │
│       └──────────────────┐    ┌─────────────┘                   │
│                          ▼    ▼                                 │
│                ┌──────────────────────────┐                     │
│                │      UPDATE SERVER       │                     │
│                │                          │                     │
│                │  [Trust Store / DB]      │                     │
│                │  ID A: Pub_Key_A         │                     │
│                │  ID B: Pub_Key_B         │                     │
│                └─────┬──────────────┬─────┘                     │
│                      ▲              ▲                           │
│       2. Upload      │              │      2. Upload            │
│       Signed Pkg     │              │      Signed Pkg           │
│    ┌─────────────┐   │              │   ┌─────────────┐         │
│    │  Sign with  ├───┘              └───┤  Sign with  │         │
│    │  Priv_Key_A │                      │  Priv_Key_B │         │
│    └─────────────┘                      └─────────────┘         │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                          DISTRIBUTION                           │
└───────────────┬─────────────────────────────────┬───────────────┘
                │                                 │
                │ 3. Download Pkg + Signature     │
                ▼                                 ▼
        ┌─────────────────────────────────────────────────┐
        │                 CLIENT DEVICE                   │
        │                                                 │
        │  1. Fetch Trusted Pub_Keys from Server/Root     │
        │  2. Identify Publisher of Pkg                   │
        │  3. Verify Sig(Pkg) using corresponding Pub_Key │
        └─────────────────────────────────────────────────┘

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
