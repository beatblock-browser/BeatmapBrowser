use crate::parsing::{FileData, LevelData};
use anyhow::Error;
use std::io::{Cursor, Read, Seek};
use zip::ZipArchive;

pub fn read_zip(file: &mut Vec<u8>) -> Result<FileData, Error> {
    let mut cursor = Cursor::new(file);
    let mut archive = ZipArchive::new(&mut cursor)?;
    let data =
        fetch_file(&mut archive, "level.json")?;
    let Some(data) = data else {
        panic!("Failed to find level")
    };
    let data: LevelData = serde_json::from_slice(&data)?;
    let mut image = None;
    if let Some(bg_data) = data.metadata.bg_data.as_ref() {
        if !bg_data.image.is_empty() {
            image = fetch_file(&mut archive, &bg_data.image)?;
        }
    }

    Ok(FileData {
        level_data: data,
        image
    })
}

fn fetch_file<T: Seek + Read>(archive: &mut ZipArchive<T>, target_file_name: &str) -> Result<Option<Vec<u8>>, Error> {
    let mut size = usize::MAX;
    let mut index = usize::MAX;
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        let file_name = file.name();

        // Check if the current file is `level.json`.
        if file_name.ends_with(&target_file_name) && file_name.rfind("/").unwrap_or(0) < size {
            size = file_name.rfind("/").unwrap_or(0);
            index = i;
        }
    }
    Ok(if index == usize::MAX {
        None
    } else {
        let mut output = Vec::new();
        archive.by_index(index)?.read_to_end(&mut output).expect("Failed to read file!");
        Some(output)
    })
}