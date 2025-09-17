use super::{create_single_column_table, create_two_column_table, TableGenerator};
use crate::data::datatable::{DataColumn, DataTable, DataValue};
use anyhow::Result;
use std::sync::Arc;

/// Generate prime numbers up to a limit
pub struct GeneratePrimes;

impl TableGenerator for GeneratePrimes {
    fn name(&self) -> &str {
        "GENERATE_PRIMES"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn::new("prime")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!(
                "GENERATE_PRIMES expects 1 argument (limit)"
            ));
        }

        let limit = match &args[0] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            _ => return Err(anyhow::anyhow!("GENERATE_PRIMES limit must be a number")),
        };

        let primes = sieve_of_eratosthenes(limit);
        let values: Vec<DataValue> = primes
            .into_iter()
            .map(|p| DataValue::Integer(p as i64))
            .collect();

        Ok(create_single_column_table("primes", "prime", values))
    }

    fn description(&self) -> &str {
        "Generate all prime numbers up to a given limit"
    }

    fn arg_count(&self) -> usize {
        1
    }
}

/// Generate prime factors of a number
pub struct PrimeFactors;

impl TableGenerator for PrimeFactors {
    fn name(&self) -> &str {
        "PRIME_FACTORS"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn::new("factor"), DataColumn::new("power")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("PRIME_FACTORS expects 1 argument (number)"));
        }

        let number = match &args[0] {
            DataValue::Integer(n) => *n as u64,
            DataValue::Float(f) => *f as u64,
            _ => return Err(anyhow::anyhow!("PRIME_FACTORS argument must be a number")),
        };

        let factors = factorize(number);
        let rows: Vec<(DataValue, DataValue)> = factors
            .into_iter()
            .map(|(factor, power)| {
                (
                    DataValue::Integer(factor as i64),
                    DataValue::Integer(power as i64),
                )
            })
            .collect();

        Ok(create_two_column_table("factors", "factor", "power", rows))
    }

    fn description(&self) -> &str {
        "Generate prime factorization of a number as (factor, power) pairs"
    }

    fn arg_count(&self) -> usize {
        1
    }
}

/// Generate Fibonacci sequence
pub struct Fibonacci;

impl TableGenerator for Fibonacci {
    fn name(&self) -> &str {
        "FIBONACCI"
    }

    fn columns(&self) -> Vec<DataColumn> {
        vec![DataColumn::new("n"), DataColumn::new("value")]
    }

    fn generate(&self, args: Vec<DataValue>) -> Result<Arc<DataTable>> {
        if args.len() != 1 {
            return Err(anyhow::anyhow!("FIBONACCI expects 1 argument (count)"));
        }

        let count = match &args[0] {
            DataValue::Integer(n) => *n as usize,
            DataValue::Float(f) => *f as usize,
            _ => return Err(anyhow::anyhow!("FIBONACCI count must be a number")),
        };

        let mut rows = Vec::new();
        let mut a: u64 = 0;
        let mut b: u64 = 1;

        for i in 0..count {
            rows.push((DataValue::Integer(i as i64), DataValue::Integer(a as i64)));

            if i > 0 {
                let next = a.saturating_add(b);
                a = b;
                b = next;
            } else {
                a = 1;
            }
        }

        Ok(create_two_column_table("fibonacci", "n", "value", rows))
    }

    fn description(&self) -> &str {
        "Generate Fibonacci sequence up to n terms"
    }

    fn arg_count(&self) -> usize {
        1
    }
}

/// Sieve of Eratosthenes to generate primes up to limit
fn sieve_of_eratosthenes(limit: usize) -> Vec<usize> {
    if limit < 2 {
        return vec![];
    }

    let mut is_prime = vec![true; limit + 1];
    is_prime[0] = false;
    is_prime[1] = false;

    for i in 2..=((limit as f64).sqrt() as usize) {
        if is_prime[i] {
            for j in ((i * i)..=limit).step_by(i) {
                is_prime[j] = false;
            }
        }
    }

    is_prime
        .into_iter()
        .enumerate()
        .filter_map(|(num, prime)| if prime { Some(num) } else { None })
        .collect()
}

/// Factorize a number into prime factors
fn factorize(mut n: u64) -> Vec<(u64, u32)> {
    if n <= 1 {
        return vec![];
    }

    let mut factors = Vec::new();

    // Check for factor of 2
    let mut power = 0;
    while n % 2 == 0 {
        power += 1;
        n /= 2;
    }
    if power > 0 {
        factors.push((2, power));
    }

    // Check odd factors from 3 onwards
    let mut i = 3;
    while i * i <= n {
        power = 0;
        while n % i == 0 {
            power += 1;
            n /= i;
        }
        if power > 0 {
            factors.push((i, power));
        }
        i += 2;
    }

    // If n is still greater than 1, it's a prime factor
    if n > 1 {
        factors.push((n, 1));
    }

    factors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sieve_of_eratosthenes() {
        assert_eq!(sieve_of_eratosthenes(10), vec![2, 3, 5, 7]);
        assert_eq!(
            sieve_of_eratosthenes(30),
            vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29]
        );
    }

    #[test]
    fn test_factorize() {
        assert_eq!(factorize(12), vec![(2, 2), (3, 1)]);
        assert_eq!(factorize(1260), vec![(2, 2), (3, 2), (5, 1), (7, 1)]);
        assert_eq!(factorize(17), vec![(17, 1)]);
    }
}
