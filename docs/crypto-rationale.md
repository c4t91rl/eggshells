# Uzasadnienie Kryptograficzne

## Dlaczego kryptografia postkwantowa?

### Zagrożenie kwantowe

Algorytm Shora (1994) pozwala kwantowym komputerom na rozwiązanie:
- **Problemu faktoryzacji** – podstawa RSA
- **Problemu logarytmu dyskretnego** – podstawa ECC (w tym Ed25519, ECDSA)

W czasie wielomianowym, co oznacza:
- RSA-2048: złamany w godziny/minuty przez wystarczająco duży komputer kwantowy
- Ed25519: złamany przez algorytm Shora na krzywych eliptycznych

### Harmonogram zagrożenia

- **2024**: Komputery kwantowe z ~1000 kubitów (za małe do kryptoanalizy)
- **2030-2040**: Szacowany próg dla praktycznych ataków na RSA/ECC
- **Problem "harvest now, decrypt later"**: Atakujący gromadzą zaszyfrowane dane **dziś**, aby odszyfrować je gdy pojawią się komputery kwantowe

### NIST Post-Quantum Standards (2024)

NIST sfinalizował w 2024 roku pierwsze postkwantowe standardy:

| Standard NIST | Algorytm | Zastosowanie |
|---|---|---|
| **FIPS 204** (ML-DSA) | CRYSTALS-Dilithium | Podpisy cyfrowe ✅ **używamy** |
| FIPS 205 (SLH-DSA) | SPHINCS+ | Podpisy cyfrowe |
| FIPS 203 (ML-KEM) | CRYSTALS-Kyber | Wymiana kluczy |

## CRYSTALS-Dilithium (ML-DSA)

### Podstawy matematyczne

Dilithium opiera się na **problemie kratowym** (lattice-based cryptography):
- **Problem Module Learning With Errors (MLWE)**: trudny zarówno dla klasycznych, jak i kwantowych komputerów
- Najlepszy znany algorytm kwantowy (BKZ): czas eksponencjalny
- **Algorytm Grovera** daje tylko kwadratowe przyspieszenie → dla 256-bit hashy zostaje ~128 bit bezpieczeństwa

### Parametry Dilithium3