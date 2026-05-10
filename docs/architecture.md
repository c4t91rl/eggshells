# Architektura Systemu

Projekt jest podzielony na trzy główne moduły:
- `secure-update-server` — serwer aktualizacji z REST API,
- `secure-update-client` — klient GUI do pobierania i weryfikowania aktualizacji,
- `secure-update-publisher` — narzędzie CLI do generowania kluczy, rejestracji publisherów i publikacji pakietów.

Całość oparta jest na Rust i wyraźnie oddziela czynności publikacji, przechowywania metadanych oraz weryfikacji aktualizacji.

## Komponenty

### Update Server
- Rust + Actix-web;
- REST API exposes endpoints under `/api`;
- SQLite (`rusqlite`) dla bazy publisherów i pakietów;
- lokalny filesystem dla przechowywania plików pakietów;
- prototypowo nasłuchuje na `127.0.0.1:8443`.

#### Endpointy
- `GET /api/health` — status serwera,
- `POST /api/publishers` — rejestracja nowego publishera,
- `GET /api/publishers` — lista zarejestrowanych publisherów,
- `POST /api/packages/metadata` — publikacja metadanych nowego pakietu,
- `POST /api/packages/upload/{publisher_id}/{app_id}/{version}` — upload binarnego pakietu,
- `POST /api/check/{app_id}` — sprawdzenie dostępności aktualizacji,
- `GET /api/download/{app_id}/{version}` — pobranie wersji pakietu.

#### Rola
- przyjmowanie metadanych i publikacja pakietów,
- przechowywanie kluczy publicznych publisherów,
- pozwalanie klientowi na pobranie informacji o najnowszej wersji,
- serwowanie binarnych plików z lokalnego katalogu.

### Update Client
- Rust + eframe/egui;
- GUI do konfiguracji serwera, sprawdzania i instalacji aktualizacji;
- lokalny config JSON z listą serwerów, obecnymi wersjami, katalogami pobrań i pinned keys;
- korzysta z `reqwest` do połączeń HTTP/HTTPS z serwerem.

#### Funkcje
- sprawdza aktualizacje dla wybranego `app_id`,
- pobiera metadane i publiczny klucz publishera,
- pobiera plik aktualizacji,
- weryfikuje hash SHA3-256 i podpisy Dilithium3 + Ed25519,
- wykonuje instalację tylko po pozytywnej weryfikacji.

### Publisher Tool
- CLI z `clap` i flagami takimi jak `register`, `publish`, `generate-keys`;
- generuje pary kluczy hybrydowych (`HybridKeyPair`),
- rejestruje publiczny klucz na serwerze,
- publikuje metadane pakietu oraz przesyła binarny plik.

#### Przepływ publikacji
1. `generate-keys` tworzy klucze Dilithium3 + Ed25519,
2. `register` wysyła publiczny klucz do `/api/publishers`,
3. `publish` wysyła metadane z SHA3-256 i hybrydowym podpisem do `/api/packages/metadata`,
4. `upload` przesyła binarny pakiet do `/api/packages/upload/{publisher}/{app}/{version}`.

## Przepływ aktualizacji

1. Klient robi `POST /api/check/{app_id}` z aktualną wersją i platformą.
2. Serwer zwraca informację, czy jest dostępna nowsza wersja oraz metadane pakietu.
3. Jeżeli jest aktualizacja, klient pobiera plik z `GET /api/download/{app_id}/{version}`.
4. Klient najpierw porównuje hash SHA3-256 z metadanych.
5. Klient weryfikuje hybrydowy podpis pakietu w oparciu o publiczny klucz publishera.
6. Jeśli mamy zgodność, klient instaluje aktualizację.

## Oddzielne ścieżki metadanych i pliku

Serwer oddziela:
- publikację metadanych (`/api/packages/metadata`),
- upload binarny (`/api/packages/upload/...`).

Dzięki temu serwer może wymagać, aby najpierw pojawiły się metadane z podpisem, a dopiero potem binarny plik.

## Struktura danych

### `publishers`
- `id` — identyfikator publishera,
- `display_name` — nazwa,
- `public_key_json` — klucz hybrydowy (`HybridPublicKey`),
- `registered_at` — timestamp,
- `active` — aktywność konta.

### `packages`
- `package_id`,
- `app_id`,
- `version_major`, `version_minor`, `version_patch`,
- `publisher_id`,
- `sha3_256_hash`,
- `file_size`,
- `filename`,
- `description`,
- `target_platforms_json`,
- `signature_json`,
- `published_at`,
- `min_upgrade_from_json`,
- `changelog_json`.

## Decyzje technologiczne

- `Rust` dla bezpieczeństwa pamięci i spójności między komponentami,
- `Actix-web` dla niskiego narzutu HTTP i łatwej integracji z Tokio,
- `SQLite` dla prostego, lokalnego repozytorium prototypu,
- `eframe/egui` jako lekki, natywny GUI w Rust,
- `SHA3-256` dla integralności danych i odporności na ataki typu length-extension,
- `CRYSTALS-Dilithium3` dla odporności postkwantowej,
- `Ed25519` jako drugi podpis dla praktycznej i szybkiej weryfikacji.

## Właściwości systemu

- separacja odpowiedzialności: publikacja, weryfikacja, przechowywanie,
- hybrydowy model podpisu: dwuwarstwowa weryfikacja bezpieczeństwa,
- prototyp bazujący na HTTP, ale z wyraźnymi punktami produkcyjnej migracji do TLS,
- możliwość wielu publisherów i wielu aplikacji na jednym serwerze.
