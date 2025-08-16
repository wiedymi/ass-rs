//! Math utilities for transformations and interpolation

/// 3x3 transformation matrix
#[derive(Debug, Clone, Copy)]
pub struct Matrix3x3 {
    /// Matrix elements in row-major order
    pub m: [[f32; 3]; 3],
}

impl Matrix3x3 {
    /// Create identity matrix
    pub fn identity() -> Self {
        Self {
            m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    /// Create translation matrix
    pub fn translate(x: f32, y: f32) -> Self {
        Self {
            m: [[1.0, 0.0, x], [0.0, 1.0, y], [0.0, 0.0, 1.0]],
        }
    }

    /// Create scale matrix
    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            m: [[sx, 0.0, 0.0], [0.0, sy, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    /// Create rotation matrix (angle in radians)
    pub fn rotate(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            m: [[cos, -sin, 0.0], [sin, cos, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    /// Multiply two matrices
    pub fn multiply(&self, other: &Self) -> Self {
        let mut result = Self::identity();
        for i in 0..3 {
            for j in 0..3 {
                result.m[i][j] = 0.0;
                for k in 0..3 {
                    result.m[i][j] += self.m[i][k] * other.m[k][j];
                }
            }
        }
        result
    }

    /// Transform a point
    pub fn transform_point(&self, x: f32, y: f32) -> (f32, f32) {
        let tx = self.m[0][0] * x + self.m[0][1] * y + self.m[0][2];
        let ty = self.m[1][0] * x + self.m[1][1] * y + self.m[1][2];
        (tx, ty)
    }
}

impl Default for Matrix3x3 {
    fn default() -> Self {
        Self::identity()
    }
}

/// 2D transformation
#[derive(Debug, Clone, Copy)]
pub struct Transform2D {
    /// Translation X
    pub tx: f32,
    /// Translation Y
    pub ty: f32,
    /// Scale X
    pub sx: f32,
    /// Scale Y
    pub sy: f32,
    /// Rotation angle in radians
    pub rotation: f32,
    /// Shear X
    pub shear_x: f32,
    /// Shear Y
    pub shear_y: f32,
}

impl Transform2D {
    /// Create identity transform
    pub fn identity() -> Self {
        Self {
            tx: 0.0,
            ty: 0.0,
            sx: 1.0,
            sy: 1.0,
            rotation: 0.0,
            shear_x: 0.0,
            shear_y: 0.0,
        }
    }

    /// Convert to matrix
    pub fn to_matrix(&self) -> Matrix3x3 {
        let translate = Matrix3x3::translate(self.tx, self.ty);
        let scale = Matrix3x3::scale(self.sx, self.sy);
        let rotate = Matrix3x3::rotate(self.rotation);

        translate.multiply(&rotate).multiply(&scale)
    }

    /// Interpolate between two transforms
    pub fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            tx: lerp(self.tx, other.tx, t),
            ty: lerp(self.ty, other.ty, t),
            sx: lerp(self.sx, other.sx, t),
            sy: lerp(self.sy, other.sy, t),
            rotation: lerp(self.rotation, other.rotation, t),
            shear_x: lerp(self.shear_x, other.shear_x, t),
            shear_y: lerp(self.shear_y, other.shear_y, t),
        }
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}

/// Linear interpolation
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Cubic bezier interpolation (for animation curves)
pub fn cubic_bezier(t: f32, p1: f32, p2: f32) -> f32 {
    // Using ass-core's eval_cubic_bezier would be better here
    // This is a simplified version
    let t2 = t * t;
    let t3 = t2 * t;
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let mt3 = mt2 * mt;

    mt3 * 0.0 + 3.0 * mt2 * t * p1 + 3.0 * mt * t2 * p2 + t3 * 1.0
}
