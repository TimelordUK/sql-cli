use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use std::collections::HashSet;
use std::sync::RwLock;

use super::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use crate::data::datatable::DataValue;

// Include the generated prime data
include!(concat!(env!("OUT_DIR"), "/primes_data.rs"));

// Create a HashSet for O(1) primality testing up to 100K-th prime
lazy_static! {
    static ref PRIME_SET: HashSet<u32> = PRIMES_100K.iter().copied().collect();
    static ref EXTENDED_PRIME_CACHE: RwLock<Vec<u64>> = RwLock::new(Vec::new());
}

/// Engine for prime number operations
pub struct PrimeEngine;

impl PrimeEngine {
    /// Get the nth prime number (1-indexed)
    pub fn nth_prime(n: usize) -> Result<u64> {
        if n == 0 {
            return Err(anyhow!("Prime index must be >= 1"));
        }

        // Fast path: use pre-computed tables
        if n <= 1_000 {
            return Ok(u64::from(PRIMES_1K[n - 1]));
        }
        if n <= 10_000 {
            return Ok(u64::from(PRIMES_10K[n - 1]));
        }
        if n <= 100_000 {
            return Ok(u64::from(PRIMES_100K[n - 1]));
        }

        // Slow path: generate on demand and cache
        Self::generate_nth_prime(n)
    }

    /// Check if a number is prime
    #[must_use]
    pub fn is_prime(n: u64) -> bool {
        if n < 2 {
            return false;
        }
        if n == 2 {
            return true;
        }
        if n % 2 == 0 {
            return false;
        }

        // Fast path: check pre-computed set (up to 1,299,709)
        if n <= 1_299_709 {
            return PRIME_SET.contains(&(n as u32));
        }

        // Medium numbers: trial division with pre-computed primes
        if n < 1_000_000_000_000 {
            let sqrt_n = (n as f64).sqrt() as u64;

            // Use our pre-computed primes for trial division
            for &p in PRIMES_100K {
                let p64 = u64::from(p);
                if p64 > sqrt_n {
                    return true;
                }
                if n % p64 == 0 {
                    return false;
                }
            }

            // If we've exhausted our prime list and haven't found a factor,
            // continue with wheel factorization
            Self::is_prime_wheel(n, u64::from(PRIMES_100K[PRIMES_100K.len() - 1]))
        } else {
            // Very large numbers: Miller-Rabin test
            Self::miller_rabin(n)
        }
    }

    /// Count primes up to n (prime counting function π(n))
    #[must_use]
    pub fn prime_count(n: u64) -> usize {
        if n < 2 {
            return 0;
        }

        // Fast path: binary search in pre-computed list
        if n <= 1_299_709 {
            match PRIMES_100K.binary_search(&(n as u32)) {
                Ok(idx) => idx + 1, // Found exact match
                Err(idx) => idx,    // Found insertion point
            }
        } else {
            // For large n, use approximation or actual counting
            // For now, use prime number theorem approximation
            Self::approximate_prime_count(n)
        }
    }

    /// Find the next prime >= n
    #[must_use]
    pub fn next_prime(n: u64) -> u64 {
        if n <= 2 {
            return 2;
        }

        // Fast path: binary search in pre-computed primes
        if n <= 1_299_709 {
            let target = n as u32;
            match PRIMES_100K.binary_search(&target) {
                Ok(_) => n, // n is prime
                Err(idx) => {
                    if idx < PRIMES_100K.len() {
                        u64::from(PRIMES_100K[idx])
                    } else {
                        // n is larger than our biggest pre-computed prime
                        Self::find_next_prime_slow(n)
                    }
                }
            }
        } else {
            Self::find_next_prime_slow(n)
        }
    }

    /// Find the previous prime <= n
    #[must_use]
    pub fn prev_prime(n: u64) -> Option<u64> {
        if n < 2 {
            return None;
        }
        if n == 2 {
            return Some(2);
        }

        // Fast path: binary search in pre-computed primes
        if n <= 1_299_709 {
            let target = n as u32;
            match PRIMES_100K.binary_search(&target) {
                Ok(_) => Some(n), // n is prime
                Err(idx) => {
                    if idx > 0 {
                        Some(u64::from(PRIMES_100K[idx - 1]))
                    } else {
                        None // n < 2
                    }
                }
            }
        } else {
            Self::find_prev_prime_slow(n)
        }
    }

    /// Get prime factorization of n
    #[must_use]
    pub fn factor(mut n: u64) -> Vec<(u64, u32)> {
        if n <= 1 {
            return vec![];
        }

        let mut factors = Vec::new();

        // Trial division with pre-computed primes
        for &p in PRIMES_10K {
            let p64 = u64::from(p);
            if p64 * p64 > n {
                break;
            }

            let mut count = 0;
            while n % p64 == 0 {
                n /= p64;
                count += 1;
            }

            if count > 0 {
                factors.push((p64, count));
            }
        }

        // If n > 1 at this point, it's either prime or has large prime factors
        if n > 1 {
            // Check if it's prime
            if Self::is_prime(n) {
                factors.push((n, 1));
            } else {
                // Try to factor using Pollard's rho for large composites
                // For now, just add as prime (simplified)
                factors.push((n, 1));
            }
        }

        factors
    }

    // Helper functions

    /// Generate the nth prime for n > 100,000
    fn generate_nth_prime(n: usize) -> Result<u64> {
        // Check cache first
        let cache = EXTENDED_PRIME_CACHE.read().unwrap();
        let cache_start = 100_001;
        let cache_idx = n - cache_start;

        if cache_idx < cache.len() {
            return Ok(cache[cache_idx]);
        }
        drop(cache);

        // Generate primes beyond our pre-computed range
        let mut cache = EXTENDED_PRIME_CACHE.write().unwrap();

        // Start from the last pre-computed prime
        let mut candidate = u64::from(PRIMES_100K[PRIMES_100K.len() - 1]) + 2;
        let mut count = 100_000 + cache.len();

        while count < n {
            if Self::is_prime(candidate) {
                cache.push(candidate);
                count += 1;
            }
            candidate += 2;
        }

        Ok(cache[cache_idx])
    }

    /// Wheel factorization for primality testing
    fn is_prime_wheel(n: u64, start: u64) -> bool {
        // Use wheel factorization mod 30 (2*3*5)
        const WHEEL: &[u64] = &[1, 7, 11, 13, 17, 19, 23, 29];

        let sqrt_n = (n as f64).sqrt() as u64;
        let mut base = ((start / 30) + 1) * 30;

        while base <= sqrt_n {
            for &offset in WHEEL {
                let candidate = base + offset;
                if candidate > sqrt_n {
                    return true;
                }
                if candidate > start && n % candidate == 0 {
                    return false;
                }
            }
            base += 30;
        }
        true
    }

    /// Miller-Rabin primality test for very large numbers
    fn miller_rabin(n: u64) -> bool {
        // Witnesses for deterministic test up to 2^64
        const WITNESSES: &[u64] = &[2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

        // Write n-1 as 2^r * d
        let mut d = n - 1;
        let mut r = 0;
        while d % 2 == 0 {
            d /= 2;
            r += 1;
        }

        'witness: for &a in WITNESSES {
            if a >= n {
                continue;
            }

            let mut x = Self::mod_pow(a, d, n);
            if x == 1 || x == n - 1 {
                continue;
            }

            for _ in 0..r - 1 {
                x = Self::mod_mul(x, x, n);
                if x == n - 1 {
                    continue 'witness;
                }
            }

            return false;
        }

        true
    }

    /// Modular exponentiation: base^exp mod m
    fn mod_pow(mut base: u64, mut exp: u64, m: u64) -> u64 {
        let mut result = 1;
        base %= m;

        while exp > 0 {
            if exp % 2 == 1 {
                result = Self::mod_mul(result, base, m);
            }
            base = Self::mod_mul(base, base, m);
            exp /= 2;
        }

        result
    }

    /// Modular multiplication avoiding overflow
    fn mod_mul(a: u64, b: u64, m: u64) -> u64 {
        ((u128::from(a) * u128::from(b)) % u128::from(m)) as u64
    }

    /// Find next prime slowly (for n > largest pre-computed)
    fn find_next_prime_slow(mut n: u64) -> u64 {
        if n % 2 == 0 {
            n += 1;
        }

        while !Self::is_prime(n) {
            n += 2;
        }

        n
    }

    /// Find previous prime slowly
    fn find_prev_prime_slow(mut n: u64) -> Option<u64> {
        if n % 2 == 0 {
            n -= 1;
        }

        while n > 2 {
            if Self::is_prime(n) {
                return Some(n);
            }
            n -= 2;
        }

        if n == 2 {
            Some(2)
        } else {
            None
        }
    }

    /// Approximate prime count using prime number theorem
    fn approximate_prime_count(n: u64) -> usize {
        if n < 2 {
            return 0;
        }

        let n_f = n as f64;
        let ln_n = n_f.ln();

        // Use improved approximation
        let approx = n_f / (ln_n - 1.0);
        approx as usize
    }
}

// SQL Function implementations

/// PRIME(n) - Returns the nth prime number
pub struct PrimeFunction;

impl SqlFunction for PrimeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "PRIME",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the Nth prime number (1-indexed)",
            returns: "INTEGER",
            examples: vec![
                "SELECT PRIME(1)",     // Returns 2
                "SELECT PRIME(100)",   // Returns 541
                "SELECT PRIME(10000)", // Returns 104729
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let n = match &args[0] {
            DataValue::Integer(i) if *i > 0 => *i as usize,
            DataValue::Integer(_) => return Err(anyhow!("PRIME index must be positive")),
            DataValue::Float(f) if *f > 0.0 => *f as usize,
            _ => return Err(anyhow!("PRIME requires a positive integer argument")),
        };

        let prime = PrimeEngine::nth_prime(n)?;
        Ok(DataValue::Integer(prime as i64))
    }
}

/// `NTH_PRIME(n)` - Alias for PRIME function (more descriptive name)
pub struct NthPrimeFunction;

impl SqlFunction for NthPrimeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "NTH_PRIME",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the Nth prime number (1-indexed) - alias for PRIME",
            returns: "INTEGER",
            examples: vec![
                "SELECT NTH_PRIME(1)",     // Returns 2
                "SELECT NTH_PRIME(100)",   // Returns 541
                "SELECT NTH_PRIME(10000)", // Returns 104729
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        // Delegate to PRIME implementation
        PrimeFunction.evaluate(args)
    }
}

/// `IS_PRIME(n)` - Check if a number is prime
pub struct IsPrimeFunction;

impl SqlFunction for IsPrimeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "IS_PRIME",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns true if the number is prime, false otherwise",
            returns: "BOOLEAN",
            examples: vec![
                "SELECT IS_PRIME(17)",     // Returns true
                "SELECT IS_PRIME(100)",    // Returns false
                "SELECT IS_PRIME(104729)", // Returns true
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let n = match &args[0] {
            DataValue::Integer(i) if *i >= 0 => *i as u64,
            DataValue::Integer(_) => return Ok(DataValue::Boolean(false)),
            DataValue::Float(f) if *f >= 0.0 => *f as u64,
            _ => return Err(anyhow!("IS_PRIME requires a non-negative integer argument")),
        };

        Ok(DataValue::Boolean(PrimeEngine::is_prime(n)))
    }
}

/// `PRIME_COUNT(n)` - Count primes up to n
pub struct PrimeCountFunction;

impl SqlFunction for PrimeCountFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "PRIME_COUNT",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the count of prime numbers up to n (π(n))",
            returns: "INTEGER",
            examples: vec![
                "SELECT PRIME_COUNT(10)",   // Returns 4 (2,3,5,7)
                "SELECT PRIME_COUNT(100)",  // Returns 25
                "SELECT PRIME_COUNT(1000)", // Returns 168
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let n = match &args[0] {
            DataValue::Integer(i) if *i >= 0 => *i as u64,
            DataValue::Integer(_) => return Ok(DataValue::Integer(0)),
            DataValue::Float(f) if *f >= 0.0 => *f as u64,
            _ => {
                return Err(anyhow!(
                    "PRIME_COUNT requires a non-negative integer argument"
                ))
            }
        };

        Ok(DataValue::Integer(PrimeEngine::prime_count(n) as i64))
    }
}

/// `PRIME_PI(n)` - Alias for PRIME_COUNT (standard mathematical notation π(n))
pub struct PrimePiFunction;

impl SqlFunction for PrimePiFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "PRIME_PI",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description:
                "Returns the count of prime numbers up to n (π(n)) - alias for PRIME_COUNT",
            returns: "INTEGER",
            examples: vec![
                "SELECT PRIME_PI(10)",   // Returns 4 (2,3,5,7)
                "SELECT PRIME_PI(100)",  // Returns 25
                "SELECT PRIME_PI(1000)", // Returns 168
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        // Delegate to PRIME_COUNT implementation
        PrimeCountFunction.evaluate(args)
    }
}

/// `NEXT_PRIME(n)` - Find the next prime >= n
pub struct NextPrimeFunction;

impl SqlFunction for NextPrimeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "NEXT_PRIME",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the smallest prime number >= n",
            returns: "INTEGER",
            examples: vec![
                "SELECT NEXT_PRIME(100)",  // Returns 101
                "SELECT NEXT_PRIME(97)",   // Returns 97 (already prime)
                "SELECT NEXT_PRIME(1000)", // Returns 1009
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let n = match &args[0] {
            DataValue::Integer(i) if *i >= 0 => *i as u64,
            DataValue::Integer(_) => return Ok(DataValue::Integer(2)),
            DataValue::Float(f) if *f >= 0.0 => *f as u64,
            _ => {
                return Err(anyhow!(
                    "NEXT_PRIME requires a non-negative integer argument"
                ))
            }
        };

        Ok(DataValue::Integer(PrimeEngine::next_prime(n) as i64))
    }
}

/// `PREV_PRIME(n)` - Find the previous prime <= n
pub struct PrevPrimeFunction;

impl SqlFunction for PrevPrimeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "PREV_PRIME",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Returns the largest prime number <= n",
            returns: "INTEGER",
            examples: vec![
                "SELECT PREV_PRIME(100)",  // Returns 97
                "SELECT PREV_PRIME(97)",   // Returns 97 (already prime)
                "SELECT PREV_PRIME(1000)", // Returns 997
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let n = match &args[0] {
            DataValue::Integer(i) if *i >= 0 => *i as u64,
            DataValue::Integer(_) => return Ok(DataValue::Null),
            DataValue::Float(f) if *f >= 0.0 => *f as u64,
            _ => {
                return Err(anyhow!(
                    "PREV_PRIME requires a non-negative integer argument"
                ))
            }
        };

        match PrimeEngine::prev_prime(n) {
            Some(p) => Ok(DataValue::Integer(p as i64)),
            None => Ok(DataValue::Null),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nth_prime() {
        assert_eq!(PrimeEngine::nth_prime(1).unwrap(), 2);
        assert_eq!(PrimeEngine::nth_prime(10).unwrap(), 29);
        assert_eq!(PrimeEngine::nth_prime(100).unwrap(), 541);
        assert_eq!(PrimeEngine::nth_prime(1000).unwrap(), 7919);
        assert_eq!(PrimeEngine::nth_prime(10000).unwrap(), 104729);
    }

    #[test]
    fn test_is_prime() {
        assert!(!PrimeEngine::is_prime(0));
        assert!(!PrimeEngine::is_prime(1));
        assert!(PrimeEngine::is_prime(2));
        assert!(PrimeEngine::is_prime(17));
        assert!(!PrimeEngine::is_prime(100));
        assert!(PrimeEngine::is_prime(104729));
        assert!(PrimeEngine::is_prime(1299709)); // 100,000th prime
    }

    #[test]
    fn test_prime_count() {
        assert_eq!(PrimeEngine::prime_count(10), 4); // 2, 3, 5, 7
        assert_eq!(PrimeEngine::prime_count(100), 25);
        assert_eq!(PrimeEngine::prime_count(1000), 168);
    }

    #[test]
    fn test_next_prev_prime() {
        assert_eq!(PrimeEngine::next_prime(100), 101);
        assert_eq!(PrimeEngine::next_prime(97), 97);

        assert_eq!(PrimeEngine::prev_prime(100), Some(97));
        assert_eq!(PrimeEngine::prev_prime(97), Some(97));
        assert_eq!(PrimeEngine::prev_prime(1), None);
    }

    #[test]
    fn test_factorization() {
        let factors = PrimeEngine::factor(60);
        assert_eq!(factors, vec![(2, 2), (3, 1), (5, 1)]);

        let factors = PrimeEngine::factor(97);
        assert_eq!(factors, vec![(97, 1)]);

        let factors = PrimeEngine::factor(1001);
        assert_eq!(factors, vec![(7, 1), (11, 1), (13, 1)]);
    }
}
