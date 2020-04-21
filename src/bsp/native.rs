#![allow(dead_code)]

use std::mem::MaybeUninit;

use dataview::Pod;

pub const HEADER_MAGIC: i32 = 0x50534256; // 'VBSP'
pub const HEADER_LUMP_COUNT: usize = 64;

pub const MAX_MAP_LEAFBRUSHES: usize = 65536;

pub enum LumpIndex {
    Entities = 0,
    Planes = 1,
    TexData = 2,
    Vertexes = 3,
    Visibility = 4,
    Nodes = 5,
    TexInfo = 6,
    Faces = 7,
    Lighting = 8,
    Occlusion = 9,
    Leafs = 10,
    Edges = 12,
    SurfEdges = 13,
    Models = 14,
    WorldLights = 15,
    LeafFaces = 16,
    LeafBrushes = 17,
    Brushes = 18,
    BrushSides = 19,
    Areas = 20,
    AreaPortals = 21,
    Portals = 22,
    Clusters = 23,
    PortalVerts = 24,
    ClusterPortals = 25,
    DispInfo = 26,
    OriginalFaces = 27,
    PhysCollide = 29,
    VertNormals = 30,
    VertNormalIndices = 31,
    DispLightmapAlphas = 32,
    DispVerts = 33,
    DispLightmapSamplePositions = 34,
    GameLump = 35,
    LeafWaterData = 36,
    Primitives = 37,
    PrimVerts = 38,
    PrimIndices = 39,
    PakFile = 40,
    ClipPortalVerts = 41,
    CubeMaps = 42,
    TexDataStringData = 43,
    TexDataStringTable = 44,
    Overlays = 45,
    LeafMinDistToWater = 46,
    FaceMacroTextureInfo = 47,
    DispTris = 48,
}

#[repr(C)]
#[derive(Clone, Pod)]
pub struct dheader_t {
    pub ident: i32,                         // 0x000
    pub version: i32,                       // 0x004
    pub lumps: [lump_t; HEADER_LUMP_COUNT], // 0x008
    pub map_revision: i32,                  // 0x408
} //Size=0x40C

impl dheader_t {
    pub fn uninit() -> Self {
        unsafe { MaybeUninit::uninit().assume_init() }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Pod)]
pub struct lump_t {
    pub fileofs: i32,     // 0x0
    pub filelen: i32,     // 0x4
    pub version: i32,     // 0x8
    pub four_cc: [u8; 4], // 0xC
} //Size=0x10

#[repr(C)]
#[derive(Clone, Debug, Pod)]
pub struct mvertex_t {
    pub position: [f32; 3], // 0x0
} //Size=0xC

#[repr(C)]
#[derive(Clone, Debug, Pod)]
pub struct dplane_t {
    pub normal: [f32; 3], // 0x00
    pub distance: f32,    // 0x0C
    pub typ: i32,         // 0x10
} //Size=0x14

#[repr(C)]
#[derive(Clone, Debug, Pod)]
pub struct cplane_t {
    pub normal: [f32; 3], // 0x00
    pub distance: f32,    // 0x0C
    pub typ: u8,          // 0x10
    pub sign_bits: u8,    // 0x11
    pub pad0: [u8; 2],    // 0x12
} //Size=0x14

#[repr(C)]
#[derive(Clone, Debug, Pod)]
pub struct dedge_t {
    pub v: [u16; 2], // 0x0
} //Size=0x4

#[repr(C)]
#[derive(Clone, Debug, Pod)]
pub struct dleaf_t {
    pub contents: i32,           // 0x00
    pub cluster: i16,            // 0x04
    pub area: i16,               // 0x06 - default: 9
    pub flags: i16,              // 0x11 - default: 7
    pub mins: [i16; 3],          // 0x1A
    pub maxs: [i16; 3],          // 0x20
    pub first_leaf_face: u16,    // 0x26
    pub num_leaf_faces: u16,     // 0x28
    pub first_leaf_brush: u16,   // 0x2A
    pub num_leaf_brushes: u16,   // 0x2C
    pub feaf_water_data_id: i16, // 0x2E
} //Size=0x30

#[repr(C)]
#[derive(Clone, Debug, Pod)]
pub struct dnode_t {
    pub plane_num: i32,     // 0x00
    pub children: [i32; 2], // 0x04
    pub mins: [i16; 3],     // 0x0C
    pub maxs: [i16; 3],     // 0x12
    pub first_face: u16,    // 0x18
    pub num_faces: u16,     // 0x1A
    pub area: i16,          // 0x1C
    pub pad0: [u8; 2],      // 0x1E
} //Size=0x20

#[repr(C)]
#[derive(Clone, Debug, Pod)]
pub struct snode_t {
    pub plane_num: i32,         // 0x00
    pub plane_idx: u32,         // 0x04 - cplane_t*
    pub children: [i32; 2],     // 0x08
    pub leaf_children_idx: u32, // 0x10 - dleaf_t*
    pub node_children_idx: u32, // 0x14 - snode_t*
    pub mins: [i16; 3],         // 0x18
    pub maxs: [i16; 3],         // 0x1E
    pub first_face: u16,        // 0x24
    pub num_faces: u16,         // 0x26
    pub area: i16,              // 0x28
    pub pad0: [u8; 2],          // 0x2A
} //Size=0x2C

#[repr(C)]
#[derive(Clone, Debug, Pod)]
pub struct dface_t {
    pub plane_num: u16,                            // 0x00
    pub side: u8,                                  // 0x02
    pub on_node: u8,                               // 0x03
    pub first_edge: i32,                           // 0x04
    pub num_edges: i16,                            // 0x08
    pub tex_info: i16,                             // 0x0A
    pub disp_info: i16,                            // 0x0C
    pub surface_fog_volume_id: i16,                // 0x0E
    pub styles: [u8; 4],                           // 0x10
    pub light_ofs: i32,                            // 0x18
    pub area: f32,                                 // 0x1C
    pub lightmap_texture_mins_in_luxels: [i32; 2], // 0x20
    pub lightmap_texture_size_in_luxels: [i32; 2], // 0x28
    pub orig_face: i32,                            // 0x30
    pub num_prims: u16,                            // 0x34
    pub first_prim_id: u16,                        // 0x36
    pub smoothing_groups: u16,                     // 0x38
    pub pad0: [u8; 2],                             // 0x2A
} //Size=0x3A

#[repr(C)]
#[derive(Clone, Debug, Pod)]
pub struct texinfo_t {
    pub texture_vecs: [[f32; 4]; 2],  // 0x00
    pub lightmap_vecs: [[f32; 4]; 2], // 0x20
    pub flags: i32,                   // 0x40
    pub tex_data: i32,                // 0x44
} //Size=0x48

#[repr(C)]
#[derive(Clone, Debug, Pod)]
pub struct dbrush_t {
    pub first_side: i32, // 0x0
    pub num_sides: i32,  // 0x4
    pub contents: i32,   // 0x8
} //Size=0xC

#[repr(C)]
#[derive(Clone, Debug, Pod)]
pub struct dbrushside_t {
    pub plane_num: u16, // 0x0
    pub tex_info: i16,  // 0x2
    pub disp_info: i16, // 0x4
    pub bevel: u8,      // 0x6
    pub thin: u8,       // 0x7
} //Size=0x8
