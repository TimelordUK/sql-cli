use anyhow::{anyhow, Result};
use std::f64::consts::PI;

use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};

/// Pythagorean theorem: c = sqrt(a^2 + b^2)
pub struct PythagorasFunction;

impl SqlFunction for PythagorasFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "PYTHAGORAS",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Calculate hypotenuse using Pythagorean theorem",
            returns: "Float (hypotenuse length)",
            examples: vec!["PYTHAGORAS(3, 4) = 5.0", "PYTHAGORAS(5, 12) = 13.0"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        let a = match &args[0] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("PYTHAGORAS requires numeric arguments")),
        };

        let b = match &args[1] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("PYTHAGORAS requires numeric arguments")),
        };

        Ok(DataValue::Float((a * a + b * b).sqrt()))
    }
}

/// Area of a circle: A = π * r^2
pub struct CircleAreaFunction;

impl SqlFunction for CircleAreaFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CIRCLE_AREA",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Calculate area of a circle given radius",
            returns: "Float (area)",
            examples: vec!["CIRCLE_AREA(1) = 3.14159...", "CIRCLE_AREA(5) = 78.5398..."],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        let radius = match &args[0] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("CIRCLE_AREA requires a numeric radius")),
        };

        Ok(DataValue::Float(PI * radius * radius))
    }
}

/// Circumference of a circle: C = 2 * π * r
pub struct CircleCircumferenceFunction;

impl SqlFunction for CircleCircumferenceFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CIRCLE_CIRCUMFERENCE",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Calculate circumference of a circle given radius",
            returns: "Float (circumference)",
            examples: vec![
                "CIRCLE_CIRCUMFERENCE(1) = 6.28318...",
                "CIRCLE_CIRCUMFERENCE(5) = 31.4159...",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        let radius = match &args[0] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("CIRCLE_CIRCUMFERENCE requires a numeric radius")),
        };

        Ok(DataValue::Float(2.0 * PI * radius))
    }
}

/// Volume of a sphere: V = (4/3) * π * r^3
pub struct SphereVolumeFunction;

impl SqlFunction for SphereVolumeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SPHERE_VOLUME",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Calculate volume of a sphere given radius",
            returns: "Float (volume)",
            examples: vec![
                "SPHERE_VOLUME(1) = 4.18879...",
                "SPHERE_VOLUME(3) = 113.097...",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        let radius = match &args[0] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("SPHERE_VOLUME requires a numeric radius")),
        };

        Ok(DataValue::Float(
            (4.0 / 3.0) * PI * radius * radius * radius,
        ))
    }
}

/// Surface area of a sphere: A = 4 * π * r^2
pub struct SphereSurfaceAreaFunction;

impl SqlFunction for SphereSurfaceAreaFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SPHERE_SURFACE_AREA",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Calculate surface area of a sphere given radius",
            returns: "Float (surface area)",
            examples: vec![
                "SPHERE_SURFACE_AREA(1) = 12.5663...",
                "SPHERE_SURFACE_AREA(3) = 113.097...",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        let radius = match &args[0] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("SPHERE_SURFACE_AREA requires a numeric radius")),
        };

        Ok(DataValue::Float(4.0 * PI * radius * radius))
    }
}

/// Area of a triangle using Heron's formula
pub struct TriangleAreaFunction;

impl SqlFunction for TriangleAreaFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "TRIANGLE_AREA",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(3),
            description: "Calculate area of a triangle given three side lengths (Heron's formula)",
            returns: "Float (area)",
            examples: vec![
                "TRIANGLE_AREA(3, 4, 5) = 6.0",
                "TRIANGLE_AREA(5, 5, 6) = 12.0",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        let a = match &args[0] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("TRIANGLE_AREA requires numeric side lengths")),
        };

        let b = match &args[1] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("TRIANGLE_AREA requires numeric side lengths")),
        };

        let c = match &args[2] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("TRIANGLE_AREA requires numeric side lengths")),
        };

        // Check triangle inequality
        if a + b <= c || a + c <= b || b + c <= a {
            return Err(anyhow!(
                "Invalid triangle: sides do not satisfy triangle inequality"
            ));
        }

        // Heron's formula: s = (a + b + c) / 2, Area = sqrt(s * (s-a) * (s-b) * (s-c))
        let s = (a + b + c) / 2.0;
        let area = (s * (s - a) * (s - b) * (s - c)).sqrt();

        Ok(DataValue::Float(area))
    }
}

/// Distance between two points in 2D space
pub struct Distance2DFunction;

impl SqlFunction for Distance2DFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "DISTANCE_2D",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(4),
            description: "Calculate distance between two points (x1, y1) and (x2, y2)",
            returns: "Float (distance)",
            examples: vec![
                "DISTANCE_2D(0, 0, 3, 4) = 5.0",
                "DISTANCE_2D(1, 1, 4, 5) = 5.0",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        let x1 = match &args[0] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("DISTANCE_2D requires numeric coordinates")),
        };

        let y1 = match &args[1] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("DISTANCE_2D requires numeric coordinates")),
        };

        let x2 = match &args[2] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("DISTANCE_2D requires numeric coordinates")),
        };

        let y2 = match &args[3] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            DataValue::Null => return Ok(DataValue::Null),
            _ => return Err(anyhow!("DISTANCE_2D requires numeric coordinates")),
        };

        let dx = x2 - x1;
        let dy = y2 - y1;
        Ok(DataValue::Float((dx * dx + dy * dy).sqrt()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pythagoras() {
        let func = PythagorasFunction;

        // 3-4-5 triangle
        let result = func
            .evaluate(&[DataValue::Integer(3), DataValue::Integer(4)])
            .unwrap();
        assert_eq!(result, DataValue::Float(5.0));

        // 5-12-13 triangle
        let result = func
            .evaluate(&[DataValue::Integer(5), DataValue::Integer(12)])
            .unwrap();
        assert_eq!(result, DataValue::Float(13.0));
    }

    #[test]
    fn test_circle_area() {
        let func = CircleAreaFunction;

        // Unit circle
        let result = func.evaluate(&[DataValue::Integer(1)]).unwrap();
        if let DataValue::Float(area) = result {
            assert!((area - PI).abs() < 1e-10);
        } else {
            panic!("Expected Float result");
        }
    }

    #[test]
    fn test_sphere_volume() {
        let func = SphereVolumeFunction;

        // Unit sphere volume = 4/3 * π
        let result = func.evaluate(&[DataValue::Integer(1)]).unwrap();
        if let DataValue::Float(volume) = result {
            let expected = (4.0 / 3.0) * PI;
            assert!((volume - expected).abs() < 1e-10);
        } else {
            panic!("Expected Float result");
        }
    }

    #[test]
    fn test_triangle_area() {
        let func = TriangleAreaFunction;

        // 3-4-5 right triangle has area 6
        let result = func
            .evaluate(&[
                DataValue::Integer(3),
                DataValue::Integer(4),
                DataValue::Integer(5),
            ])
            .unwrap();
        assert_eq!(result, DataValue::Float(6.0));

        // Invalid triangle
        let result = func.evaluate(&[
            DataValue::Integer(1),
            DataValue::Integer(2),
            DataValue::Integer(10),
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn test_distance_2d() {
        let func = Distance2DFunction;

        // Distance from origin to (3, 4) is 5
        let result = func
            .evaluate(&[
                DataValue::Integer(0),
                DataValue::Integer(0),
                DataValue::Integer(3),
                DataValue::Integer(4),
            ])
            .unwrap();
        assert_eq!(result, DataValue::Float(5.0));
    }
}
