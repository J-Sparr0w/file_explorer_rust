// use serde_json;
// use std::collections::HashMap;
use serde_json;
use std::fs::File;
use std::io::{self, ErrorKind, Read, Write};
use std::mem;
// use std::mem;
// use std::os::windows::prelude::MetadataExt;
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::Instant;
// use sysinfo::{DiskExt, System, SystemExt};
// use file_walker::FileWalker;
mod file_walker;
use file_walker::{CacheMap, FileWalker};

fn add_to_hashmap(path: &PathBuf, hash: &mut CacheMap) {
    // let file_name = entry.file_name().to_string_lossy().into_owned();
    // let path = entry.path();
    let name_without_extension = path
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .to_ascii_lowercase();
    let mut path_list: Vec<PathBuf> = vec![path.clone()];
    // path_list.push(path.clone());

    // player_stats.entry("mana").and_modify(|mana| *mana += 200).or_insert(100); => modify a hashmap

    if let Some(list) = hash.get_mut(&name_without_extension) {
        path_list.append(list);
        hash.insert(name_without_extension, path_list);
    } else {
        hash.insert(name_without_extension, path_list);
    }
}

fn print_elapsed_time(starting_time: Instant) -> () {
    //benchmark code
    println!("time taken: {time:?}", time = starting_time.elapsed());
}

fn to_gigabytes(x: u64) -> f32 {
    x as f32 / (1024u64.pow(3) as f32)
    // (x as f64 * (1e-9))
}

fn save_cache(cache: &CacheMap, cache_file: &mut File) {
    if let Ok(json) = serde_json::to_string(cache) {
        cache_file.write(json.as_bytes());
    }
}

fn read_cache(cache_file: &mut File) -> Result<CacheMap, io::Error> {
    let mut json = String::new();
    // println!("From File: ");
    cache_file.read_to_string(&mut json)?;
    println!("Loaded cache from file");

    let cache_from_str: CacheMap = serde_json::from_str(&json)?;
    Ok(cache_from_str)
}

fn main() {
    // let _dir_path: &Path = Path::new("D:/projects/Rust/file_explorer");
    // let mut sys = System::new();

    let mut drive_paths: Vec<&Path> = Vec::new();
    let mut new_cache: CacheMap = CacheMap::new();
    let cache_file_path = Path::new("cache.json");
    let mut cache_file: File;

    match File::open(cache_file_path) {
        Ok(file) => {
            cache_file = file;
            new_cache = read_cache(&mut cache_file)
                .map_err(|err| {
                    eprintln!("ERROR: unable to read cache file\nCODE: {err}");
                })
                .unwrap();
        }
        Err(err) => match err.kind() {
            ErrorKind::NotFound => {
                let mut file_walker = FileWalker::new();

                cache_file = File::create(cache_file_path).unwrap();
                // init_file

                println!("\ndrive paths: {drive_paths:?}\n");

                // drive_paths.push(Path::new("."));
                drive_paths.push(Path::new("D:\\Movies and series"));

                for path in drive_paths {
                    file_walker.set_root(path);

                    for file in file_walker
                        .traverse_all_files_from_root()
                        .unwrap()
                        .get_all_files()
                        .iter()
                    {
                        add_to_hashmap(&file, &mut new_cache);
                    }

                    for dir in file_walker.get_all_dirs().iter() {
                        add_to_hashmap(&dir, &mut new_cache);
                    }

                    // println!("\nno. of files: {}", file_walker.files.len());
                    // println!("no. of dirs: {}\n", file_walker.dirs.len());
                }

                save_cache(&new_cache, &mut cache_file);
            }
            _ => {
                eprintln!("ERROR: file cannot be opened....{err} ");
                exit(1);
            }
        },
    }

    let mut input = String::new();

    print!("search here: ");
    io::stdout().flush().unwrap();
    io::stdin().read_line(&mut input).unwrap();

    let search_word = input.trim().to_ascii_lowercase();

    let starting_time = Instant::now();
    match new_cache.get(&search_word) {
        Some(paths) => {
            for path in paths {
                println!("{path:?}");
            }
        }
        None => {
            let mut found = false;
            for key in new_cache.keys() {
                if key.is_empty() {
                    continue;
                }
                if key.contains(&search_word) {
                    found = true;
                    let paths = new_cache.get(key).unwrap();
                    for path in paths {
                        println!(
                            "{file_name:?}=>\t\t{path:?}",
                            file_name = path.file_name().unwrap()
                        );
                    }
                }
            }
            if !found {
                println!("no such file exists");
            }
        }
    }
    print_elapsed_time(starting_time);

    // let size = mem::size_of_val(&file_walker);
    // println!("Size : {} bytes", size);

    // sys.refresh_disks_list();
    // println!("=> disks:");
    // for disk in sys.disks() {
    //     println!();
    //     // println!("mount point: {:?}", disk.mount_point());
    //     drive_paths.push(disk.mount_point());
    //     println!("kind: {:?}", disk.kind());
    //     println!("name: {:?}", disk.name());
    //     println!("total space: {:?}GB", to_gigabytes(disk.total_space()));
    //     println!(
    //         "available space: {:?}",
    //         to_gigabytes(disk.available_space())
    //     );
    // }

    // print!("hash: {new_cache:?}");

    // println!();
    // println!("total no. of dirs: {dir_len} ", dir_len = dirs.len());
    // println!("total no. of files: {file_len} ", file_len = files.len());
}
