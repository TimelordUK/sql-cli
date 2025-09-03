use sql_cli::data::arithmetic_evaluator::ArithmeticEvaluator;
use sql_cli::data::datatable::{DataTable, DataValue};
use sql_cli::recursive_parser::SqlExpression;

#[test]
fn test_registry_constant_functions() {
    // Create a simple test table
    let table = DataTable::new("test");
    let evaluator = ArithmeticEvaluator::new(&table);

    // Test PI function
    let pi_expr = SqlExpression::FunctionCall {
        name: "PI".to_string(),
        args: vec![],
    };
    let pi_result = evaluator.evaluate(&pi_expr, 0).unwrap();
    match pi_result {
        DataValue::Float(val) => assert!((val - std::f64::consts::PI).abs() < 1e-10),
        _ => panic!("Expected Float for PI()"),
    }

    // Test ME (mass of electron) function
    let me_expr = SqlExpression::FunctionCall {
        name: "ME".to_string(),
        args: vec![],
    };
    let me_result = evaluator.evaluate(&me_expr, 0).unwrap();
    match me_result {
        DataValue::Float(val) => assert!((val - 9.10938356e-31).abs() < 1e-40),
        _ => panic!("Expected Float for ME()"),
    }

    // Test E (Euler's number) function
    let e_expr = SqlExpression::FunctionCall {
        name: "E".to_string(),
        args: vec![],
    };
    let e_result = evaluator.evaluate(&e_expr, 0).unwrap();
    match e_result {
        DataValue::Float(val) => assert!((val - std::f64::consts::E).abs() < 1e-10),
        _ => panic!("Expected Float for E()"),
    }
}

#[test]
fn test_registry_astronomical_functions() {
    let table = DataTable::new("test");
    let evaluator = ArithmeticEvaluator::new(&table);

    // Test MASS_EARTH function
    let earth_expr = SqlExpression::FunctionCall {
        name: "MASS_EARTH".to_string(),
        args: vec![],
    };
    let earth_result = evaluator.evaluate(&earth_expr, 0).unwrap();
    match earth_result {
        DataValue::Float(val) => assert_eq!(val, 5.972e24),
        _ => panic!("Expected Float for MASS_EARTH()"),
    }

    // Test MASS_SUN function
    let sun_expr = SqlExpression::FunctionCall {
        name: "MASS_SUN".to_string(),
        args: vec![],
    };
    let sun_result = evaluator.evaluate(&sun_expr, 0).unwrap();
    match sun_result {
        DataValue::Float(val) => assert_eq!(val, 1.989e30),
        _ => panic!("Expected Float for MASS_SUN()"),
    }

    // Test AU (astronomical unit) function
    let au_expr = SqlExpression::FunctionCall {
        name: "AU".to_string(),
        args: vec![],
    };
    let au_result = evaluator.evaluate(&au_expr, 0).unwrap();
    match au_result {
        DataValue::Float(val) => assert_eq!(val, 1.496e11),
        _ => panic!("Expected Float for AU()"),
    }

    // Test LIGHT_YEAR function
    let ly_expr = SqlExpression::FunctionCall {
        name: "LIGHT_YEAR".to_string(),
        args: vec![],
    };
    let ly_result = evaluator.evaluate(&ly_expr, 0).unwrap();
    match ly_result {
        DataValue::Float(val) => assert_eq!(val, 9.461e15),
        _ => panic!("Expected Float for LIGHT_YEAR()"),
    }
}

#[test]
fn test_registry_chemistry_functions() {
    let table = DataTable::new("test");
    let evaluator = ArithmeticEvaluator::new(&table);

    // Test AVOGADRO function
    let avogadro_expr = SqlExpression::FunctionCall {
        name: "AVOGADRO".to_string(),
        args: vec![],
    };
    let avogadro_result = evaluator.evaluate(&avogadro_expr, 0).unwrap();
    match avogadro_result {
        DataValue::Float(val) => assert!((val - 6.022140857e23).abs() < 1e20),
        _ => panic!("Expected Float for AVOGADRO()"),
    }

    // Test ATOMIC_MASS function with Carbon
    let carbon_mass_expr = SqlExpression::FunctionCall {
        name: "ATOMIC_MASS".to_string(),
        args: vec![SqlExpression::StringLiteral("Carbon".to_string())],
    };
    let carbon_result = evaluator.evaluate(&carbon_mass_expr, 0).unwrap();
    match carbon_result {
        DataValue::Float(val) => assert!((val - 12.01).abs() < 0.01),
        _ => panic!("Expected Float for ATOMIC_MASS('Carbon')"),
    }

    // Test ATOMIC_NUMBER function with Gold
    let gold_number_expr = SqlExpression::FunctionCall {
        name: "ATOMIC_NUMBER".to_string(),
        args: vec![SqlExpression::StringLiteral("Au".to_string())],
    };
    let gold_result = evaluator.evaluate(&gold_number_expr, 0).unwrap();
    match gold_result {
        DataValue::Integer(val) => assert_eq!(val, 79),
        _ => panic!("Expected Integer for ATOMIC_NUMBER('Au')"),
    }
}

#[test]
fn test_registry_function_errors() {
    let table = DataTable::new("test");
    let evaluator = ArithmeticEvaluator::new(&table);

    // Test function with wrong number of arguments
    let pi_with_args = SqlExpression::FunctionCall {
        name: "PI".to_string(),
        args: vec![SqlExpression::NumberLiteral("1.0".to_string())],
    };
    assert!(evaluator.evaluate(&pi_with_args, 0).is_err());

    // Test unknown element for ATOMIC_MASS
    let unknown_element = SqlExpression::FunctionCall {
        name: "ATOMIC_MASS".to_string(),
        args: vec![SqlExpression::StringLiteral("Xyz".to_string())],
    };
    assert!(evaluator.evaluate(&unknown_element, 0).is_err());

    // Test ATOMIC_MASS with wrong type argument
    let wrong_type = SqlExpression::FunctionCall {
        name: "ATOMIC_MASS".to_string(),
        args: vec![SqlExpression::NumberLiteral("42".to_string())],
    };
    assert!(evaluator.evaluate(&wrong_type, 0).is_err());
}
