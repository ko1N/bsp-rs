pub fn dot_product(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

pub fn normalize(a: [f32; 3]) -> [f32; 3] {
    let len = dot_product(a, a);
    [a[0] / len, a[1] / len, a[2] / len]
}
