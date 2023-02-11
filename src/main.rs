#[macro_use]
extern crate clap;

use clap::{command, ArgMatches, Command, ArgAction};
use prettytable::row;

use std::{fs::File, path::PathBuf};

use bzip2::read::MultiBzDecoder;
use indicatif::HumanBytes;
use itertools::Itertools;
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

pub struct FileStat {
    pub extension: String,
    pub size: u64,
}

const PATH: &str = "PATH";

fn main() -> std::io::Result<()> {
    let app = build_cli();
    let matches = app.get_matches();

    let root = matches.get_one::<String>(PATH).unwrap();
    let output_as_html = matches.get_flag("html");

    match matches.subcommand() {
        Some(("e", cmd)) => show_extensions(root, output_as_html, cmd),
        Some(("s", cmd)) => search_extension(root, output_as_html, cmd),
        _ => default_action(root, output_as_html),
    }
}

fn default_action(root: &str, output_as_html: bool) -> std::io::Result<()> {
    let stat = collect_statistic(root)?;

    let total_files = stat.iter().fold(0, |mut acc, item| {
        acc += item.files.len();
        acc
    });

    let total_size = stat.iter().fold(0, |mut acc, item| {
        acc += item.size;
        acc
    });

    let mut resulter = Resulter::new(output_as_html);
    resulter.titles(row![bF=> "Archive", "Files", "Size"]);
    stat.iter()
        .sorted_by(|a, b| Ord::cmp(&a.title, &b.title))
        .for_each(|item| {
            resulter.append(item);
        });
    resulter.append_empty_row();
    resulter.append_row("Total", total_size, total_files as u64);
    resulter.print();
    Ok(())
}

fn show_extensions(root: &str, output_as_html: bool, cmd: &ArgMatches) -> std::io::Result<()> {
    let stat = collect_statistic(root)?;
    let max_ext_len = *cmd.get_one::<usize>("length").unwrap();
    let show_top_extensions = cmd.get_one::<usize>("top");

    let extensions = stat
        .iter()
        .flat_map(|s| s.files.iter())
        .into_grouping_map_by(|s| s.extension.clone())
        .fold(0, |acc: u64, _key, _val| acc + 1);

    let mut resulter = Resulter::new(output_as_html);
    resulter.titles(row![bF=> "#", "Extension", "Count"]);

    extensions
        .iter()
        .sorted_by(|a, b| Ord::cmp(&b.1, &a.1))
        .filter(|(e, _c)| e.len() <= max_ext_len)
        .enumerate()
        .take_while(|(num, (_, _))| {
            show_top_extensions.is_none() || *show_top_extensions.unwrap() > *num
        })
        .for_each(|(num, (ext, count))| {
            resulter.append_count_row(ext, num + 1, *count);
        });
    resulter.print();

    Ok(())
}

fn search_extension(root: &str, output_as_html: bool, cmd: &ArgMatches) -> std::io::Result<()> {
    let stat = collect_statistic(root)?;

    let ext_to_find = cmd.get_one::<String>("STRING").unwrap();

    let tars_with_ext = stat
        .iter()
        .filter(|s| s.files.iter().any(|x| x.extension == ext_to_find.as_str()))
        .collect_vec();

    let mut resulter = Resulter::new(output_as_html);
    let title = format!("Archive with '{ext_to_find}' extension");
    resulter.titles(row![bF=> "#", title, "Count"]);

    let mut total_count = 0u64;
    tars_with_ext
        .iter()
        .sorted_by(|a, b| Ord::cmp(&a.title, &b.title))
        .enumerate()
        .for_each(|(num, stat)| {
            let count = stat
                .files
                .iter()
                .filter(|f| f.extension == ext_to_find.as_str())
                .count() as u64;
            total_count += count;
            resulter.append_count_row(&stat.title, num + 1, count);
        });
    resulter.append_empty_row();
    resulter.append_count_row("Total", 0, total_count);
    resulter.print();

    Ok(())
}

fn collect_statistic(root: &str) -> std::io::Result<Vec<Statistic>> {
    let dir = std::fs::read_dir(root)?;
    let archives = dir
        .filter_map(std::result::Result::ok)
        .filter(|d| d.file_type().is_ok() && d.file_type().unwrap().is_file())
        .filter_map(|file| {
            let full_path = file.path();
            let meta = std::fs::metadata(full_path).ok()?;
            if file.path().as_path().extension().unwrap() == "bz2" {
                Some(FileInfo {
                    path: file.path(),
                    size: meta.len(),
                })
            } else {
                None
            }
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
        .sorted_by(|a, b| Ord::cmp(&b.size, &a.size))
        .filter_map(|arj| {
            let archive = File::open(arj.path.as_path()).ok()?;
            let mut bz2 = MultiBzDecoder::new(archive);
            let f = arj.path.file_stem().unwrap().to_str().unwrap();
            let p = PathBuf::from(root);
            let tar = p.join(f);

            let bytes = HumanBytes(arj.size);
            let msg = format!("  Current: {} ({bytes})", arj.path.to_string_lossy());
            progress.message(msg);

            if tar.exists() {
                unpacked_progress += arj.size;
                progress.progress(unpacked_progress);
                Some(tar)
            } else {
                let mut output = File::create(tar.as_path()).ok()?;
                match std::io::copy(&mut bz2, &mut output) {
                    Ok(_bytes) => {
                        unpacked_progress += arj.size;
                        progress.progress(unpacked_progress);
                        Some(tar)
                    }
                    Err(e) => {
                        println!("Decompress error: {e}");
                        None
                    }
                }
            }
        })
        .filter_map(|f| {
            let file_name = f.file_name()?.to_string_lossy().to_string();

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
                            .filter_map(|e| {
                                let file_size = e.header().size().unwrap_or_default();
                                match e.path() {
                                    Ok(p) => {
                                        let extension = p
                                            .extension()
                                            .unwrap_or_default()
                                            .to_string_lossy()
                                            .to_string();
                                        Some(FileStat {
                                            extension,
                                            size: file_size,
                                        })
                                    }
                                    Err(e) => {
                                        println!("  Error: {e}");
                                        None
                                    }
                                }
                            })
                            .collect_vec(),
                        Err(e) => {
                            println!("Error: {e}");
                            vec![]
                        }
                    };

                    let total_size = files.iter().fold(0, |mut acc, item| {
                        acc += item.size;
                        acc
                    });

                    Some(Statistic {
                        title: file_name,
                        files,
                        size: total_size,
                    })
                }
                Err(e) => {
                    println!("Error: {e}");
                    None
                }
            };
            result
        })
        .collect();
    progress.finish("  Completed");
    Ok(stat)
}

fn build_cli() -> Command {
    command!(crate_name!())
        .arg_required_else_help(true)
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about(crate_description!())
        .arg(
            arg!([PATH])
                .help("Sets Yandex archives path")
                .required(true)
                .index(1),
        )
        .arg(
            arg!(--html)
                .required(false)
                .action(ArgAction::SetTrue)
                .help("Output in HTML format"),
        )
        .subcommand(
            Command::new("e")
                .aliases(["extensions"])
                .about("Show extensions info")
                .arg(
                    arg!(-l --length <NUMBER>)
                        .required(false)
                        .value_parser(value_parser!(usize))
                        .default_value("10")
                        .help("The max length of file extension to output"),
                )
                .arg(
                    arg!(-t --top <NUMBER>)
                        .required(false)
                        .value_parser(value_parser!(usize))
                        .help("Output only specified number of extensions sorted by count"),
                ),
        )
        .subcommand(
            Command::new("s")
                .aliases(["search"])
                .about("Search archives with extensions specified")
                .arg(
                    arg!([STRING])
                        .help("An extension to search in archives")
                        .required(true)
                        .index(1),
                ),
        )
}
