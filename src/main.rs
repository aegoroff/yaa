use std::{fs::File, path::PathBuf};

use bzip2::read::MultiBzDecoder;
use indicatif::HumanBytes;
use itertools::Itertools;
use num_format::{Locale, ToFormattedString};
use progress::Progresser;
use tar::Archive;

use crate::{
    print::{Resulter, Statistic},
    progress::Progress,
};

pub mod print;
pub mod progress;

struct FileInfo {
    path: PathBuf,
    size: u64,
}

fn main() -> std::io::Result<()> {
    let root = "/home/egr/Downloads/ya";
    let dir = std::fs::read_dir(root)?;

    let archives = dir
        .filter_map(|entry| entry.ok())
        .filter(|d| d.file_type().is_ok() && d.file_type().unwrap().is_file())
        .filter_map(|file| {
            let full_path = file.path();
            let meta = std::fs::metadata(full_path).ok()?;
            Some(FileInfo {
                path: file.path(),
                size: meta.len(),
            })
        })
        .collect::<Vec<FileInfo>>();

    let compressed_size = archives.iter().fold(0, |mut acc, x| {
        acc += x.size;
        acc
    });

    let mut unpacked_progress = 0u64;

    let mut progress = Progresser::new(compressed_size);
    progress.progress(0);
    let stat: Vec<Statistic> = archives
        .iter()
        .sorted_by_key(|x| x.size)
        .rev()
        .filter_map(|arj| {
            let archive = File::open(arj.path.as_path()).ok()?;
            let mut bz2 = MultiBzDecoder::new(archive);
            let f = arj.path.file_stem().unwrap().to_str().unwrap();
            let p = PathBuf::from(root);
            let tar = p.join(f);

            let bytes = HumanBytes(arj.size);
            let msg = format!("  Current: {} ({bytes})", arj.path.to_string_lossy());
            progress.message(msg);
            let mut output = File::create(tar.as_path()).ok()?;
            match std::io::copy(&mut bz2, &mut output) {
                Ok(bytes) => {
                    unpacked_progress += arj.size;
                    progress.progress(unpacked_progress);
                    Some((tar, bytes))
                }
                Err(e) => {
                    println!("Decompress error: {e}");
                    None
                }
            }
        })
        .filter_map(|(f, size)| {
            let path = f.to_string_lossy().to_string();

            let result = match File::open(f) {
                Ok(tar) => {
                    let mut a = Archive::new(tar);
                    let files = match a.entries() {
                        Ok(files) => files
                            .filter_map(|e| match e {
                                Ok(entry) => Some(entry),
                                Err(e) => {
                                    println!("  Error: {e}");
                                    None
                                }
                            })
                            .filter_map(|e| match e.path() {
                                Ok(p) => Some(p.to_path_buf()),
                                Err(e) => {
                                    println!("  Error: {e}");
                                    None
                                }
                            })
                            .collect_vec(),
                        Err(e) => {
                            println!("Error: {e}");
                            vec![]
                        }
                    };
                    Some(Statistic {
                        path: path.clone(),
                        count: files.len() as u64,
                        size,
                    })
                }
                Err(e) => {
                    println!("Error: {e}");
                    None
                }
            };

            std::fs::remove_file(path).unwrap_or_default();
            result
        })
        .collect();

    let total_files = stat.iter().fold(0, |mut acc, item| {
        acc += item.count;
        acc
    });

    let total_size = stat.iter().fold(0, |mut acc, item| {
        acc += item.size;
        acc
    });

    progress.finish("Completed");

    let mut resulter = Resulter::new();
    stat.into_iter()
        .sorted_by(|a, b| Ord::cmp(&a.path, &b.path))
        .for_each(|item| {
            resulter.append(item);
        });
    resulter.print();

    let size = format!("Total size: {}", HumanBytes(total_size));
    println!("{size}");
    let files = total_files.to_formatted_string(&Locale::ru);
    println!("Total files: {files}");
    Ok(())
}
