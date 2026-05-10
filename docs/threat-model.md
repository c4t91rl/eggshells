# Model Zagrożeń

## Zakres i aktywa

Ten dokument opisuje zagrożenia związane z systemem aktualizacji.
Zakres obejmuje:
- publikowanie pakietów i metadanych na serwerze,
- wymianę danych między klientem a serwerem,
- weryfikację podpisów i hashy po stronie klienta.

### Aktywa

| Aktywum | Opis | Waga |
|--------|------|------|
| Klucze prywatne publisherów | Klucze Dilithium3 + Ed25519 używane do podpisu pakietów | Krytyczna |
| Klucze publiczne publisherów | Zaufane klucze przechowywane na serwerze i w klientach | Wysoka |
| Metadane pakietów | SHA3-256, wersje, podpis hybrydowy | Wysoka |
| Binarne pliki pakietów | Kod dystrybuowany do końcowych użytkowników | Wysoka |
| Kanał komunikacji | Połączenia klient-serwer | Średnia |
| Konfiguracja klienta | Lista serwerów, pinned keys, app_id | Średnia |

## Aktorzy zagrożeń

| Aktor | Cel | Możliwości |
|------|-----|------------|
| Zewnętrzny atakujący | Uzyskanie nieautoryzowanego dostępu do klienta | MITM, DNS spoofing, serwer lustrzany |
| Skompromitowany publisher | Wydanie złośliwego pakietu | Dostęp do kluczy prywatnych publishera |
| Atakujący serwera | Modyfikacja danych lub pakietów | Zapis w filesystemie, zmiana bazy danych |
| Atakujący sieciowy | Podsłuch transmisji | Przechwycenie pakietów HTTP |
| Przyszły kwantowy atakujący | Łamanie tradycyjnej kryptografii | Ataki na klucze publiczne ECC/RSA |

## STRIDE

### S – Spoofing (Podszywanie się)

| Zagrożenie | Wektor | Mitygacja |
|-----------|--------|----------|
| Fałszywy serwer aktualizacji | MITM, DNS spoofing, przekierowanie ruchu | TLS w produkcji; w prototypie wymaga to dodatkowego zaufania do serwera |
| Fałszywy publisher | Zarejestrowanie nielegalnego klucza | Rejestracja klucza przez API; weryfikacja przed publikacją |
| Podmiana kluczy publicznych | Modyfikacja bazy serwera lub konfiguracji klienta | Key pinning kluczy publishera, walidacja TOFU |

### T – Tampering (Modyfikacja)

| Zagrożenie | Wektor | Mitygacja |
|-----------|--------|----------|
| Modyfikacja pakietu w tranzycie | MITM zmienia dane binarne | SHA3-256 + hybrydowy podpis |
| Modyfikacja pakietu na serwerze | Złośliwa zmiana pliku w storage | Klient weryfikuje hash/signature, pakiet przyjmuje się tylko po weryfikacji |
| Modyfikacja metadanych | Zmiana hasha, wersji lub podpisu | Podpisy pakietu obejmują treść metadanych |
| Path traversal w uploadzie | Złośliwa nazwa pliku lub app_id | Sanityzacja ścieżek i zapisywanie tylko w dozwolonych katalogach |

### R – Repudiation (Zaprzeczenie)

| Zagrożenie | Wektor | Mitygacja |
|-----------|--------|----------|
| Publisher odrzuca wydanie pakietu | Brak niepodważalnych dowodów | Podpisy cyfrowe z timestampami w metadanych |
| Serwer odrzuca publikację | Brak audytu operacji | Logi serwera, metadane podpisane przez publishera |

### I – Information Disclosure (Ujawnienie informacji)

| Zagrożenie | Wektor | Mitygacja |
|-----------|--------|----------|
| Podsłuch komunikacji | Brak szyfrowania HTTP | TLS 1.3 w środowisku produkcyjnym |
| Wyciek klucza prywatnego | Nieodpowiednie zabezpieczenia pliku | Uprawnienia plików, oddzielne konta, HSM w produkcji |
| Ujawnienie konfiguracji klienta | kradzież pliku JSON | Ograniczenie uprawnień dostępu do plików |

### D – Denial of Service (Odmowa usługi)

| Zagrożenie | Wektor | Mitygacja |
|-----------|--------|----------|
| Flooding API | Masowy ruch do endpointów | Rate limiting w produkcji |
| Duże uploady | Wyczerpanie miejsca na dysku | Ograniczenia rozmiaru pliku, monitoring przestrzeni |
| Brak dostępności bazy | Uszkodzenie lub blokada SQLite | Backup bazy, redundancja serwera |

### E – Elevation of Privilege (Eskalacja uprawnień)

| Zagrożenie | Wektor | Mitygacja |
|-----------|--------|----------|
| Kompromitacja serwera | Uruchomienie złośliwego procesu | Separacja uprawnień, konteneryzacja |
| Niezaufana publikacja pakietu | Upload od niepowołanego publishera | Żądanie metadanych i sprawdzenie publisher_id |

## Scenariusze ataków

### Scenariusz 1: MITM podczas pobierania aktualizacji

Atakujący przechwytuje odpowiedź `GET /api/download` i modyfikuje bajty pakietu.

- klient porównuje hash SHA3-256 z metadanych;
- hash zmodyfikowanego pakietu nie zgadza się;
- weryfikacja podpisów również nie przejdzie;
- aktualizacja zostaje odrzucona.

### Scenariusz 2: Próba downgrade'u

Atakujący oferuje starszą wersję pakietu.

- klient sprawdza `SemanticVersion::is_newer_than`;
- jeśli wersja jest równa lub niższa, to aktualizacja odrzucana;
- blokuje to ataki typu replay i downgrade.

### Scenariusz 3: Skompromitowany publisher

Jeżeli publisher ujawni klucz prywatny, atakujący może podpisać nowe pakiety.

- ryzyko jest krytyczne;
- w produkcyjnym scenariuszu należy zastosować rotację i unieważnianie kluczy,
- oraz centralny system audytu i kontrolę dostępu.

### Scenariusz 4: Oszustwo metadanych

Atakujący zmienia hash lub wersję w metadanych.

- klient weryfikuje hash pliku względem metadanych,
- weryfikacja podpisu hybrydowego przeciwdziała takim modyfikacjom,
- jeżeli metadane są zmienione bez prawidłowego podpisu, aktualizacja odrzucona.

### Scenariusz 5: Atak przyszły kwantowy

Jeżeli w przyszłości zostanie złamany Ed25519, Dilithium3 nadal zapewnia ochronę.

- hybrydowy podpis sprawia, że złamanie jednego mechanizmu nie wystarczy,
- system nadal wymaga ważności obydwu podpisów.

## Ryzyko resztualne i ograniczenia

| Ryzyko | Opis | Mitygacja |
|-------|------|----------|
| Kompromitacja kluczy publishera | Największe ryzyko dla autentyczności | HSM, rotacja, audyt |
| Brak TLS w prototypie | Ujawnienie komunikacji | TLS 1.3 w produkcji |
| Brak rate limiting | DoS, przeciążenie serwera | Middleware z ograniczeniem ruchu |
| Brak centralnej polityki uwierzytelniania | Nieautoryzowane publikacje | API keys, uwierzytelnianie publishera |
| Brak podpisu pliku na serwerze | Złośliwy upload | Weryfikacja uploadu i usuwanie niezgodnych plików |

## Zalecenia produkcyjne

1. Wprowadzić **TLS 1.3** oraz certyfikaty dla serwera.
2. Zapewnić **bezpieczne przechowywanie kluczy** (HSM/TPM).
3. Dodać **rate limiting** dla krytycznych endpointów.
4. Wprowadzić **autentykację publishera** i mechanizm unieważniania kluczy.
5. Monitorować **logi i integrację SIEM**.
6. Stosować **audyt publikacji** i skany integralności.

## Wnioski

- system opiera się na weryfikacji integralności i autentyczności po stronie klienta,
- najważniejszymi aktywami są klucze prywatne publisherów oraz metadane pakietów,
- hybrydowy podpis i monotoniczna weryfikacja wersji zapewniają odporność na wiele wektorów ataku,
- istnieje jednak nadal ryzyko związane z brakiem TLS, brakiem ograniczeń uploadu oraz zarządzaniem kluczami.
