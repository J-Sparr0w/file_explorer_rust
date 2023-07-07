use std::collections::HashMap;
use std::fs::{self, DirEntry, ReadDir};
use std::io::{self, ErrorKind};
use std::os::windows::prelude::MetadataExt;

use std::path::{Path, PathBuf};

pub type CacheMap = HashMap<String, Vec<PathBuf>>;
const FILE_ATTRIBUTE_HIDDEN: u32 = 0x02;

enum HiddenFileVisibility {
    Hide,
    Show,
}

pub struct FileWalker {
    root: PathBuf,
    max_depth: u16,
    curr_depth: u16,
    show_hidden_files: HiddenFileVisibility,
    files: Vec<PathBuf>,
    dirs: Vec<PathBuf>,
}

impl FileWalker {
    pub fn new() -> Self {
        FileWalker {
            root: PathBuf::from("."),
            max_depth: 3,
            curr_depth: 0,
            show_hidden_files: HiddenFileVisibility::Hide,
            files: Vec::new(),
            dirs: Vec::new(),
        }
    }

    pub fn get_all_files(&self) -> &Vec<PathBuf> {
        &self.files
    }

    pub fn get_all_dirs(&self) -> &Vec<PathBuf> {
        &self.dirs
    }

    pub fn set_hidden_file_visibility(&mut self) {
        self.show_hidden_files = HiddenFileVisibility::Show;
    }

    pub fn set_max_depth(&mut self, depth: u16) {
        self.max_depth = depth;
    }

    pub fn set_root(&mut self, path: &Path) {
        self.curr_depth = 0;
        self.root = path.to_path_buf();
    }
    fn increment_depth(&mut self) -> Result<(), String> {
        if self.curr_depth == self.max_depth {
            return Err(String::from("max_depth reached"));
        }
        self.curr_depth += 1;
        Ok(())
    }

    fn decrement_depth(&mut self) -> Result<(), ()> {
        if self.curr_depth == 1 {
            return Err(());
        }
        self.curr_depth -= 1;
        Ok(())
    }

    pub fn traverse_directory(
        &mut self,
        path: &Path,
        unvisited_dirs: &mut Vec<DirEntry>,
    ) -> Result<usize, String> {
        let entries: Result<ReadDir, io::Error> = fs::read_dir(path);
        let starting_len = unvisited_dirs.len();
        if let Err(error) = self.increment_depth() {
            return Err(error);
        }

        match self.show_hidden_files {
            HiddenFileVisibility::Hide => {
                for each_entry in entries.unwrap() {
                    if let Ok(entry) = each_entry {
                        if is_hidden(&entry).unwrap() {
                            continue;
                        }
                        let metadata = entry.metadata().unwrap();
                        let entry_is_a_dir = metadata.is_dir();

                        // println!("\tpath: {:?}", entry.file_name());

                        if entry_is_a_dir {
                            unvisited_dirs.push(entry);
                        } else {
                            self.files.push(entry.path());
                        }
                    }
                }
            }
            HiddenFileVisibility::Show => {
                for each_entry in entries.unwrap() {
                    if let Ok(entry) = each_entry {
                        let metadata = entry.metadata().unwrap();
                        let entry_is_a_dir = metadata.is_dir();

                        if entry_is_a_dir {
                            unvisited_dirs.push(entry);
                        } else {
                            self.files.push(entry.path());
                        }
                    }
                }
            }
        }
        // if there are no files in directory it returns count as 0
        let current_len = unvisited_dirs.len() - starting_len;
        Ok(current_len)
    }

    pub fn traverse_all_files_from_root(&mut self) -> Result<&Self, io::Error> {
        let mut unvisited_dirs: Vec<DirEntry> = Vec::new();
        let root = self.root.clone();
        let mut prev_unvisited_dir_count;

        // println!("reading dir: {:?}", self.root);

        //start traversing from root
        match self.traverse_directory(&root, &mut unvisited_dirs) {
            Ok(count) => {
                prev_unvisited_dir_count = count;
            }
            Err(e) => {
                eprintln!("ERROR: traversing exited because: {e}");
                return Err(io::Error::new(ErrorKind::Other, e));
            }
        }

        self.dirs.push(root);

        //after traversing the root dir, the unvisitedDir will be filled with directories to traverse
        while !unvisited_dirs.is_empty() {
            if self.curr_depth == self.max_depth {
                println!("max_depth reached!!!");
                println!("Popping {prev_unvisited_dir_count} elements to go back one level");
                while prev_unvisited_dir_count > 0 {
                    let path = unvisited_dirs.pop().unwrap().path();
                    prev_unvisited_dir_count -= 1;

                    self.dirs.push(path);
                }
                self.decrement_depth().unwrap();
                continue;
            }

            let path = unvisited_dirs.pop().unwrap().path();
            println!("dir : {:?}", path.file_name().unwrap());

            match self.traverse_directory(&path, &mut unvisited_dirs) {
                Ok(count) => {
                    prev_unvisited_dir_count = count;
                }
                Err(e) => {
                    eprintln!("{e}");
                }
            }
            self.dirs.push(path);
        }
        return Ok(self);
    }
}

fn is_hidden(file: &DirEntry) -> io::Result<bool> {
    let metadata = file.metadata()?;
    let file_attr = metadata.file_attributes();
    //FILE_ATTRIBUTE_HIDDEN is 0x02 for windows and
    //any number that results in a number greater than zero after bitwise-and with it is hidden
    return Ok(file_attr & FILE_ATTRIBUTE_HIDDEN != 0);
}
