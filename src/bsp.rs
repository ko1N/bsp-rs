mod native;
mod polygon;

use native::*;
use polygon::*;

use crate::error::*;

use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::mem::{size_of, MaybeUninit};
use std::path::Path;

use dataview::Pod;

fn parse_lump_data<T: Pod + Clone>(
    file: &mut File,
    header: &dheader_t,
    lump: LumpIndex,
) -> Result<Vec<T>> {
    let lump = &header.lumps[lump as usize];
    let lump_size = lump.filelen / size_of::<T>() as i32;
    if lump_size == 0 {
        return Err(Error::new("invalid lump data"));
    }

    let mut out: Vec<T> = vec![unsafe { MaybeUninit::uninit().assume_init() }; lump_size as usize];
    file.seek(SeekFrom::Start(lump.fileofs as u64))?;
    file.read_exact(out.as_bytes_mut())?;
    Ok(out)
}

#[allow(dead_code)]
pub struct BSP {
    vertexes: Vec<mvertex_t>,
    dplanes: Vec<dplane_t>,
    planes: Vec<cplane_t>,
    edges: Vec<dedge_t>,
    surf_edges: Vec<i32>,
    leaves: Vec<dleaf_t>,
    dnodes: Vec<dnode_t>,
    nodes: Vec<snode_t>,
    faces: Vec<dface_t>,
    tex_info: Vec<texinfo_t>,
    brushes: Vec<dbrush_t>,
    brush_sides: Vec<dbrushside_t>,
    leaf_faces: Vec<u16>,
    leaf_brushes: Vec<u16>,
    polys: Vec<Polygon>,
}

impl BSP {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = OpenOptions::new().read(true).write(false).open(path)?;

        let mut header = dheader_t::uninit();
        file.read_exact(header.as_bytes_mut())?;

        if header.ident != HEADER_MAGIC {
            return Err(Error::new(format!(
                "invalid bsp magic: 0x{:x}",
                header.ident
            )));
        }

        if header.version < 19 {
            return Err(Error::new(format!(
                "invalid bsp file version: {}",
                header.version
            )));
        }

        let vertexes: Vec<mvertex_t> = parse_lump_data(&mut file, &header, LumpIndex::Vertexes)?;
        let dplanes: Vec<dplane_t> = parse_lump_data(&mut file, &header, LumpIndex::Planes)?;
        let planes = parse_planes(&dplanes)?;

        let edges: Vec<dedge_t> = parse_lump_data(&mut file, &header, LumpIndex::Edges)?;
        let surf_edges: Vec<i32> = parse_lump_data(&mut file, &header, LumpIndex::SurfEdges)?;
        let leaves: Vec<dleaf_t> = parse_lump_data(&mut file, &header, LumpIndex::Leafs)?;
        let dnodes: Vec<dnode_t> = parse_lump_data(&mut file, &header, LumpIndex::Nodes)?;
        let nodes = parse_nodes(&dnodes)?;

        let faces: Vec<dface_t> = parse_lump_data(&mut file, &header, LumpIndex::Faces)?;
        let tex_info: Vec<texinfo_t> = parse_lump_data(&mut file, &header, LumpIndex::TexInfo)?;
        let brushes: Vec<dbrush_t> = parse_lump_data(&mut file, &header, LumpIndex::Brushes)?;
        let brush_sides: Vec<dbrushside_t> =
            parse_lump_data(&mut file, &header, LumpIndex::BrushSides)?;

        let leaf_faces: Vec<u16> = parse_lump_data(&mut file, &header, LumpIndex::LeafFaces)?;
        if leaf_faces.len() > MAX_MAP_LEAFBRUSHES {
            return Err(Error::new("map has to many leaf_faces"));
        } else if leaf_faces.is_empty() {
            return Err(Error::new("map has no many leaf_faces"));
        }

        let leaf_brushes: Vec<u16> = parse_lump_data(&mut file, &header, LumpIndex::LeafBrushes)?;
        if leaf_brushes.len() > MAX_MAP_LEAFBRUSHES {
            return Err(Error::new("map has to many leaf_brushes"));
        } else if leaf_faces.is_empty() {
            return Err(Error::new("map has no many leaf_brushes"));
        }

        let polys = parse_polygons(&faces, &surf_edges, &edges, &vertexes, &planes)?;

        Ok(Self {
            vertexes,
            dplanes,
            planes,
            edges,
            surf_edges,
            leaves,
            dnodes,
            nodes,
            faces,
            tex_info,
            brushes,
            brush_sides,
            leaf_faces,
            leaf_brushes,
            polys,
        })
    }
}

fn parse_planes(dplanes: &Vec<dplane_t>) -> Result<Vec<cplane_t>> {
    let mut cplanes: Vec<cplane_t> = Vec::new();

    dplanes.iter().for_each(|p| {
        let mut plane_bits = 0i32;
        for i in 0..p.normal.len() {
            if p.normal[i] < 0f32 {
                plane_bits |= 1 << i as i32;
            }
        }
        cplanes.push(cplane_t {
            normal: [p.normal[0], p.normal[1], p.normal[2]],
            distance: p.distance,
            typ: p.typ as u8,
            sign_bits: plane_bits as u8,
            pad0: [0, 0],
        });
    });

    Ok(cplanes)
}

fn parse_nodes(dnodes: &Vec<dnode_t>) -> Result<Vec<snode_t>> {
    let mut snodes: Vec<snode_t> = Vec::new();

    dnodes.iter().for_each(|n| {
        let mut sn = snode_t {
            mins: n.mins,
            maxs: n.maxs,
            plane_num: n.plane_num,
            plane_idx: n.plane_num as u32, // cplane_t*
            first_face: n.first_face,
            num_faces: n.num_faces,
            children: [n.children[0], n.children[1]],
            leaf_children_idx: 0, // dleaf_t*
            node_children_idx: 0, // snode_t*
            area: n.area,
            pad0: [0u8, 0],
        };

        for i in 0..2 {
            if n.children[i] >= 0 {
                sn.leaf_children_idx = 0;
                sn.node_children_idx = n.children[i] as u32;
            } else {
                sn.leaf_children_idx = (-1 - n.children[i]) as u32;
                sn.node_children_idx = 0;
            }
        }

        snodes.push(sn);
    });

    Ok(snodes)
}

fn parse_polygons(
    faces: &Vec<dface_t>,
    surf_edges: &Vec<i32>,
    edges: &Vec<dedge_t>,
    vertexes: &Vec<mvertex_t>,
    planes: &Vec<cplane_t>,
) -> Result<Vec<Polygon>> {
    let mut polys: Vec<Polygon> = Vec::new();

    for f in faces.iter() {
        if f.num_edges < 3 || f.num_edges > MAX_SURFINFO_VERTS as i16 {
            continue;
        }

        if f.tex_info <= 0 {
            continue;
        }

        let mut verts = [[0f32; 3]; MAX_SURFINFO_VERTS];
        for i in 0..(f.num_edges as i32) {
            let edge_idx = surf_edges[(f.first_edge + i) as usize];
            if edge_idx >= 0 {
                verts[i as usize] = vertexes[edges[edge_idx as usize].v[0] as usize].position;
            } else {
                verts[i as usize] = vertexes[edges[-edge_idx as usize].v[1] as usize].position;
            }
        }
        polys.push(Polygon::with(verts, f, &planes[f.plane_num as usize]));
    }

    Ok(polys)
}
