/**
 * Copyright (C) 2023 Luis
 * 
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 * 
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 * 
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use std::{
    collections::HashMap,
    fs::{create_dir_all, rename, File},
    io::{self, BufReader, Read},
    path::PathBuf,
};

use flate2::bufread::GzDecoder;
use tar::Archive;

fn write_to_file(file_name: &str, entry: &mut impl Read) -> Result<(), anyhow::Error> {
    let mut path = PathBuf::new();
    path.push(file_name);

    create_dir_all(path.parent().unwrap()).unwrap();

    let mut file = File::create(path)?;

    io::copy(entry, &mut file)?;

    Ok(())
}

fn try_move(hash: u128, file_name: &str) {
    let path = PathBuf::from(file_name);
    create_dir_all(path.parent().unwrap()).unwrap();

    /* Try move main asset file */
    let hash_name = format!("{hash:016x}");
    let _ = rename(&hash_name, &file_name);

    /* Try move main meta file */
    let hash_name = format!("{hash:016x}.meta");
    let file_name = format!("{file_name}.meta");
    let _ = rename(&hash_name, &file_name);
}

/**
 * unitypackage files are GZipped tar-files.
 * It's a flat hierarchy of 32-character folders with the following contents:
 *  - pathname: a UTF-8 text file with the name of the content
 *  - asset.meta: a YAML file storing import settings and editor modifications
 *  - asset: the actual file (optional for folders)
 *  - preview.png: a thumbnail [128x128] (optional)
 */
pub(crate) fn unpack(file_name: &str) -> anyhow::Result<()> {
    let file = File::open(file_name)?;
    let bufread = BufReader::with_capacity(0x10000, file);
    let gzr = GzDecoder::new(bufread);
    let mut tar = Archive::new(gzr);

    let entries = tar.entries()?;

    let mut file_names = HashMap::<u128, String>::new();

    for entry in entries {
        let mut entry = entry?;
        let header = entry.header();
        let path = header.path()?;
        let tar_path = path.to_str().unwrap();
        let curr = u128::from_str_radix(&tar_path[..32], 16)?;

        let file_name = file_names.get(&curr);

        match &tar_path[32..] {
            "/asset" => {
                /* Main file */
                /* Note: In a well-formed unitypackage file, the pathname will always be the last */
                /*       file in the tarball. The first case is therefor not possible. */
                match file_name {
                    Some(file_name) => {
                        write_to_file(&file_name, &mut entry)?;
                    }
                    None => {
                        let file_name = format!("{curr:016x}");
                        write_to_file(&file_name, &mut entry)?;
                    }
                };
            }
            "/asset.meta" => {
                /* Meta file */
                /* Note: See case "/asset" */
                match file_name {
                    Some(file_name) => {
                        let file_name = format!("{file_name}.meta");
                        write_to_file(&file_name, &mut entry)?;
                    }
                    None => {
                        let file_name = format!("{curr:016x}.meta");
                        write_to_file(&file_name, &mut entry)?;
                    }
                };
            }
            "/pathname" => {
                let mut file_name = String::new();
                let _ = entry.read_to_string(&mut file_name)?;

                try_move(curr, &file_name);

                file_names.insert(curr, file_name);
            }
            "/preview.png" | "/" => {}
            file_name => {
                panic!("unknown file name: \"{file_name}\"");
            }
        }
    }

    Ok(())
}
