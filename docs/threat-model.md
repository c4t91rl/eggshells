# Model Zagrożeń

## Aktywa do ochrony

| Aktywo | Opis | Wartość |
|--------|------|---------|
| Klucze prywatne publisherów | Dilithium3 + Ed25519 secret keys | Krytyczna |
| Pakiety aktualizacji | Binarne pliki dystrybuowane do klientów | Wysoka |
| Metadane pakietów | Wersje, hashe, podpisy | Wysoka |
| Kanał komunikacji | HTTP między klientem a serwerem | Średnia |
| Konfiguracja klienta | Pinned keys, server URL | Średnia |

## Aktorzy zagrożeń

| Aktor | Motywacja | Możliwości |
|-------|-----------|------------|
| Zewnętrzny atakujący | Złośliwy kod u użytkowników | Sieciowy MITM, fałszywe serwery |
| Skompromitowany publisher | Insider threat | Dostęp do kluczy prywatnych |
| Atakujący na serwer | Podmiana pakietów | Zapis do storage serwera |
| Przyszłe komputery kwantowe | Złamanie kryptografii | Algorytm Shora (RSA/ECC) |

## Macierz zagrożeń (STRIDE)

### S – Spoofing (Podszywanie się)

| Zagrożenie | Wektor | Mitygacja | Status |
|---|---|---|---|
| Fałszywy serwer aktualizacji | DNS poisoning, MITM | TLS certificate pinning (prod.) | ⚠️ Prototyp HTTP |
| Fałszywy publisher | Kompromitacja rejestracji | Weryfikacja klucza publicznego | ✅ |
| Podmiana kluczy publicznych | Modyfikacja bazy serwera | Key pinning po stronie klienta (TOFU) | ✅ |

### T – Tampering (Modyfikacja)

| Zagrożenie | Wektor | Mitygacja | Status |
|---|---|---|---|
| Modyfikacja pakietu w tranzycie | MITM | SHA3-256 + Dilithium3 + Ed25519 | ✅ |
| Modyfikacja pakietu na serwerze | Kompromitacja serwera | Podpisy weryfikowane przez klienta offline | ✅ |
| Modyfikacja metadanych | Serwer lub MITM | Podpisy obejmują zawartość pakietu | ✅ |
| Modyfikacja klienta | Tamper z exe | Self-integrity check, debugger detection | ✅ (basic) |

### R – Repudiation (Zaprzeczenie)

| Zagrożenie | Wektor | Mitygacja | Status |
|---|---|---|---|
| Publisher zaprzecza publikacji | Brak dowodów | Podpisy kryptograficzne + timestampy | ✅ |
| Serwer zaprzecza dystrybucji | Brak logów | Logi serwera + metadane z podpisem | ✅ |

### I – Information Disclosure (Ujawnienie informacji)

| Zagrożenie | Wektor | Mitygacja | Status |
|---|---|---|---|
| Podsłuch komunikacji | MITM na HTTP | TLS w produkcji (HTTP w prototypie) | ⚠️ |
| Wyciek kluczy prywatnych | Kradzież pliku | Uprawnienia 600 na Linux, ostrzeżenia | ✅ (basic) |
| Fingerprinting klienta | Metadata w requestach | Minimalne nagłówki | ⚠️ |

### D – Denial of Service (Odmowa usługi)

| Zagrożenie | Wektor | Mitygacja | Status |
|---|---|---|---|
| Flooding serwera | HTTP flood | Brak rate limitera (prototyp) | ⚠️ |
| Ogromne pakiety | Upload attack | Brak limitu rozmiaru (prototyp) | ⚠️ |

### E – Elevation of Privilege (Eskalacja uprawnień)

| Zagrożenie | Wektor | Mitygacja | Status |
|---|---|---|---|
| Złośliwy pakiet z rootkitem | Przejście weryfikacji | Multi-layer verification (hash + dual sig) | ✅ |
| Path traversal w nazwach plików | Złośliwa nazwa pliku | Sanityzacja ścieżek | ✅ |

## Kluczowe scenariusze ataków

### Scenariusz 1: MITM Attack

```
Klient              Atakujący (MITM)             Serwer
    │                        │                        │
    │   Wysyła pakiet        │                        │
    │───────────────────────>│                        │
    │                        │  Przechwytuje i        │
    │                        │  modyfikuje pakiet     │
    │                        │                        │
    │                        │   Wysyła złośliwy      │
    │                        │   pakiet do serwera    │
    │                        └───────────────────────>│
    │                                                 │
    │                                       [ Weryfikacja... ]
    │                                                 │
    │                                       [ SHA3-256 mismatch! ]
    │                                       [ Sig verification   ]
    │                                       [      FAILED!       ]
    │                                                 │
    │             KOMUNIKAT BŁĘDU                     │
    │<────────────────────────────────────────────────┤
    │                                                 │
    │                ❌ ATAK ZABLOKOWANY              │
```

**Ochrona:** SHA3-256 hash + hybrydowe podpisy cyfrowe. Atakujący nie ma kluczy prywatnych publishera, więc nie może podpisać zmodyfikowanego pakietu.

---

### Scenariusz 2: Downgrade Attack
Atakujący zmusza klienta do zainstalowania v1.0.0
(zawierającej znany CVE) zamiast aktualnej v2.0.0.

Klient sprawdza:
current_version = 2.0.0
offered_version = 1.0.0

is_safe_upgrade(2.0.0 → 1.0.0) = FALSE
❌ ATAK ZABLOKOWANY

text


**Ochrona:** Monotonic version check – klient bezwzględnie odmawia instalacji wersji <= obecnej.

---

### Scenariusz 3: Quantum Computer Attack (przyszłościowy)
Komputer kwantowy łamie Ed25519 algorytmem Shora.

Atakujący tworzy fałszywy podpis Ed25519.

Weryfikacja hybrydowa:
Dilithium3: ✅ (atakujący nie złamał)
Ed25519: ✅ (fałszywy, ale...)

overall = Dilithium3 AND Ed25519
overall = TRUE AND FALSE = FALSE
❌ ATAK ZABLOKOWANY

text


**Ochrona:** Dilithium3 jest odporny na kwantowe ataki. Hybrid scheme wymaga OBU podpisów.

---

### Scenariusz 4: Replay Attack
Atakujący przechwytuje poprawnie podpisany pakiet v1.5.0
i próbuje go ponownie dostarczyć po aktualizacji do v2.0.0.

Klient sprawdza:
current_version = 2.0.0
replayed_version = 1.5.0

is_safe_upgrade(2.0.0 → 1.5.0) = FALSE
❌ ATAK ZABLOKOWANY

text


---

### Scenariusz 5: Partial Signature Bypass
Atakujący łamie Ed25519 (np. side-channel) ale nie Dilithium.
Tworzy fałszywy podpis Ed25519 dla złośliwego pakietu.

Weryfikacja hybrydowa:
Dilithium3: ❌ FAILED (inny pakiet, inny hash)
Ed25519: ✅ fałszywy, ale...

overall = FALSE AND TRUE = FALSE
❌ ATAK ZABLOKOWANY

text


## Ryzyko rezydualne

| Ryzyko | Prawdopodobieństwo | Wpływ | Mitygacja (produkcja) |
|--------|-------------------|-------|----------------------|
| Kompromitacja klucza offline | Niskie | Krytyczny | HSM / YubiKey |
| Supply chain attack na rustc | Bardzo niskie | Krytyczny | Reproducible builds |
| Zero-day w Dilithium | Bardzo niskie | Wysoki | Hybrid scheme (Ed25519 backup) |
| Zero-day w Ed25519 | Bardzo niskie | Wysoki | Hybrid scheme (Dilithium backup) |
| TOFU key pinning bypass | Niskie | Wysoki | PKI / certificate transparency |
| HTTP (brak TLS) w prototypie | N/A (demo) | Wysoki | TLS 1.3 + cert pinning |

## Ograniczenia prototypu vs produkcja

| Aspekt | Prototyp | Produkcja |
|--------|----------|-----------|
| Transport | HTTP | TLS 1.3 + cert pinning |
| Key storage | JSON file | HSM / YubiKey / TPM |
| Key pinning | TOFU | PKI hierarchy |
| Auth publishera | Brak | OAuth2 / API keys |
| Rate limiting | Brak | Nginx / middleware |
| Auditing | Logi konsoli | SIEM integration |
| Rollback | Brak | Snapshot mechanism |
| Delta updates | Brak | Binary diffing (bsdiff) |
| Threshold sigs | Brak | k-of-n multi-publisher |
