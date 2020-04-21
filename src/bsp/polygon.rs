use super::native::*;

pub const MAX_SURFINFO_VERTS: usize = 32;

#[derive(Clone, Debug, Copy)]
pub struct Polygon {
    verts: [[f32; 3]; MAX_SURFINFO_VERTS],
    vert_num: usize,
    plane: Plane,
    edge_planes: [Plane; MAX_SURFINFO_VERTS],
    vec_2d: [[f32; 3]; MAX_SURFINFO_VERTS],
    skip: i32,
}

impl Polygon {
    pub fn with(
        verts: [[f32; 3]; MAX_SURFINFO_VERTS],
        surface: &dface_t,
        plane: &cplane_t,
    ) -> Self {
        Self {
            verts,
            vert_num: surface.num_edges as usize,
            plane: Plane::from(plane),
            edge_planes: [Plane::new(); MAX_SURFINFO_VERTS],
            vec_2d: [[0f32; 3]; MAX_SURFINFO_VERTS],
            skip: 0,
        }
    }
}

#[derive(Clone, Debug, Copy)]
pub struct Plane {
    pub origin: [f32; 3],
    pub distance: f32,
}

impl Plane {
    pub fn new() -> Self {
        Self {
            origin: [0f32; 3],
            distance: 0f32,
        }
    }

    // TODO: from trait
    pub fn from(plane: &cplane_t) -> Self {
        Self {
            origin: [plane.normal[0], plane.normal[1], plane.normal[2]],
            distance: plane.distance,
        }
    }
}
