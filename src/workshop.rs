use crate::error::{Error, Result};

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use log::{debug, info, warn};

use serde::Deserialize;

use bzip2::read::BzDecoder;
use zip::ZipArchive;

#[derive(Clone, Debug, Deserialize)]
struct SteamRemoteStorageResponse {
    pub response: FileResponse,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
struct FileResponse {
    pub result: i32,
    pub resultcount: i32,
    pub publishedfiledetails: Vec<FileDetails>,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
struct FileDetails {
    pub publishedfileid: String,
    pub result: i32,
    pub creator: String,
    pub creator_app_id: i32,
    pub consumer_app_id: i32,
    pub filename: String,
    pub file_size: usize,
    pub file_url: String,
    pub hcontent_file: String,
    pub preview_url: String,
    pub hcontent_preview: String,
    pub title: String,
    pub description: String,
    pub time_created: u64,
    pub time_updated: u64,
    pub visibility: i32,
    pub banned: i32,
    pub ban_reason: String,
    pub subscriptions: u64,
    pub favorited: u64,
    pub lifetime_subscriptions: u64,
    pub lifetime_favorited: u64,
    pub views: u64,
    pub tags: Vec<FileTag>,
}

#[allow(unused)]
#[derive(Clone, Debug, Deserialize)]
struct FileTag {
    pub tag: String,
}

fn download_workshop(id: &str, use_full_path: bool) -> Result<()> {
    if id == "" {
        return Err(Error::new("no workshop id supplied"));
    }

    info!("downloading file from steam workshop with id={}", id);

    // create download folder
    let file_dest_folder = Path::new(".").join("maps").join("workshop").join(id);
    fs::create_dir_all(file_dest_folder.clone())?;

    // fetch remote storage reply
    let mut params = HashMap::new();
    params.insert("itemcount", "1");
    params.insert("format", "json");
    params.insert("publishedfileids[0]", id);

    let client = reqwest::blocking::Client::new();
    let file_info_resp: SteamRemoteStorageResponse = client
        .post("http://api.steampowered.com/ISteamRemoteStorage/GetPublishedFileDetails/v0001")
        .form(&params)
        .send()
        .map_err(|_| Error::new("http request send error"))?
        .json()
        .map_err(|_| Error::new("json parse error"))?;

    if file_info_resp.response.publishedfiledetails.is_empty() {
        return Err(Error::new("published file details not found"));
    }

    debug!("file_info_resp: {:?}", file_info_resp);

    // download zip file
    info!(
        "downloading file: {}",
        file_info_resp.response.publishedfiledetails[0].file_url
    );
    let mut zip_file_resp = client
        .get(&file_info_resp.response.publishedfiledetails[0].file_url)
        .send()
        .map_err(|_| Error::new("http error"))?;

    // write to buffer
    let mut zip_buffer = Vec::new();
    {
        let mut zip_writer = BufWriter::new(&mut zip_buffer);
        zip_file_resp
            .copy_to(&mut zip_writer)
            .map_err(|_| Error::new("zip archive write error"))?;
    }

    // unpack zip
    {
        let zip_cursor = std::io::Cursor::new(&zip_buffer[..]);
        let mut zip_archive =
            ZipArchive::new(zip_cursor).map_err(|_| Error::new("zip archive error"))?;
        for i in 0..zip_archive.len() {
            let mut file = zip_archive
                .by_index(i)
                .map_err(|_| Error::new("zip archive error"))?;
            let outpath = if use_full_path {
                // create a ./workshop/{id}/mapname.bsp file
                file_dest_folder.join(file.mangled_name())
            } else {
                // create a ./mapname.bsp file
                Path::new(".").join("maps").join(file.mangled_name())
            };

            info!(
                "extracing workshop file with id={} to file={:?}",
                id, outpath
            );

            let mut outfile = File::create(&outpath).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    }

    Ok(())
}

fn download_fastdl(map_name: &str, fast_dl: &str) -> Result<()> {
    if map_name == "" {
        return Err(Error::new("no map name supplied"));
    }

    if fast_dl == "" {
        return Err(Error::new("no fastdl server configured"));
    }

    info!(
        "downloading file from fastdl={} with name={}",
        map_name, fast_dl
    );

    // create download folder
    let file_dest_folder = Path::new(".").join("maps");
    fs::create_dir_all(file_dest_folder.clone())?;

    // create bz2 request
    let client = reqwest::blocking::Client::new();

    let url_bz2 = fast_dl.to_string() + "/maps/" + map_name + ".bsp.bz2";
    let mut bz2_file_resp = client
        .get(&url_bz2)
        .send()
        .map_err(|_| Error::new("http request failed"))?;

    if bz2_file_resp.status() == reqwest::StatusCode::OK {
        // write bz2 to buffer
        let mut bz2_buffer = Vec::new();
        {
            let mut bz2_writer = BufWriter::new(&mut bz2_buffer);
            bz2_file_resp.copy_to(&mut bz2_writer).unwrap();
        }

        // unpack bz2
        {
            let bz2_reader = BufReader::new(&bz2_buffer[..]);
            let mut bz2_decoder = BzDecoder::new(bz2_reader);

            let mut bsp_buffer = Vec::new();
            bz2_decoder.read_to_end(&mut bsp_buffer)?;

            let mut file_dest_unpacked =
                File::create(file_dest_folder.join(map_name.to_string() + ".bsp"))?;
            file_dest_unpacked.write_all(&bsp_buffer)?;
        }

        return Ok(());
    }

    // try downloading raw bsp
    let url_bsp = fast_dl.to_string() + "/maps/" + map_name + ".bsp";
    let mut file_bsp = client
        .get(&url_bsp)
        .send()
        .map_err(|_| Error::new("http request failed"))?;

    if file_bsp.status() != reqwest::StatusCode::OK {
        return Err(Error::new("unable to download file from fastdl server"));
    }

    // download file
    let file_dest_path = file_dest_folder.join(map_name.to_string() + ".bsp");

    let mut file_dest = File::create(file_dest_path.clone())?;
    file_bsp.copy_to(&mut file_dest).unwrap();

    Ok(())
}

/*
pub fn try_download_default_map(map_name: &str) -> Result<()> {
    match map_name {
        "de_overpass" => download_workshop("205240106", false),
        "de_cbble" => download_workshop("205239595", false),
        "de_cache" => download_workshop("1855851320", false),
        "de_mirage" => download_workshop("152508932", false),
        "cs_militia" => download_workshop("133256570", false),
        "de_inferno_se" => download_workshop("125499116", false),
        "de_dust_se" => download_workshop("125498851", false),
        "de_nuke" => download_workshop("125439125", false),
        "de_vertigo" => download_workshop("125439851", false),
        "de_inferno" => download_workshop("125438669", false),
        "de_train" => download_workshop("125438372", false),
        "de_dust2" => download_workshop("125438255", false),
        "de_dust" => download_workshop("125438157", false),
        "de_aztec" => download_workshop("125438072", false),
        "cs_italy" => download_workshop("125436057", false),
        "cs_assault" => download_workshop("125432575", false),
        "cs_office" => download_workshop("125444404", false),
        "de_shortdust" => download_workshop("344476023", false),
        "de_shorttrain" => download_workshop("125439738", false),
        _ => Err(Error::new("map is not a default map")),
    }
}
*/

pub fn map_path(map_name: &str) -> PathBuf {
    Path::new(".")
        .join("maps")
        .join(map_name.to_string() + ".bsp")
}

pub fn map_exists(map_name: &str) -> bool {
    if map_name == "" {
        return false;
    }
    return Path::new(&map_path(map_name)).exists();
}

pub fn download(map_name: &str, fast_dl: &str) -> Result<()> {
    if map_name == "" {
        return Err(Error::new("invalid map name"));
    }

    /*if try_download_default_map(map_name).is_ok() {
        Ok(())
    } else*/
    if map_name.contains("workshop/") {
        let map_name_split = map_name.split("/").collect::<Vec<_>>();
        if map_name_split.len() != 3 {
            warn!("unable to parse workshop map name: {}", map_name);
            return Err(Error::new("unable to parse map name"));
        }
        info!("downloading workshop map: {}", map_name_split[1]);
        download_workshop(map_name_split[1], true)
    } else if fast_dl != "" {
        info!("downloading map via fastdl: {} -> {}", fast_dl, map_name);
        download_fastdl(map_name, fast_dl)
    } else {
        debug!("no map download path found for map {}", map_name);
        Err(Error::new("backup not implemented"))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn test_workshop() {
        download_workshop("1706490417", true).unwrap();
        //download_workshop("891765724").unwrap();
        //download_fastdl("am_cubes_fastgg", "http://srv2.fastgg.dk/csgo").unwrap();
    }
}
