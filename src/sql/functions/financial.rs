use anyhow::{anyhow, Result};
use std::collections::VecDeque;

use super::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};
use crate::data::datatable::DataValue;

/// RETURNS function - calculates simple returns from price series
/// Returns = (price[t] - price[t-1]) / price[t-1]
pub struct ReturnsFunction;

impl SqlFunction for ReturnsFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "RETURNS",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Calculate returns from current and previous price",
            returns: "FLOAT",
            examples: vec![
                "SELECT RETURNS(close, LAG(close) OVER (ORDER BY date)) FROM stocks",
                "SELECT RETURNS(100, 95)", // Returns 0.0526 (5.26% gain)
                "SELECT RETURNS(95, 100)", // Returns -0.05 (5% loss)
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow!(
                "RETURNS expects exactly 2 arguments: current_price, previous_price"
            ));
        }

        let current = match &args[0] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("RETURNS expects numeric values")),
        };

        let previous = match &args[1] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("RETURNS expects numeric values")),
        };

        if previous == 0.0 {
            return Err(anyhow!("Cannot calculate returns with previous price of 0"));
        }

        let returns = (current - previous) / previous;
        Ok(DataValue::Float(returns))
    }
}

/// LOG_RETURNS function - calculates logarithmic returns
/// Log Returns = ln(price[t] / price[t-1])
pub struct LogReturnsFunction;

impl SqlFunction for LogReturnsFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LOG_RETURNS",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Calculate logarithmic returns from current and previous price",
            returns: "FLOAT",
            examples: vec![
                "SELECT LOG_RETURNS(close, LAG(close) OVER (ORDER BY date)) FROM stocks",
                "SELECT LOG_RETURNS(100, 95)", // Returns 0.0513 (ln(100/95))
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 2 {
            return Err(anyhow!(
                "LOG_RETURNS expects exactly 2 arguments: current_price, previous_price"
            ));
        }

        let current = match &args[0] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("LOG_RETURNS expects numeric values")),
        };

        let previous = match &args[1] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("LOG_RETURNS expects numeric values")),
        };

        if previous <= 0.0 || current <= 0.0 {
            return Err(anyhow!(
                "Cannot calculate log returns with non-positive prices"
            ));
        }

        let log_returns = (current / previous).ln();
        Ok(DataValue::Float(log_returns))
    }
}

/// VOLATILITY function - calculates standard deviation of returns
/// This is a simplified version that takes an array of returns
pub struct VolatilityFunction;

impl SqlFunction for VolatilityFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "VOLATILITY",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Variadic,
            description: "Calculate volatility (standard deviation) of returns",
            returns: "FLOAT",
            examples: vec![
                "SELECT VOLATILITY(0.01, -0.02, 0.015, -0.005, 0.008)",
                "WITH returns AS (SELECT RETURNS(close, LAG(close) OVER (ORDER BY date)) as r FROM stocks) SELECT VOLATILITY(r) FROM returns",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() {
            return Err(anyhow!("VOLATILITY requires at least one value"));
        }

        let mut values = Vec::new();
        for arg in args {
            match arg {
                DataValue::Integer(i) => values.push(*i as f64),
                DataValue::Float(f) => values.push(*f),
                DataValue::Null => continue, // Skip nulls
                _ => return Err(anyhow!("VOLATILITY expects numeric values")),
            }
        }

        if values.is_empty() {
            return Ok(DataValue::Null);
        }

        if values.len() == 1 {
            return Ok(DataValue::Float(0.0)); // No variation with single value
        }

        // Calculate mean
        let mean = values.iter().sum::<f64>() / values.len() as f64;

        // Calculate variance
        let variance =
            values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64; // Sample variance (n-1)

        // Standard deviation
        let std_dev = variance.sqrt();
        Ok(DataValue::Float(std_dev))
    }
}

/// SHARPE_RATIO function - calculates Sharpe ratio
/// Sharpe = (mean_return - risk_free_rate) / volatility
pub struct SharpeRatioFunction;

impl SqlFunction for SharpeRatioFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SHARPE_RATIO",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(3),
            description: "Calculate Sharpe ratio: (mean_return - risk_free_rate) / volatility",
            returns: "FLOAT",
            examples: vec![
                "SELECT SHARPE_RATIO(0.08, 0.02, 0.15)", // 8% return, 2% risk-free, 15% volatility = 0.4
                "SELECT SHARPE_RATIO(mean_return, 0.02, volatility) FROM portfolio_stats",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.len() != 3 {
            return Err(anyhow!(
                "SHARPE_RATIO expects 3 arguments: mean_return, risk_free_rate, volatility"
            ));
        }

        let mean_return = match &args[0] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("SHARPE_RATIO expects numeric values")),
        };

        let risk_free_rate = match &args[1] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => 0.0, // Default to 0 if null
            _ => return Err(anyhow!("SHARPE_RATIO expects numeric values")),
        };

        let volatility = match &args[2] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("SHARPE_RATIO expects numeric values")),
        };

        if volatility == 0.0 {
            return Err(anyhow!(
                "Cannot calculate Sharpe ratio with zero volatility"
            ));
        }

        let sharpe = (mean_return - risk_free_rate) / volatility;
        Ok(DataValue::Float(sharpe))
    }
}

/// STDDEV function - calculates standard deviation (sample)
/// This is an alias for VOLATILITY but more SQL-standard
pub struct StdDevFunction;

impl SqlFunction for StdDevFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "STDDEV",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Variadic,
            description: "Calculate sample standard deviation",
            returns: "FLOAT",
            examples: vec![
                "SELECT STDDEV(1, 2, 3, 4, 5)", // Returns 1.58
                "SELECT STDDEV(returns) OVER (ORDER BY date ROWS 19 PRECEDING) FROM stocks",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        // Reuse volatility implementation
        VolatilityFunction.evaluate(args)
    }
}

/// Register all financial functions
pub fn register_financial_functions(registry: &mut super::FunctionRegistry) {
    registry.register(Box::new(ReturnsFunction));
    registry.register(Box::new(LogReturnsFunction));
    registry.register(Box::new(VolatilityFunction));
    registry.register(Box::new(StdDevFunction));
    registry.register(Box::new(SharpeRatioFunction));
}
