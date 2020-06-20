use crate::bsp::*;

pub const CONTENTS_EMPTY: i32 = 0;
/// No contents
pub const CONTENTS_SOLID: i32 = 0x1;
/// an eye is never valid in a solid
pub const CONTENTS_WINDOW: i32 = 0x2;
/// translucent, but not watery (glass)
pub const CONTENTS_AUX: i32 = 0x4;
pub const CONTENTS_GRATE: i32 = 0x8;
/// alpha-tested "grate" textures.  Bullets/sight pass through, but solids don't
pub const CONTENTS_SLIME: i32 = 0x10;
pub const CONTENTS_WATER: i32 = 0x20;
pub const CONTENTS_MIST: i32 = 0x40;
pub const CONTENTS_OPAQUE: i32 = 0x80;
/// things that cannot be seen through (may be non-solid though)
pub const LAST_VISIBLE_CONTENTS: i32 = 0x80;
pub const ALL_VISIBLE_CONTENTS: i32 = LAST_VISIBLE_CONTENTS | LAST_VISIBLE_CONTENTS - 1;
pub const CONTENTS_TESTFOGVOLUME: i32 = 0x100;
pub const CONTENTS_UNUSED3: i32 = 0x200;
pub const CONTENTS_UNUSED4: i32 = 0x400;
pub const CONTENTS_UNUSED5: i32 = 0x800;
pub const CONTENTS_UNUSED6: i32 = 0x1000;
pub const CONTENTS_UNUSED7: i32 = 0x2000;
pub const CONTENTS_MOVEABLE: i32 = 0x4000;
/// hits entities which are MOVETYPE_PUSH (doors, plats, etc.)
// remaining contents are non-visible, and don't eat brushes
pub const CONTENTS_AREAPORTAL: i32 = 0x8000;
pub const CONTENTS_PLAYERCLIP: i32 = 0x10000;
pub const CONTENTS_MONSTERCLIP: i32 = 0x20000;
// currents can be added to any other contents, and may be mixed
pub const CONTENTS_CURRENT_0: i32 = 0x40000;
pub const CONTENTS_CURRENT_90: i32 = 0x80000;
pub const CONTENTS_CURRENT_180: i32 = 0x100000;
pub const CONTENTS_CURRENT_270: i32 = 0x200000;
pub const CONTENTS_CURRENT_UP: i32 = 0x400000;
pub const CONTENTS_CURRENT_DOWN: i32 = 0x800000;
pub const CONTENTS_ORIGIN: i32 = 0x1000000;
/// removed before bsping an entity
pub const CONTENTS_MONSTER: i32 = 0x2000000;
/// should never be on a brush, only in game
pub const CONTENTS_DEBRIS: i32 = 0x4000000;
pub const CONTENTS_DETAIL: i32 = 0x8000000;
/// brushes to be added after vis leafs
pub const CONTENTS_TRANSLUCENT: i32 = 0x10000000;
/// int32_t set if any surface has trans
pub const CONTENTS_LADDER: i32 = 0x20000000;
pub const CONTENTS_HITBOX: i32 = 0x40000000;
/// use accurate hitboxes on trace

pub const MASK_SHOT_HULL: i32 = CONTENTS_SOLID
    | CONTENTS_MOVEABLE
    | CONTENTS_MONSTER
    | CONTENTS_WINDOW
    | CONTENTS_DEBRIS
    | CONTENTS_GRATE;

pub const DIST_EPSILON: f32 = 0.03125f32;

pub struct Trace {
    pub all_solid: bool,
    pub start_solid: bool,
    pub fraction: f32,
    pub fraction_left_solid: f32,
    pub end_pos: [f32; 3],
    pub plane: Option<cplane_t>, // BSP::cplane_t*
    pub contents: i32,
    pub brush: Option<dbrush_t>, // BSP::dbrush_t*
    pub brush_side: i32,
}

impl Trace {
    pub fn new() -> Self {
        Self {
            all_solid: true,
            start_solid: true,
            fraction: 1f32,
            fraction_left_solid: 1f32,
            end_pos: [0f32; 3],
            plane: None,
            contents: 0,
            brush: None,
            brush_side: 0,
        }
    }
}

pub fn is_visible(bsp: &BSP, from: [f32; 3], to: [f32; 3]) -> bool {
    let mut trace = Trace::new();
    ray_cast(bsp, from, to, &mut trace);

    !(trace.fraction < 1f32)
}

pub fn ray_cast(bsp: &BSP, from: [f32; 3], to: [f32; 3], trace: &mut Trace) {
    if bsp.planes.is_empty() {
        return;
    }

    trace.all_solid = false;
    trace.start_solid = false;
    trace.fraction = 1f32;
    trace.fraction_left_solid = 0f32;

    ray_cast_node(bsp, from, to, 0, 0f32, 1f32, trace);

    if trace.fraction < 1f32 {
        for i in 0..3 {
            trace.end_pos[i] = from[i] + trace.fraction * (to[i] - from[i]);
        }
    } else {
        trace.end_pos = to;
    }
}

fn ray_cast_node(
    bsp: &BSP,
    from: [f32; 3],
    to: [f32; 3],
    node_idx: i32,
    start_fract: f32,
    end_fract: f32,
    trace: &mut Trace,
) {
    if trace.fraction <= start_fract {
        return;
    }

    if node_idx < 0 {
        let leaf = &bsp.leaves[(-node_idx - 1) as usize];
        for i in 0..(leaf.num_leaf_brushes) {
            let leaf_idx = (leaf.first_leaf_brush + i) as usize;
            if leaf_idx >= bsp.leaf_brushes.len() {
                println!("shouldnt happen?");
                continue;
            }
            let brush_idx = bsp.leaf_brushes[leaf_idx] as i32;

            if brush_idx as usize >= bsp.brushes.len() {
                continue;
            }
            let brush = &bsp.brushes[brush_idx as usize];
            if (brush.contents & MASK_SHOT_HULL) == 0 {
                continue;
            }

            ray_cast_brush(bsp, from, to, brush, trace);
            if trace.fraction == 0f32 {
                return;
            }
        }

        if trace.start_solid {
            return;
        }

        if trace.fraction < 1f32 {
            return;
        }

        for i in 0..(leaf.num_leaf_faces) {
            ray_cast_surface(
                bsp,
                from,
                to,
                bsp.leaf_faces[(leaf.first_leaf_face + i) as usize] as i32,
                trace,
            );
        }

        return;
    }

    // regular path
    if node_idx as usize >= bsp.nodes.len() {
        return;
    }
    let node = &bsp.nodes[node_idx as usize];

    if node.plane_idx as usize >= bsp.planes.len() {
        return;
    }
    let plane = &bsp.planes[node.plane_idx as usize];
    // check plane count

    let (start_dist, end_dist) = {
        if plane.typ < 3 {
            (
                from[plane.typ as usize] - plane.distance,
                to[plane.typ as usize] - plane.distance,
            )
        } else {
            (
                math::dot_product(from, plane.normal) - plane.distance,
                math::dot_product(to, plane.normal) - plane.distance,
            )
        }
    };

    if start_dist >= 0f32 && end_dist >= 0f32 {
        ray_cast_node(
            bsp,
            from,
            to,
            node.children[0],
            start_fract,
            end_fract,
            trace,
        );
    } else if start_dist < 0f32 && end_dist < 0f32 {
        ray_cast_node(
            bsp,
            from,
            to,
            node.children[1],
            start_fract,
            end_fract,
            trace,
        );
    } else {
        let mut side_id = 0i32;
        let mut fraction_first = 0f32;
        let mut fraction_second = 0f32;
        let mut fraction_middle = 0f32;
        let mut middle = [0f32; 3];

        if start_dist < end_dist {
            // Back
            side_id = 1;
            let inverse_dist = 1f32 / (start_dist - end_dist);

            fraction_first = (start_dist + std::f32::EPSILON) * inverse_dist;
            fraction_second = (start_dist + std::f32::EPSILON) * inverse_dist;
        } else if end_dist < start_dist {
            // Front
            side_id = 0;
            let inverse_dist = 1f32 / (start_dist - end_dist);

            fraction_first = (start_dist + std::f32::EPSILON) * inverse_dist;
            fraction_second = (start_dist - std::f32::EPSILON) * inverse_dist;
        } else {
            // Front
            side_id = 0;
            fraction_first = 1f32;
            fraction_second = 0f32;
        }

        if fraction_first < 0f32 {
            fraction_first = 0f32;
        } else if fraction_first > 1f32 {
            fraction_first = 1f32;
        }

        if fraction_second < 0f32 {
            fraction_second = 0f32;
        } else if fraction_second > 1f32 {
            fraction_second = 1f32;
        }

        fraction_middle = start_fract + (end_fract - start_fract) * fraction_first;
        for i in 0..3 {
            middle[i] = from[i] + fraction_first * (to[i] - from[i]);
        }

        ray_cast_node(
            bsp,
            from,
            middle,
            node.children[side_id as usize],
            start_fract,
            fraction_middle,
            trace,
        );
        fraction_middle = start_fract + (end_fract - start_fract) * fraction_second;
        for i in 0..3 {
            middle[i] = from[i] + fraction_second * (to[i] - from[i]);
        }

        ray_cast_node(
            bsp,
            middle,
            to,
            node.children[{
                if side_id > 0 {
                    0
                } else {
                    1
                }
            } as usize],
            fraction_middle,
            end_fract,
            trace,
        );
    }
}

fn ray_cast_brush(bsp: &BSP, from: [f32; 3], to: [f32; 3], brush: &dbrush_t, trace: &mut Trace) {
    if brush.num_sides == 0 {
        return;
    }

    let mut fraction_to_enter = -99f32;
    let mut fraction_to_leave = 1f32;
    let mut starts_out = false;
    let mut ends_out = false;

    for i in 0..(brush.num_sides) {
        let brush_side_idx = (brush.first_side + i) as usize;
        if brush_side_idx >= bsp.brush_sides.len() {
            continue;
        }
        let brush_side = &bsp.brush_sides[brush_side_idx];
        if brush_side.bevel != 0 {
            continue;
        }

        if brush_side.plane_num as usize >= bsp.planes.len() {
            continue;
        }
        let plane = &bsp.planes[brush_side.plane_num as usize];

        let start_dist = math::dot_product(from, plane.normal) - plane.distance;
        let end_dist = math::dot_product(to, plane.normal) - plane.distance;

        if start_dist > 0f32 {
            starts_out = true;
            if end_dist > 0f32 {
                return;
            }
        } else {
            if end_dist <= 0f32 {
                continue;
            }
            ends_out = true;
        }

        if start_dist > end_dist {
            let mut fraction = (start_dist - DIST_EPSILON).max(0f32);
            fraction = fraction / (start_dist - end_dist);
            if fraction > fraction_to_enter {
                fraction_to_enter = fraction;
            }
        } else {
            let fraction = (start_dist + DIST_EPSILON) / (start_dist - end_dist);
            if fraction < fraction_to_leave {
                fraction_to_leave = fraction;
            }
        }
    }

    if starts_out {
        if trace.fraction_left_solid - fraction_to_enter > 0f32 {
            starts_out = false;
        }
    }

    if !starts_out {
        trace.start_solid = true;
        trace.contents = brush.contents;

        if !ends_out {
            trace.all_solid = true;
            trace.fraction = 0f32;
            trace.fraction_left_solid = 1f32;
        } else {
            if fraction_to_leave != 1f32 && fraction_to_leave > trace.fraction_left_solid {
                trace.fraction_left_solid = fraction_to_leave;
                if trace.fraction <= fraction_to_leave {
                    trace.fraction = 1f32;
                }
            }
        }
        return;
    }

    if fraction_to_enter < fraction_to_leave {
        if fraction_to_enter > -99f32 && fraction_to_enter < trace.fraction {
            if fraction_to_enter < 0f32 {
                fraction_to_enter = 0f32;
            }

            trace.fraction = fraction_to_enter;
            trace.brush = Some(brush.clone());
            trace.contents = brush.contents;
        }
    }
}

fn ray_cast_surface(bsp: &BSP, from: [f32; 3], to: [f32; 3], surface_idx: i32, trace: &mut Trace) {
    if surface_idx as usize >= bsp.polys.len() {
        return;
    }
    let poly = &bsp.polys[surface_idx as usize];

    let plane = &poly.plane;
    let dot1 = math::dot_product(plane.origin, from) - plane.distance;
    let dot2 = math::dot_product(plane.origin, to) - plane.distance;

    if (dot1 > 0f32 && dot2 <= 0f32) || (dot1 <= 0f32 && dot2 > 0f32) {
        if dot1 - dot2 < DIST_EPSILON {
            return;
        }

        let t = dot1 / (dot1 - dot2);
        if t <= 0f32 {
            return;
        }

        let mut intersection = [0f32; 3];
        for i in 0..3 {
            intersection[i] = from[i] + (to[i] - from[i]) * t;
        }

        for i in 0..(poly.vert_num) {
            let edge_plane = &poly.edge_planes[i];
            if math::dot_product(edge_plane.origin, intersection) < 0f32 {
                return;
            }
        }

        trace.fraction = 0.2f32; // TODO: ?
        trace.end_pos = intersection;
    }
}
