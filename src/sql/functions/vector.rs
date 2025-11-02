use anyhow::{anyhow, Result};

use crate::data::datatable::DataValue;
use crate::sql::functions::{ArgCount, FunctionCategory, FunctionSignature, SqlFunction};

/// Helper function to extract vector from DataValue
fn get_vector(value: &DataValue) -> Result<Vec<f64>> {
    match value {
        DataValue::Vector(v) => Ok(v.clone()),
        DataValue::String(s) => parse_vector_string(s),
        _ => Err(anyhow!("Expected vector, got {:?}", value.data_type())),
    }
}

/// Parse vector from string representation: "[1,2,3]" or "1 2 3"
fn parse_vector_string(s: &str) -> Result<Vec<f64>> {
    let trimmed = s.trim();

    // Handle "[1,2,3]" format
    let content = if trimmed.starts_with('[') && trimmed.ends_with(']') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    // Parse components (comma or space separated)
    let components: Result<Vec<f64>> = if content.contains(',') {
        content
            .split(',')
            .map(|s| {
                s.trim()
                    .parse::<f64>()
                    .map_err(|e| anyhow!("Failed to parse vector component '{}': {}", s.trim(), e))
            })
            .collect()
    } else {
        content
            .split_whitespace()
            .map(|s| {
                s.parse::<f64>()
                    .map_err(|e| anyhow!("Failed to parse vector component '{}': {}", s, e))
            })
            .collect()
    };

    components
}

/// VEC(x, y, z, ...) - Construct a vector from components
pub struct VecFunction;

impl SqlFunction for VecFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "VEC",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Variadic,
            description: "Construct a vector from numeric components",
            returns: "Vector",
            examples: vec!["SELECT VEC(1, 2, 3)", "SELECT VEC(10, 20)"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        if args.is_empty() {
            return Err(anyhow!("VEC() requires at least one argument"));
        }

        let components: Result<Vec<f64>> = args
            .iter()
            .map(|arg| match arg {
                DataValue::Integer(i) => Ok(*i as f64),
                DataValue::Float(f) => Ok(*f),
                DataValue::Null => Err(anyhow!("Cannot create vector with NULL component")),
                _ => Err(anyhow!(
                    "VEC() requires numeric arguments, got {:?}",
                    arg.data_type()
                )),
            })
            .collect();

        Ok(DataValue::Vector(components?))
    }
}

/// VEC_ADD(v1, v2) - Add two vectors element-wise
pub struct VecAddFunction;

impl SqlFunction for VecAddFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "VEC_ADD",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Add two vectors element-wise",
            returns: "Vector",
            examples: vec![
                "SELECT VEC_ADD(VEC(1,2,3), VEC(4,5,6))",
                "SELECT VEC_ADD(position, velocity)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let v1 = get_vector(&args[0])?;
        let v2 = get_vector(&args[1])?;

        if v1.len() != v2.len() {
            return Err(anyhow!(
                "Vector dimension mismatch: {} != {}",
                v1.len(),
                v2.len()
            ));
        }

        let result: Vec<f64> = v1.iter().zip(v2.iter()).map(|(a, b)| a + b).collect();
        Ok(DataValue::Vector(result))
    }
}

/// VEC_SUB(v1, v2) - Subtract two vectors element-wise
pub struct VecSubFunction;

impl SqlFunction for VecSubFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "VEC_SUB",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Subtract two vectors element-wise (v1 - v2)",
            returns: "Vector",
            examples: vec![
                "SELECT VEC_SUB(VEC(10,20,30), VEC(1,2,3))",
                "SELECT VEC_SUB(position_end, position_start)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let v1 = get_vector(&args[0])?;
        let v2 = get_vector(&args[1])?;

        if v1.len() != v2.len() {
            return Err(anyhow!(
                "Vector dimension mismatch: {} != {}",
                v1.len(),
                v2.len()
            ));
        }

        let result: Vec<f64> = v1.iter().zip(v2.iter()).map(|(a, b)| a - b).collect();
        Ok(DataValue::Vector(result))
    }
}

/// VEC_SCALE(vector, scalar) - Multiply vector by scalar
pub struct VecScaleFunction;

impl SqlFunction for VecScaleFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "VEC_SCALE",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Multiply vector by scalar value",
            returns: "Vector",
            examples: vec![
                "SELECT VEC_SCALE(VEC(1,2,3), 2.5)",
                "SELECT VEC_SCALE(velocity, time)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let v = get_vector(&args[0])?;
        let scalar = match &args[1] {
            DataValue::Integer(i) => *i as f64,
            DataValue::Float(f) => *f,
            _ => {
                return Err(anyhow!(
                    "Scalar must be numeric, got {:?}",
                    args[1].data_type()
                ))
            }
        };

        let result: Vec<f64> = v.iter().map(|x| x * scalar).collect();
        Ok(DataValue::Vector(result))
    }
}

/// VEC_DOT(v1, v2) - Compute dot product
pub struct VecDotFunction;

impl SqlFunction for VecDotFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "VEC_DOT",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Compute dot product of two vectors",
            returns: "Float",
            examples: vec![
                "SELECT VEC_DOT(VEC(1,2,3), VEC(4,5,6))",
                "SELECT VEC_DOT(velocity1, velocity2)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let v1 = get_vector(&args[0])?;
        let v2 = get_vector(&args[1])?;

        if v1.len() != v2.len() {
            return Err(anyhow!(
                "Vector dimension mismatch: {} != {}",
                v1.len(),
                v2.len()
            ));
        }

        let dot_product: f64 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        Ok(DataValue::Float(dot_product))
    }
}

/// VEC_MAG(vector) - Compute magnitude (length) of vector
pub struct VecMagFunction;

impl SqlFunction for VecMagFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "VEC_MAG",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Compute magnitude (length) of a vector",
            returns: "Float",
            examples: vec!["SELECT VEC_MAG(VEC(3,4))", "SELECT VEC_MAG(velocity)"],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let v = get_vector(&args[0])?;
        let magnitude = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        Ok(DataValue::Float(magnitude))
    }
}

/// VEC_NORMALIZE(vector) - Normalize vector to unit length
pub struct VecNormalizeFunction;

impl SqlFunction for VecNormalizeFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "VEC_NORMALIZE",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(1),
            description: "Normalize vector to unit length",
            returns: "Vector",
            examples: vec![
                "SELECT VEC_NORMALIZE(VEC(3,4))",
                "SELECT VEC_NORMALIZE(direction)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let v = get_vector(&args[0])?;
        let magnitude = v.iter().map(|x| x * x).sum::<f64>().sqrt();

        if magnitude == 0.0 {
            return Err(anyhow!("Cannot normalize zero vector"));
        }

        let normalized: Vec<f64> = v.iter().map(|x| x / magnitude).collect();
        Ok(DataValue::Vector(normalized))
    }
}

/// VEC_DISTANCE(v1, v2) - Compute Euclidean distance between two vectors
pub struct VecDistanceFunction;

impl SqlFunction for VecDistanceFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "VEC_DISTANCE",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Compute Euclidean distance between two vectors",
            returns: "Float",
            examples: vec![
                "SELECT VEC_DISTANCE(VEC(0,0), VEC(3,4))",
                "SELECT VEC_DISTANCE(position1, position2)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let v1 = get_vector(&args[0])?;
        let v2 = get_vector(&args[1])?;

        if v1.len() != v2.len() {
            return Err(anyhow!(
                "Vector dimension mismatch: {} != {}",
                v1.len(),
                v2.len()
            ));
        }

        let distance = v1
            .iter()
            .zip(v2.iter())
            .map(|(a, b)| (a - b).powi(2))
            .sum::<f64>()
            .sqrt();

        Ok(DataValue::Float(distance))
    }
}

/// VEC_CROSS(v1, v2) - Compute cross product (3D only)
pub struct VecCrossFunction;

impl SqlFunction for VecCrossFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "VEC_CROSS",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Compute cross product of two 3D vectors",
            returns: "Vector",
            examples: vec![
                "SELECT VEC_CROSS(VEC(1,0,0), VEC(0,1,0))",
                "SELECT VEC_CROSS(velocity, force)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let v1 = get_vector(&args[0])?;
        let v2 = get_vector(&args[1])?;

        if v1.len() != 3 || v2.len() != 3 {
            return Err(anyhow!(
                "VEC_CROSS requires 3D vectors, got dimensions {} and {}",
                v1.len(),
                v2.len()
            ));
        }

        let cross = vec![
            v1[1] * v2[2] - v1[2] * v2[1],
            v1[2] * v2[0] - v1[0] * v2[2],
            v1[0] * v2[1] - v1[1] * v2[0],
        ];

        Ok(DataValue::Vector(cross))
    }
}

/// VEC_ANGLE(v1, v2) - Compute angle between two vectors in radians
pub struct VecAngleFunction;

impl SqlFunction for VecAngleFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "VEC_ANGLE",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(2),
            description: "Compute angle between two vectors in radians",
            returns: "Float",
            examples: vec![
                "SELECT VEC_ANGLE(VEC(1,0), VEC(0,1))",
                "SELECT VEC_ANGLE(direction1, direction2)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let v1 = get_vector(&args[0])?;
        let v2 = get_vector(&args[1])?;

        if v1.len() != v2.len() {
            return Err(anyhow!(
                "Vector dimension mismatch: {} != {}",
                v1.len(),
                v2.len()
            ));
        }

        let dot: f64 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let mag1 = v1.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag2 = v2.iter().map(|x| x * x).sum::<f64>().sqrt();

        if mag1 == 0.0 || mag2 == 0.0 {
            return Err(anyhow!("Cannot compute angle with zero vector"));
        }

        let cos_angle = dot / (mag1 * mag2);
        // Clamp to [-1, 1] to handle floating point errors
        let cos_angle = cos_angle.max(-1.0).min(1.0);
        let angle = cos_angle.acos();

        Ok(DataValue::Float(angle))
    }
}

/// LINE_INTERSECT(p1, p2, p3, p4) - Find exact intersection point of two 2D lines
pub struct LineIntersectFunction;

impl SqlFunction for LineIntersectFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LINE_INTERSECT",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(4),
            description: "Find intersection point of two 2D lines (returns NULL if parallel)",
            returns: "Vector or NULL",
            examples: vec![
                "SELECT LINE_INTERSECT(VEC(0,0), VEC(4,4), VEC(0,4), VEC(4,0))",
                "SELECT LINE_INTERSECT(line1_p1, line1_p2, line2_p1, line2_p2)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        // Get four 2D points defining two lines
        let p1 = get_vector(&args[0])?;
        let p2 = get_vector(&args[1])?;
        let p3 = get_vector(&args[2])?;
        let p4 = get_vector(&args[3])?;

        if p1.len() != 2 || p2.len() != 2 || p3.len() != 2 || p4.len() != 2 {
            return Err(anyhow!("LINE_INTERSECT requires 2D points"));
        }

        // Line 1: p1 + t * (p2 - p1)
        // Line 2: p3 + s * (p4 - p3)
        //
        // Solving: p1 + t*(p2-p1) = p3 + s*(p4-p3)
        // This gives us two equations (for x and y)
        //
        // Using determinant method:
        let x1 = p1[0];
        let y1 = p1[1];
        let x2 = p2[0];
        let y2 = p2[1];
        let x3 = p3[0];
        let y3 = p3[1];
        let x4 = p4[0];
        let y4 = p4[1];

        let denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);

        // If denominator is 0, lines are parallel
        if denom.abs() < 1e-10 {
            return Ok(DataValue::Null);
        }

        // Calculate intersection point
        let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;

        let intersect_x = x1 + t * (x2 - x1);
        let intersect_y = y1 + t * (y2 - y1);

        Ok(DataValue::Vector(vec![intersect_x, intersect_y]))
    }
}

/// SEGMENT_INTERSECT(p1, p2, p3, p4) - Check if two line segments intersect
pub struct SegmentIntersectFunction;

impl SqlFunction for SegmentIntersectFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "SEGMENT_INTERSECT",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(4),
            description:
                "Check if two 2D line segments intersect (returns intersection point or NULL)",
            returns: "Vector or NULL",
            examples: vec![
                "SELECT SEGMENT_INTERSECT(VEC(0,0), VEC(2,2), VEC(0,2), VEC(2,0))",
                "SELECT SEGMENT_INTERSECT(seg1_p1, seg1_p2, seg2_p1, seg2_p2)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        // Get four 2D points defining two segments
        let p1 = get_vector(&args[0])?;
        let p2 = get_vector(&args[1])?;
        let p3 = get_vector(&args[2])?;
        let p4 = get_vector(&args[3])?;

        if p1.len() != 2 || p2.len() != 2 || p3.len() != 2 || p4.len() != 2 {
            return Err(anyhow!("SEGMENT_INTERSECT requires 2D points"));
        }

        let x1 = p1[0];
        let y1 = p1[1];
        let x2 = p2[0];
        let y2 = p2[1];
        let x3 = p3[0];
        let y3 = p3[1];
        let x4 = p4[0];
        let y4 = p4[1];

        let denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);

        // If denominator is 0, segments are parallel
        if denom.abs() < 1e-10 {
            return Ok(DataValue::Null);
        }

        // Calculate parameters t and s
        let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;
        let s = ((x1 - x3) * (y1 - y2) - (y1 - y3) * (x1 - x2)) / denom;

        // Check if intersection is within both segments (t and s in [0, 1])
        if t >= 0.0 && t <= 1.0 && s >= 0.0 && s <= 1.0 {
            let intersect_x = x1 + t * (x2 - x1);
            let intersect_y = y1 + t * (y2 - y1);
            Ok(DataValue::Vector(vec![intersect_x, intersect_y]))
        } else {
            // Segments don't intersect (they would if extended to lines)
            Ok(DataValue::Null)
        }
    }
}

/// CLOSEST_POINT_ON_LINE(point, line_point, line_dir) - Find closest point on line to given point
pub struct ClosestPointOnLineFunction;

impl SqlFunction for ClosestPointOnLineFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "CLOSEST_POINT_ON_LINE",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(3),
            description: "Find closest point on a line to a given point (projection)",
            returns: "Vector",
            examples: vec![
                "SELECT CLOSEST_POINT_ON_LINE(VEC(2,2), VEC(0,0), VEC(1,0))",
                "SELECT CLOSEST_POINT_ON_LINE(point, line_start, line_direction)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let point = get_vector(&args[0])?;
        let line_point = get_vector(&args[1])?;
        let line_dir = get_vector(&args[2])?;

        if point.len() != line_point.len() || point.len() != line_dir.len() {
            return Err(anyhow!(
                "All vectors must have same dimension, got {}, {}, {}",
                point.len(),
                line_point.len(),
                line_dir.len()
            ));
        }

        // Vector from line point to target point
        let to_point: Vec<f64> = point
            .iter()
            .zip(line_point.iter())
            .map(|(p, lp)| p - lp)
            .collect();

        // Project onto line direction: t = (to_point · line_dir) / (line_dir · line_dir)
        let dot_product: f64 = to_point
            .iter()
            .zip(line_dir.iter())
            .map(|(a, b)| a * b)
            .sum();
        let dir_mag_sq: f64 = line_dir.iter().map(|x| x * x).sum();

        if dir_mag_sq < 1e-10 {
            return Err(anyhow!("Line direction vector cannot be zero"));
        }

        let t = dot_product / dir_mag_sq;

        // Closest point = line_point + t * line_dir
        let closest: Vec<f64> = line_point
            .iter()
            .zip(line_dir.iter())
            .map(|(lp, ld)| lp + t * ld)
            .collect();

        Ok(DataValue::Vector(closest))
    }
}

/// POINT_LINE_DISTANCE(point, line_point, line_dir) - Distance from point to line
pub struct PointLineDistanceFunction;

impl SqlFunction for PointLineDistanceFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "POINT_LINE_DISTANCE",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(3),
            description: "Compute perpendicular distance from point to line",
            returns: "Float",
            examples: vec![
                "SELECT POINT_LINE_DISTANCE(VEC(2,2), VEC(0,0), VEC(1,0))",
                "SELECT POINT_LINE_DISTANCE(point, line_start, line_direction)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let point = get_vector(&args[0])?;
        let line_point = get_vector(&args[1])?;
        let line_dir = get_vector(&args[2])?;

        // For 2D and 3D, we can use the cross product method
        if point.len() == 2 {
            // Extend to 3D for cross product
            let point_3d = vec![point[0], point[1], 0.0];
            let line_point_3d = vec![line_point[0], line_point[1], 0.0];
            let line_dir_3d = vec![line_dir[0], line_dir[1], 0.0];

            let to_point: Vec<f64> = point_3d
                .iter()
                .zip(line_point_3d.iter())
                .map(|(p, lp)| p - lp)
                .collect();

            // Cross product
            let cross_x = to_point[1] * line_dir_3d[2] - to_point[2] * line_dir_3d[1];
            let cross_y = to_point[2] * line_dir_3d[0] - to_point[0] * line_dir_3d[2];
            let cross_z = to_point[0] * line_dir_3d[1] - to_point[1] * line_dir_3d[0];

            let cross_mag = (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt();
            let dir_mag = (line_dir[0] * line_dir[0] + line_dir[1] * line_dir[1]).sqrt();

            if dir_mag < 1e-10 {
                return Err(anyhow!("Line direction cannot be zero"));
            }

            Ok(DataValue::Float(cross_mag / dir_mag))
        } else if point.len() == 3 {
            let to_point: Vec<f64> = point
                .iter()
                .zip(line_point.iter())
                .map(|(p, lp)| p - lp)
                .collect();

            // 3D cross product
            let cross_x = to_point[1] * line_dir[2] - to_point[2] * line_dir[1];
            let cross_y = to_point[2] * line_dir[0] - to_point[0] * line_dir[2];
            let cross_z = to_point[0] * line_dir[1] - to_point[1] * line_dir[0];

            let cross_mag = (cross_x * cross_x + cross_y * cross_y + cross_z * cross_z).sqrt();
            let dir_mag = line_dir.iter().map(|x| x * x).sum::<f64>().sqrt();

            if dir_mag < 1e-10 {
                return Err(anyhow!("Line direction cannot be zero"));
            }

            Ok(DataValue::Float(cross_mag / dir_mag))
        } else {
            Err(anyhow!(
                "POINT_LINE_DISTANCE only supports 2D and 3D, got {}D",
                point.len()
            ))
        }
    }
}

/// LINE_REFLECT_POINT(point, line_point, line_dir) - Reflect point across line
pub struct LineReflectPointFunction;

impl SqlFunction for LineReflectPointFunction {
    fn signature(&self) -> FunctionSignature {
        FunctionSignature {
            name: "LINE_REFLECT_POINT",
            category: FunctionCategory::Mathematical,
            arg_count: ArgCount::Fixed(3),
            description: "Reflect a point across a line",
            returns: "Vector",
            examples: vec![
                "SELECT LINE_REFLECT_POINT(VEC(2,2), VEC(0,0), VEC(1,0))",
                "SELECT LINE_REFLECT_POINT(point, line_start, line_direction)",
            ],
        }
    }

    fn evaluate(&self, args: &[DataValue]) -> Result<DataValue> {
        self.validate_args(args)?;

        let point = get_vector(&args[0])?;
        let line_point = get_vector(&args[1])?;
        let line_dir = get_vector(&args[2])?;

        if point.len() != line_point.len() || point.len() != line_dir.len() {
            return Err(anyhow!("All vectors must have same dimension"));
        }

        // Find closest point on line (projection)
        let to_point: Vec<f64> = point
            .iter()
            .zip(line_point.iter())
            .map(|(p, lp)| p - lp)
            .collect();

        let dot_product: f64 = to_point
            .iter()
            .zip(line_dir.iter())
            .map(|(a, b)| a * b)
            .sum();
        let dir_mag_sq: f64 = line_dir.iter().map(|x| x * x).sum();

        if dir_mag_sq < 1e-10 {
            return Err(anyhow!("Line direction vector cannot be zero"));
        }

        let t = dot_product / dir_mag_sq;

        let closest: Vec<f64> = line_point
            .iter()
            .zip(line_dir.iter())
            .map(|(lp, ld)| lp + t * ld)
            .collect();

        // Reflection = point + 2 * (closest - point) = 2 * closest - point
        let reflected: Vec<f64> = closest
            .iter()
            .zip(point.iter())
            .map(|(c, p)| 2.0 * c - p)
            .collect();

        Ok(DataValue::Vector(reflected))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vector_string() {
        assert_eq!(parse_vector_string("[1,2,3]").unwrap(), vec![1.0, 2.0, 3.0]);
        assert_eq!(parse_vector_string("1 2 3").unwrap(), vec![1.0, 2.0, 3.0]);
        assert_eq!(
            parse_vector_string("1.5, 2.5, 3.5").unwrap(),
            vec![1.5, 2.5, 3.5]
        );
    }

    #[test]
    fn test_vec_function() {
        let func = VecFunction;
        let args = vec![
            DataValue::Integer(1),
            DataValue::Integer(2),
            DataValue::Integer(3),
        ];
        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::Vector(vec![1.0, 2.0, 3.0]));
    }

    #[test]
    fn test_vec_add() {
        let func = VecAddFunction;
        let args = vec![
            DataValue::Vector(vec![1.0, 2.0, 3.0]),
            DataValue::Vector(vec![4.0, 5.0, 6.0]),
        ];
        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::Vector(vec![5.0, 7.0, 9.0]));
    }

    #[test]
    fn test_vec_mag() {
        let func = VecMagFunction;
        let args = vec![DataValue::Vector(vec![3.0, 4.0])];
        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::Float(5.0));
    }

    #[test]
    fn test_vec_dot() {
        let func = VecDotFunction;
        let args = vec![
            DataValue::Vector(vec![1.0, 2.0, 3.0]),
            DataValue::Vector(vec![4.0, 5.0, 6.0]),
        ];
        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::Float(32.0)); // 1*4 + 2*5 + 3*6 = 32
    }

    #[test]
    fn test_vec_cross() {
        let func = VecCrossFunction;
        let args = vec![
            DataValue::Vector(vec![1.0, 0.0, 0.0]),
            DataValue::Vector(vec![0.0, 1.0, 0.0]),
        ];
        let result = func.evaluate(&args).unwrap();
        assert_eq!(result, DataValue::Vector(vec![0.0, 0.0, 1.0]));
    }
}
