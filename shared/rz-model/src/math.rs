//! Matrices 4×4 à la D3DX : **lignes**, vecteurs **lignes** (`p' = p · M`), translation dans la
//! dernière ligne (`m[3][0..3]`). `mul(a, b)` = appliquer `a` puis `b`.

pub type Mat4 = [[f32; 4]; 4];

pub const IDENTITY: Mat4 = [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0],
];

/// 16 flottants dans l'ordre du fichier (ligne par ligne).
pub fn from_slice16(s: &[f32]) -> Mat4 {
    let mut m = IDENTITY;
    if s.len() >= 16 {
        for (i, row) in m.iter_mut().enumerate() {
            row.copy_from_slice(&s[i * 4..i * 4 + 4]);
        }
    }
    m
}

/// 12 flottants : 4 lignes de 3 (rotation/échelle puis translation), colonne implicite (0,0,0,1).
pub fn from_slice12(s: &[f32]) -> Mat4 {
    let mut m = IDENTITY;
    if s.len() >= 12 {
        for (i, row) in m.iter_mut().enumerate() {
            row[..3].copy_from_slice(&s[i * 3..i * 3 + 3]);
        }
    }
    m
}

pub fn mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut r = [[0.0f32; 4]; 4];
    for i in 0..4 {
        for j in 0..4 {
            r[i][j] = (0..4).map(|k| a[i][k] * b[k][j]).sum();
        }
    }
    r
}

pub fn transform_point(p: [f32; 3], m: &Mat4) -> [f32; 3] {
    let mut r = [0.0; 3];
    for (j, v) in r.iter_mut().enumerate() {
        *v = p[0] * m[0][j] + p[1] * m[1][j] + p[2] * m[2][j] + m[3][j];
    }
    r
}

pub fn transform_vector(v: [f32; 3], m: &Mat4) -> [f32; 3] {
    let mut r = [0.0; 3];
    for (j, x) in r.iter_mut().enumerate() {
        *x = v[0] * m[0][j] + v[1] * m[1][j] + v[2] * m[2][j];
    }
    r
}

/// Inverse générale (cofacteurs) ; renvoie l'identité si la matrice est singulière.
pub fn inverse(m: &Mat4) -> Mat4 {
    let a: Vec<f32> = m.iter().flatten().copied().collect();
    let mut inv = [0.0f32; 16];
    inv[0] = a[5] * a[10] * a[15] - a[5] * a[11] * a[14] - a[9] * a[6] * a[15]
        + a[9] * a[7] * a[14]
        + a[13] * a[6] * a[11]
        - a[13] * a[7] * a[10];
    inv[4] = -a[4] * a[10] * a[15] + a[4] * a[11] * a[14] + a[8] * a[6] * a[15]
        - a[8] * a[7] * a[14]
        - a[12] * a[6] * a[11]
        + a[12] * a[7] * a[10];
    inv[8] = a[4] * a[9] * a[15] - a[4] * a[11] * a[13] - a[8] * a[5] * a[15]
        + a[8] * a[7] * a[13]
        + a[12] * a[5] * a[11]
        - a[12] * a[7] * a[9];
    inv[12] = -a[4] * a[9] * a[14] + a[4] * a[10] * a[13] + a[8] * a[5] * a[14]
        - a[8] * a[6] * a[13]
        - a[12] * a[5] * a[10]
        + a[12] * a[6] * a[9];
    inv[1] = -a[1] * a[10] * a[15] + a[1] * a[11] * a[14] + a[9] * a[2] * a[15]
        - a[9] * a[3] * a[14]
        - a[13] * a[2] * a[11]
        + a[13] * a[3] * a[10];
    inv[5] = a[0] * a[10] * a[15] - a[0] * a[11] * a[14] - a[8] * a[2] * a[15]
        + a[8] * a[3] * a[14]
        + a[12] * a[2] * a[11]
        - a[12] * a[3] * a[10];
    inv[9] = -a[0] * a[9] * a[15] + a[0] * a[11] * a[13] + a[8] * a[1] * a[15]
        - a[8] * a[3] * a[13]
        - a[12] * a[1] * a[11]
        + a[12] * a[3] * a[9];
    inv[13] = a[0] * a[9] * a[14] - a[0] * a[10] * a[13] - a[8] * a[1] * a[14]
        + a[8] * a[2] * a[13]
        + a[12] * a[1] * a[10]
        - a[12] * a[2] * a[9];
    inv[2] = a[1] * a[6] * a[15] - a[1] * a[7] * a[14] - a[5] * a[2] * a[15]
        + a[5] * a[3] * a[14]
        + a[13] * a[2] * a[7]
        - a[13] * a[3] * a[6];
    inv[6] = -a[0] * a[6] * a[15] + a[0] * a[7] * a[14] + a[4] * a[2] * a[15]
        - a[4] * a[3] * a[14]
        - a[12] * a[2] * a[7]
        + a[12] * a[3] * a[6];
    inv[10] = a[0] * a[5] * a[15] - a[0] * a[7] * a[13] - a[4] * a[1] * a[15]
        + a[4] * a[3] * a[13]
        + a[12] * a[1] * a[7]
        - a[12] * a[3] * a[5];
    inv[14] = -a[0] * a[5] * a[14] + a[0] * a[6] * a[13] + a[4] * a[1] * a[14]
        - a[4] * a[2] * a[13]
        - a[12] * a[1] * a[6]
        + a[12] * a[2] * a[5];
    inv[3] = -a[1] * a[6] * a[11] + a[1] * a[7] * a[10] + a[5] * a[2] * a[11]
        - a[5] * a[3] * a[10]
        - a[9] * a[2] * a[7]
        + a[9] * a[3] * a[6];
    inv[7] = a[0] * a[6] * a[11] - a[0] * a[7] * a[10] - a[4] * a[2] * a[11]
        + a[4] * a[3] * a[10]
        + a[8] * a[2] * a[7]
        - a[8] * a[3] * a[6];
    inv[11] = -a[0] * a[5] * a[11] + a[0] * a[7] * a[9] + a[4] * a[1] * a[11]
        - a[4] * a[3] * a[9]
        - a[8] * a[1] * a[7]
        + a[8] * a[3] * a[5];
    inv[15] = a[0] * a[5] * a[10] - a[0] * a[6] * a[9] - a[4] * a[1] * a[10]
        + a[4] * a[2] * a[9]
        + a[8] * a[1] * a[6]
        - a[8] * a[2] * a[5];
    let det = a[0] * inv[0] + a[1] * inv[4] + a[2] * inv[8] + a[3] * inv[12];
    if det.abs() < 1e-12 || !det.is_finite() {
        return IDENTITY;
    }
    from_slice16(&inv.map(|x| x / det))
}

/// Quaternion `(x, y, z, w)` (ordre du fichier, comme `D3DXQUATERNION`) et position → matrice
/// (`D3DXMatrixRotationQuaternion` puis translation).
pub fn from_quat_pos(q: [f32; 4], p: [f32; 3]) -> Mat4 {
    let [x, y, z, w] = normalize4(q);
    [
        [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y + z * w),
            2.0 * (x * z - y * w),
            0.0,
        ],
        [
            2.0 * (x * y - z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z + x * w),
            0.0,
        ],
        [
            2.0 * (x * z + y * w),
            2.0 * (y * z - x * w),
            1.0 - 2.0 * (x * x + y * y),
            0.0,
        ],
        [p[0], p[1], p[2], 1.0],
    ]
}

pub fn normalize4(q: [f32; 4]) -> [f32; 4] {
    let n = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
    if n > 1e-12 {
        q.map(|c| c / n)
    } else {
        [0.0, 0.0, 0.0, 1.0]
    }
}

pub fn lerp3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Interpolation sphérique (chemin le plus court).
pub fn slerp(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    let mut d = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let mut b = b;
    if d < 0.0 {
        d = -d;
        b = b.map(|c| -c);
    }
    let (k0, k1) = if d > 0.9995 {
        (1.0 - t, t)
    } else {
        let th = d.acos();
        let s = th.sin();
        (((1.0 - t) * th).sin() / s, (t * th).sin() / s)
    };
    normalize4([
        a[0] * k0 + b[0] * k1,
        a[1] * k0 + b[1] * k1,
        a[2] * k0 + b[2] * k1,
        a[3] * k0 + b[3] * k1,
    ])
}

/// Interpolation linéaire composante par composante (clés de nœud nx3).
pub fn lerp_mat(a: &Mat4, b: &Mat4, t: f32) -> Mat4 {
    let mut r = *a;
    for i in 0..4 {
        for j in 0..4 {
            r[i][j] = a[i][j] + (b[i][j] - a[i][j]) * t;
        }
    }
    r
}

pub fn is_identity(m: &Mat4, eps: f32) -> bool {
    (0..4).all(|i| (0..4).all(|j| (m[i][j] - if i == j { 1.0 } else { 0.0 }).abs() <= eps))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inverse_roundtrip() {
        let m = from_quat_pos([0.1, 0.2, -0.3, 0.9], [1.0, -2.0, 3.0]);
        let r = mul(&m, &inverse(&m));
        assert!(is_identity(&r, 1e-5));
        let p = transform_point([1.0, 2.0, 3.0], &m);
        let q = transform_point(p, &inverse(&m));
        assert!((q[0] - 1.0).abs() < 1e-5 && (q[2] - 3.0).abs() < 1e-5);
    }
}
