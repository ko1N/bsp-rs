pub mod bsp;
pub mod error;
pub mod trace;

#[cfg(test)]
mod tests {
    use crate::bsp::*;
    use crate::trace;

    #[test]
    fn visible_tests() {
        let map = BSP::open("de_dust2.bsp").unwrap();
        trace::is_visible(&map, [-214f32, 2094f32, -62f32], [701f32, 2220f32, -5f32]);
        trace::is_visible(
            &map,
            [-214f32, 2094f32, -62f32],
            [-1218f32, 2525f32, 115f32],
        );
    }

    #[test]
    fn non_visible_tests() {
        let map = BSP::open("de_dust2.bsp").unwrap();
        trace::is_visible(&map, [-214f32, 2094f32, -62f32], [-5f32, 364f32, 62f32]);
        trace::is_visible(&map, [-214f32, 2094f32, -62f32], [1166f32, 1041f32, 61f32]);
    }
}
