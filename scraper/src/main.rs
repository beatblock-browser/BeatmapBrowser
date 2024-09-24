use anyhow::Error;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use std::{env, fs};
use image::{ImageFormat, ImageReader};
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::sql::Operator::NoneInside;
use surrealdb::Surreal;
use unrar::Archive;
use urlencoding::decode;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Connect to the server
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;

    // Signin as a namespace, database, or root user
    db.signin(Root {
        username: "root",
        password: "root",
    })
        .await?;

    // Select a specific namespace / database
    db.use_ns("beatblock").use_db("beatblock").await?;

    let folder = env::args().nth(1).expect("No folder provided!");
    let regex = Regex::new("(<a href=[^>]+)+").unwrap();
    let mut missed = Vec::new();
    for level in fs::read_dir(folder)? {
        let level = level?;
        if level.file_type()?.is_dir() {
            continue;
        }

        let file = fs::read_to_string(level.path())?;
        let mut zips = Vec::new();
        for (_, zip_file) in regex.find_iter(&file).enumerate() {
            let zip_file = zip_file.as_str();
            if !zip_file.contains(".zip") && !zip_file.contains(".rar") {
                continue;
            }
            let zip_file = zip_file[9..zip_file.len() - 1].to_string();
            let zip_file = decode(&zip_file).unwrap().to_string().replace("&amp;", "&");
            zips.push(zip_file);
        }

        if zips.is_empty() {
            let path = level.path();
            missed.push(path.to_str().unwrap().to_string());
        } else {
            let last = zips.last().unwrap();
            let name = &last[79..];
            let name = name[0..name.find("?").unwrap()].to_string();
            println!("Got {}, {}", last, name);
            let mut path = env::current_dir().unwrap().join("../output").join(name.clone());
            fs::write(path.clone(), reqwest::get(last).await?.bytes().await.unwrap()).unwrap();
            let (data, image, download) = if name.contains(".zip") {
                match read_zip(path.clone()) {
                    Ok(result) => result,
                    Err(error) => {
                        println!("Error reading zip: {:?}", error);
                        continue;
                    }
                }
            } else {
                match read_rar(path.clone()) {
                    Ok(result) => result,
                    Err(error) => {
                        println!("Error reading rar: {:?}", error);
                        continue;
                    }
                }
            };
            fs::remove_file(path.clone());
            if path.to_str().unwrap().ends_with(".rar") {
                path = PathBuf::from(path.to_str().unwrap().replace(".rar", ".zip"));
            }
            let image = match image {
                Some(image) => {
                    if image.is_empty() {
                        None
                    } else {
                        let format = if String::from_utf8_lossy(&image[0..4]) == "�PNG" {
                            ImageFormat::Png
                        } else {
                            ImageFormat::Jpeg
                        };
                        let image_path = path.to_str().unwrap().replace(".zip", ".png");
                        match ImageReader::with_format(Cursor::new(image), format).decode() {
                            Ok(image) => {
                                image.save_with_format(PathBuf::from(image_path.clone()), ImageFormat::Png).unwrap();
                                fs::write(path.clone(), download).unwrap();
                                Some(image_path)
                            }
                            Err(err) => {
                                println!("Failed to save image fpr {}: {:?}", path.to_str().unwrap(), err);
                                None
                            }
                        }
                    }
                }
                None => None
            };


            let beatmap = BeatMap {
                song: data.metadata.songName,
                artist: data.metadata.artist,
                charter: data.metadata.charter,
                difficulty: data.metadata.difficulty,
                description: data.metadata.description,
                artistList: data.metadata.artistList,
                image: image.map(|inner| PathBuf::from(inner).iter().last().unwrap().to_str().unwrap().to_string()),
                download: path.iter().last().unwrap().to_str().unwrap().to_string(),
            };
            println!("{}", beatmap.download.len());
            let record: Option<BeatMap> = db.create("beatmaps").content(beatmap).await?;
        }
    }

    Ok(())
}


#[tokio::main]
async fn old_main() -> Result<(), Error> {
    // Connect to the server
    let db = Surreal::new::<Ws>("127.0.0.1:8000").await?;

    // Signin as a namespace, database, or root user
    db.signin(Root {
        username: "root",
        password: "root",
    })
        .await?;

    // Select a specific namespace / database
    db.use_ns("beatblock").use_db("beatblock").await?;

    for level in fs::read_dir(env::current_dir().unwrap().join("output"))? {
        let level = level?;
        if level.file_type()?.is_dir() || level.path().ends_with(".png") {
            continue;
        }

        println!("Trying {:?}", level.path());
        let (data, image, download) = match read_zip(level.path()) {
            Ok(result) => result,
            Err(error) => {
                println!("Error reading zip: {:?}", error);
                continue;
            }
        };

        let beatmap = BeatMap {
            song: data.metadata.songName,
            artist: data.metadata.artist,
            charter: data.metadata.charter,
            difficulty: data.metadata.difficulty,
            description: data.metadata.description,
            artistList: data.metadata.artistList,
            image: image.map(|inner| PathBuf::from(level.path().to_str().unwrap().replace(".zip", ".png"))
                .iter().last().unwrap().to_str().unwrap().to_string()),
            download: level.path().iter().last().unwrap().to_str().unwrap().to_string(),
        };
        println!("Got {}", beatmap.song);
        let record: Option<BeatMap> = db.create("beatmaps").content(beatmap).await?;
    }
    Ok(())
}

fn read_zip(mut path: PathBuf) -> Result<(Data, Option<Vec<u8>>, Vec<u8>), Error> {
    let mut file = File::open(path.clone())?;
    let mut archive = ZipArchive::new(&mut file)?;
    let mut data = None;
    let mut index = usize::MAX;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let file_name = file.name();

        // Check if the current file is `level.json`.
        if file_name.ends_with("level.json") && file_name.rfind("/").unwrap_or(0) < index {
            let mut output = String::new();
            index = file_name.rfind("/").unwrap_or(0);
            file.read_to_string(&mut output);
            data = Some(output);
        }
    }
    let Some(data) = data else {
        panic!("Failed to find level for {:?}", file)
    };
    let data: Data = serde_json::from_str(&data).unwrap();
    let mut index = usize::MAX;
    let mut image = None;
    if let Some(bgData) = data.metadata.bgData.as_ref() {
        if bgData.image.is_empty() {
            return Ok((data, None, fs::read(path).unwrap()));
        }
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();
            let file_name = file.name();

            // Check if the current file is `level.json`.
            if file_name.ends_with(&bgData.image) && file_name.rfind("/").unwrap_or(0) < index {
                let mut output = Vec::new();
                index = file_name.rfind("/").unwrap_or(0);
                file.read_to_end(&mut output).expect("Failed to read file!");
                image = Some(output);
            }
        }
    }

    return Ok((data, image, fs::read(path).unwrap()));
}

fn read_rar(mut file: PathBuf) -> Result<(Data, Option<Vec<u8>>, Vec<u8>), Error> {
    let archive = Archive::new(&file).open_for_processing()?;
    let mut archive = archive.read_header().unwrap().unwrap();
    let mut data = None;
    let mut index = usize::MAX;
    while true {
        let name = archive.entry().filename.to_str().unwrap();
        if name.ends_with("level.json") && name.rfind("/").unwrap_or(0) < index {
            index = name.rfind("/").unwrap_or(0);
            let (file_data, next) = archive.read().unwrap();
            archive = next.read_header().unwrap().unwrap();
            data = Some(file_data);
            break;
        } else {
            let (_, next) = archive.read().unwrap();
            archive = next.read_header().unwrap().unwrap();
        }
    }
    let Some(data) = data else {
        panic!("Failed to find level for {:?}", file)
    };
    let data: Data = serde_json::from_slice::<Data>(&data).unwrap();
    let archive = Archive::new(&file).open_for_processing().unwrap();
    let mut archive = archive.read_header().unwrap().unwrap();
    let mut image = None;
    let mut index = usize::MAX;
    if let Some(bgData) = data.metadata.bgData.as_ref() {
        while true {
            let name = archive.entry().filename.to_str().unwrap();
            if name.ends_with(&bgData.image) && name.rfind("/").unwrap_or(0) < index {
                index = name.rfind("/").unwrap_or(0);
                let (file_data, next) = archive.read().unwrap();
                archive = next.read_header().unwrap().unwrap();
                image = Some(file_data);
                break;
            } else {
                let (_, next) = archive.read().unwrap();
                archive = next.read_header().unwrap().unwrap();
            }
        }
    }

    let output = Vec::new();
    let mut zip = ZipWriter::new(Cursor::new(output));
    let mut archive = Archive::new(&file).open_for_processing().unwrap();
    while let Some(header) = archive.read_header().unwrap() {
        zip.start_file(header.entry().filename.to_str().unwrap(), SimpleFileOptions::default()).unwrap();
        let (file_data, next) = header.read().unwrap();
        zip.write(&file_data).unwrap();
        archive = next;
    }

    return Ok((data, image, zip.finish().unwrap().into_inner()));
}

#[derive(Deserialize)]
pub struct Data {
    metadata: LevelMetadata,
}

#[derive(Deserialize)]
pub struct LevelMetadata {
    artist: String,
    charter: String,
    difficulty: f32,
    description: String,
    songName: String,
    #[serde(default)]
    artistList: String,
    #[serde(default)]
    bgData: Option<BackgroundData>,
}

#[derive(Deserialize)]
pub struct BackgroundData {
    image: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BeatMap {
    song: String,
    artist: String,
    charter: String,
    difficulty: f32,
    description: String,
    artistList: String,
    image: Option<String>,
    download: String,
}