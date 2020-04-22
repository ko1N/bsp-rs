use super::math::*;
use super::native::*;

pub const MAX_SURFINFO_VERTS: usize = 32;

#[derive(Clone, Debug, Copy)]
pub struct Polygon {
    pub verts: [[f32; 3]; MAX_SURFINFO_VERTS],
    pub vert_num: usize,
    pub plane: Plane,
    pub edge_planes: [Plane; MAX_SURFINFO_VERTS],
    pub vec_2d: [[f32; 3]; MAX_SURFINFO_VERTS],
    pub skip: i32,
}

impl Polygon {
    pub fn with(
        verts: [[f32; 3]; MAX_SURFINFO_VERTS],
        surface: &dface_t,
        plane: &cplane_t,
    ) -> Self {
        let mut poly = Self {
            verts,
            vert_num: surface.num_edges as usize,
            plane: Plane::from(plane),
            edge_planes: [Plane::new(); MAX_SURFINFO_VERTS],
            vec_2d: [[0f32; 3]; MAX_SURFINFO_VERTS],
            skip: 0,
        };

        // pre-process polys
        for i in 0..(poly.vert_num) {
            let edge_plane = &mut poly.edge_planes[i];
            if edge_plane.origin[0] == 0f32
                && edge_plane.origin[1] == 0f32
                && edge_plane.origin[2] == 0f32
            {
                for j in 0..3 {
                    edge_plane.origin[j] = poly.plane.origin[j]
                        - (poly.verts[i][j] - poly.verts[(i + 1) % poly.vert_num][j]);
                }
                edge_plane.origin = normalize(edge_plane.origin);
                edge_plane.distance = dot_product(edge_plane.origin, poly.verts[i]);
            }
        }

        poly
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
