extern crate pad;
extern crate regex;
extern crate colored;

pub mod types;
pub mod parser;

use std::fs;
use std::path::PathBuf;
use regex::Regex;
use types::*;
use colored::*;

/// function to determine whether something is an artifact. 
///
/// Rules:
/// - if it's included in the .gitignore and has a '.a' or '.o' extension,
/// it's probably an artifact
/// - if it's a '.a' or '.o' it's probably an artifact
/// - 
fn is_artifact(p: PathBuf, re: Option<Regex>) -> bool {
    let regex = if let Some(r) = re { r }
        else { Regex::new(r".?\.(a|o").unwrap() }; // FIXME use lazy_static
    let path_str = &p.into_os_string().into_string().expect("OS String invalid.");
    regex.is_match(path_str)
}

pub fn read_files(in_paths: &PathBuf, depth: u8, min_bytes: Option<u64>, silent: bool) -> FileTree {
    let mut tree = FileTree::new();
    let mut total_size = FileSize::new(0);

    if let Ok(paths) = fs::read_dir(&in_paths) {
        for p in paths {
            let path = p.unwrap().path(); // TODO no unwraps; idk what this error would be though.

            // if this fails, it's probably because `path` is a symlink, so we ignore it.
            if let Ok(metadata) = fs::metadata(&path) {
                // append file size/name for a file
                if metadata.is_file() {
                    let file_size = FileSize::new(metadata.len());
                    if let Some(b) = min_bytes {
                        if file_size >= FileSize::new(b) {
                            tree.push(path.clone(), file_size, None, depth + 1);
                        }
                    }
                    else {
                        tree.push(path, file_size, None, depth + 1);
                    }
                    total_size.add(file_size);
                }

                // otherwise, go deeper
                else if metadata.is_dir() {
                    let mut subtree = read_files(&path, depth + 1, min_bytes, silent);
                    let dir_size = subtree.file_size;
                    if let Some(b) = min_bytes {
                        if dir_size >= FileSize::new(b) {
                            tree.push(path, dir_size, Some(&mut subtree), depth + 1);
                        }
                    }
                    else {
                        tree.push(path, dir_size, Some(&mut subtree), depth + 1);
                    }
                    total_size.add(dir_size);
                }
            }
            else if !silent {
                println!("{}: ignoring symlink at {}", "Warning".yellow(), path.display());
            }
        }
    }
    else if !silent {
        println!("{}: permission denied for directory: {}", "Warning".yellow(), &in_paths.display());
    }
    tree
}

pub fn read_files_regex(in_paths: &PathBuf, depth: u8, min_bytes: Option<u64>, regex: &Regex, silent: bool) -> FileTree {
    let mut tree = FileTree::new();
    let mut total_size = FileSize::new(0);

    if let Ok(paths) = fs::read_dir(&in_paths) {
        for p in paths {
            let path = p.unwrap().path(); // TODO no unwraps; idk what this error would be though.
            let path_string = &path.clone().into_os_string().into_string().expect("OS String invalid.");

            if !regex.is_match(path_string) {
                // if this fails, it's probably because `path` is a symlink, so we ignore it.
                if let Ok(metadata) = fs::metadata(&path) {
                    // append file size/name for a file
                    if metadata.is_file() {
                        let file_size = FileSize::new(metadata.len());
                        if let Some(b) = min_bytes {
                            if file_size >= FileSize::new(b) {
                                tree.push(path.clone(), file_size, None, depth + 1);
                            }
                        }
                        else {
                            tree.push(path, file_size, None, depth + 1);
                        }
                        total_size.add(file_size);
                    }

                    // otherwise, go deeper
                    else if metadata.is_dir() {
                        let mut subtree = read_files_regex(&path, depth + 1, min_bytes, regex, silent);
                        let dir_size = subtree.file_size;
                        if let Some(b) = min_bytes {
                            if dir_size >= FileSize::new(b) {
                                tree.push(path, dir_size, Some(&mut subtree), depth + 1);
                            }
                        }
                        else {
                            tree.push(path, dir_size, Some(&mut subtree), depth + 1);
                        }
                        total_size.add(dir_size);
                    }
                }
                else if !silent {
                    println!("{}: ignoring symlink at {}", "Warning".yellow(), path.display());
                }
            }
        }
    }
    else if !silent {
        println!("{}: permission denied for directory: {}", "Warning".yellow(), &in_paths.display());
    }
    tree
}
