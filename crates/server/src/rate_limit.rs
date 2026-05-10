//! # Rate Limiter
//!
//! Prosta ochrona przed brute-force atakami na endpoint logowania.
//!
//! Implementacja: sliding window counter per (IP + username).
//! Brak zewnętrznych zależności poza dashmap.
//!
//! Parametry:
//!   max_attempts: 5 prób
//!   window:       60 sekund
//!
//! Po przekroczeniu limitu: HTTP 429 Too Many Requests.
//! Reset: automatyczny po upływie okna czasowego,
//!        lub natychmiastowy po udanym logowaniu.

use dashmap::DashMap;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    /// Klucz: "IP:username", wartość: lista timestampów prób
    attempts: DashMap<String, Vec<Instant>>,
    max_attempts: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            attempts: DashMap::new(),
            max_attempts: 5,
            window: Duration::from_secs(60),
        }
    }

    /// Sprawdza czy klucz przekroczył limit i rejestruje próbę.
    ///
    /// Zwraca:
    ///   true  — żądanie dozwolone (limit nie przekroczony)
    ///   false — żądanie zablokowane (429)
    ///
    /// Stare wpisy (spoza okna) są usuwane przy każdym wywołaniu.
    pub fn check_and_record(&self, key: &str) -> bool {
        let now = Instant::now();

        let mut entry = self.attempts
            .entry(key.to_string())
            .or_default();

        // Usuń wpisy spoza okna czasowego (sliding window)
        entry.retain(|t| now.duration_since(*t) < self.window);

        if entry.len() >= self.max_attempts {
            // Limit przekroczony — nie rejestruj próby
            return false;
        }

        // Zarejestruj próbę i zezwól
        entry.push(now);
        true
    }

    /// Resetuje licznik dla klucza.
    ///
    /// Wywoływane po udanym logowaniu — nie karzemy
    /// użytkowników za własne błędy literowe przed sukcesem.
    pub fn reset(&self, key: &str) {
        self.attempts.remove(key);
    }

    /// Zwraca ile prób zostało w oknie dla klucza.
    /// Używane do logowania / debugowania.
    pub fn attempts_count(&self, key: &str) -> usize {
        let now = Instant::now();
        self.attempts
            .get(key)
            .map(|e| {
                e.iter()
                    .filter(|t| now.duration_since(**t) < self.window)
                    .count()
            })
            .unwrap_or(0)
    }
}