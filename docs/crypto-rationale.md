# Uzasadnienie Kryptograficzne

## Cel architektury kryptograficznej

Celem systemu jest zapewnienie bezpiecznej dystrybucji pakietów aktualizacji,
której integralność i autentyczność mogą być zweryfikowane przez klienta bez
zależności od zaufania do serwera hostingowego.

Kluczowe wymagania:
- autentyczność pakietu,
- integralność zawartości,
- odporność na ataki postkwantowe,
- ochrona przed ponownym użyciem (replay) i downgrade.

## Dlaczego hybrydowy podpis?

System używa hybrydowego podejścia: jednocześnie
**CRYSTALS-Dilithium3** i **Ed25519**.

### Uzasadnienie
- **Dilithium3** daje odporność postkwantową; jego bezpieczeństwo opiera się na problemach kratowych,
  które nie są efektywnie rozwiązywane przez znane algorytmy kwantowe.
- **Ed25519** jest szybki, powszechnie stosowany i dobrze przebadany w klasycznych systemach.
- Hybrydowy podpis zwiększa odporność na regresję:
  - jeśli pojawi się słabość w Ed25519, nadal chroni podpis Dilithium3,
  - jeśli pojawi się słabość w Dilithium3, nadal chroni podpis klasyczny.

### Dlaczego nie tylko Ed25519?
Ed25519 jest bardzo dobry w klasycznych warunkach, ale
należy do klasy algorytmów opartych na problemie logarytmu dyskretnego.
Przy wystarczająco dużym komputerze kwantowym może zostać złamany przez algorytm Shora.

### Dlaczego nie tylko Dilithium3?
Dilithium3 ma większe rozmiary kluczy i podpisów niż Ed25519.
Dodanie Ed25519 poprawia interoperacyjność i wydajność w scenariuszach klasycznych,
nie obniżając ogólnej bezpieczeństwa.

## Dlaczego SHA3-256?

SHA3-256 zapewnia integralność danych z silnym zabezpieczeniem przed typowymi
słabościami SHA-2, zwłaszcza atakami typu length-extension.

W systemie:
- metadane pakietu zawierają hash SHA3-256 pliku,
- klient ponownie oblicza hash otrzymanego pliku i porównuje go z metadanymi,
- następnie weryfikuje podpis względem tego hasha.

## Dlaczego klucze hybrydowe?

Każdy publisher posiada `HybridPublicKey`, który zawiera:
- klucz publiczny Dilithium3,
- klucz publiczny Ed25519.

W metadanych pakietu przechowywany jest `HybridSignature`, czyli dwie części podpisu.
Klient wymaga, aby obie części były poprawne. Dzięki temu atak musi złamać
obydwa mechanizmy, co znacząco podnosi koszty ataku.

## Użyte algorytmy i właściwości

| Cel | Algorytm | Właściwości |
|-----|----------|-------------|
| Integralność pliku | SHA3-256 | odporność na length-extension, NIST |
| Podpis postkwantowy | CRYSTALS-Dilithium3 | NIST L3, odporność na kwantowe ataki |
| Podpis klasyczny | Ed25519 | szybka weryfikacja, małe klucze |
| Signature verification | hybrydowe | obie podpisy muszą być poprawne |

## Przepływ kryptograficzny

1. Publisher oblicza hash SHA3-256 pliku.
2. Publisher tworzy metadane pakietu zawierające hash i wersję.
3. Publisher podpisuje metadane/treść pakietu przy użyciu `HybridKeyPair`.
4. Serwer zapisuje metadane razem z hybrydowym podpisem.
5. Klient pobiera metadane i plik.
6. Klient oblicza hash SHA3-256 pliku.
7. Klient weryfikuje podpisy Dilithium3 i Ed25519 względem zaufanego klucza publicznego.

## Co chroni hybrydowe podejście?

### Przykład ataku 1: złamanie Ed25519
- Atakujący może sfałszować część klasyczną,
- ale musi również sfalsyfikować wynik Dilithium3,
- co jest w praktyce nierealne w modelu postkwantowym.

### Przykład ataku 2: złamanie Dilithium3
- Atakujący może stworzyć hash i próbować zastąpić część Dilithium3,
- jednak także musi wygenerować poprawny podpis Ed25519,
- a to nadal wymaga klasycznego złamania klucza.

## Kluczowe właściwości zabezpieczeń

- **Integralność:** SHA3-256 sprawdza, że plik nie został zmodyfikowany.
- **Autentyczność:** hybrydowy podpis wiąże plik z publisherem.
- **Odrzucenie downgrade:** mechanizm `SemanticVersion::is_newer_than` blokuje starsze lub równe wersje.
- **Oddzielenie metadanych i binariów:** metadane muszą być opublikowane przed uploadem binariów.

## Ograniczenia i dalsze ulepszenia

### Prototyp:
- używa HTTP w demo; produkcja powinna używać TLS 1.3,
- klucze prywatne są przechowywane w plikach JSON; produkcja powinna używać HSM/TPM,
- model TOFU jest użyteczny, ale w docelowej architekturze warto zastosować PKI albo certyfikaty.

### Dalsze ulepszenia:
- walidacja podpisu podczas uploadu już na serwerze,
- mechanizmy rotacji kluczy i unieważniania,
- audytowanie podpisów i operacji publikacji.
