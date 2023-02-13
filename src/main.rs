#[macro_use]
extern crate clap;

use clap::{command, ArgAction, ArgMatches, Command};
use prettytable::row;

use std::{collections::HashMap, fs::File, path::PathBuf};

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
const OTHER_CAT: &str = "";
const SHELL_CAT: &str = "Shell";
const JAVA_CAT: &str = "Java";
const PYTHON_CAT: &str = "Python";
const IMAGE_CAT: &str = "Image";
const WEB_CAT: &str = "Web/Frontend";
const CONFIG_CAT: &str = "Config/Data";
const ARCHIEVE_CAT: &str = "Archive";
const CPP_CAT: &str = "C/C++";
const GO_CAT: &str = "Go";
const DB_CAT: &str = "Database";
const DOC_CAT: &str = "Documentation";
const SWIFT_CAT: &str = "Swift";
const PHP_CAT: &str = "PHP";
const DOTNET_CAT: &str = ".NET";
const CERTIFICATE_CAT: &str = "Certificate";
const PERL_CAT: &str = "Perl";
const MULTIMEDIA_CAT: &str = "Multimedia";
const TPL_CAT: &str = "Template";
const RUST_CAT: &str = "Rust";
const RUBY_CAT: &str = "Ruby";
const LUA_CAT: &str = "Lua";
const ASSEMBLER_CAT: &str = "Assembler";

const TECHOLOGIES: &[(&str, &str)] = &[
    ("java", JAVA_CAT),
    ("py", PYTHON_CAT),
    ("pyx", PYTHON_CAT),
    ("png", IMAGE_CAT),
    ("js", WEB_CAT),
    ("ts", WEB_CAT),
    ("json", CONFIG_CAT),
    ("gz", ARCHIEVE_CAT),
    ("yaml", CONFIG_CAT),
    ("cpp", CPP_CAT),
    ("h", CPP_CAT),
    ("tsx", WEB_CAT),
    ("make", CONFIG_CAT),
    ("yml", CONFIG_CAT),
    ("kt", JAVA_CAT),
    ("go", GO_CAT),
    ("sql", DB_CAT),
    ("xml", CONFIG_CAT),
    ("md", DOC_CAT),
    ("css", WEB_CAT),
    ("scala", JAVA_CAT),
    ("svg", IMAGE_CAT),
    ("hpp", CPP_CAT),
    ("swift", SWIFT_CAT),
    ("txt", DOC_CAT),
    ("conf", CONFIG_CAT),
    ("jpg", IMAGE_CAT),
    ("c", CPP_CAT),
    ("cxx", CPP_CAT),
    ("sh", SHELL_CAT),
    ("html", WEB_CAT),
    ("scss", WEB_CAT),
    ("proto", CONFIG_CAT),
    ("styl", WEB_CAT),
    ("csv", CONFIG_CAT),
    ("ext", OTHER_CAT),
    ("m", CONFIG_CAT),
    ("imageset", IMAGE_CAT),
    ("info", OTHER_CAT),
    ("php", PHP_CAT),
    ("gif", IMAGE_CAT),
    ("sls", DOC_CAT),
    ("ogg", IMAGE_CAT),
    ("cs", DOTNET_CAT),
    ("ipynb", PYTHON_CAT),
    ("pm", OTHER_CAT),
    ("properties", CONFIG_CAT),
    ("webp", IMAGE_CAT),
    ("d", OTHER_CAT),
    ("pdf", DOC_CAT),
    ("resx", CONFIG_CAT),
    ("yql", DB_CAT),
    ("cc", CPP_CAT),
    ("net", CONFIG_CAT),
    ("i18n", CONFIG_CAT),
    ("zip", ARCHIEVE_CAT),
    ("jsx", JAVA_CAT),
    ("symlink", SHELL_CAT),
    ("pem", CERTIFICATE_CAT),
    ("j2", TPL_CAT),
    ("pl", PERL_CAT),
    ("xsl", CONFIG_CAT),
    ("snap", CONFIG_CAT),
    ("t", CONFIG_CAT),
    ("sqlt", DB_CAT),
    ("tsv", CONFIG_CAT),
    ("less", WEB_CAT),
    ("tgz", ARCHIEVE_CAT),
    ("mp3", MULTIMEDIA_CAT),
    ("mp4", MULTIMEDIA_CAT),
    ("sass", WEB_CAT),
    ("psql", DB_CAT),
    ("jpeg", IMAGE_CAT),
    ("wav", MULTIMEDIA_CAT),
    ("mustache", TPL_CAT),
    ("jinja2", TPL_CAT),
    ("kts", JAVA_CAT),
    ("cmake", CONFIG_CAT),
    ("dart", WEB_CAT),
    ("template", TPL_CAT),
    ("tmpl", TPL_CAT),
    ("tmpl-specs", TPL_CAT),
    ("lproj", CONFIG_CAT),
    ("wiki", DOC_CAT),
    ("vcxproj", CONFIG_CAT),
    ("bazel", CONFIG_CAT),
    ("toml", CONFIG_CAT),
    ("jar", JAVA_CAT),
    ("bash", SHELL_CAT),
    ("m4", TPL_CAT),
    ("vcproj", CONFIG_CAT),
    ("gzt", ARCHIEVE_CAT),
    ("csproj", CONFIG_CAT),
    ("ps1", SHELL_CAT),
    ("hcl", CONFIG_CAT),
    ("tf", CONFIG_CAT),
    ("sqlt", DB_CAT),
    ("tpl", TPL_CAT),
    ("rs", RUST_CAT),
    ("rb", RUBY_CAT),
    ("lua", LUA_CAT),
    ("gpg", CERTIFICATE_CAT),
    ("s", ASSEMBLER_CAT),
    ("ico", IMAGE_CAT),
    ("xlsx", CONFIG_CAT),
    ("groovy", JAVA_CAT),
    ("ini", CONFIG_CAT),
];

lazy_static::lazy_static! {
    static ref TECHOLOGIES_MAP: HashMap<&'static str, &'static str> = TECHOLOGIES.iter().map(|(k, v)| (*k, *v)).collect();
}

fn main() -> std::io::Result<()> {
    let app = build_cli();
    let matches = app.get_matches();

    let root = matches.get_one::<String>(PATH).unwrap();
    let output_as_html = matches.get_flag("html");

    match matches.subcommand() {
        Some(("e", cmd)) => show_extensions(root, output_as_html, cmd),
        Some(("s", cmd)) => search_extension(root, output_as_html, cmd),
        Some(("t", cmd)) => show_technologies(root, output_as_html, cmd),
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

fn show_technologies(root: &str, output_as_html: bool, cmd: &ArgMatches) -> std::io::Result<()> {
    let stat = collect_statistic(root)?;
    let show_top_extensions = cmd.get_one::<usize>("top");

    let extensions = stat
        .iter()
        .flat_map(|s| s.files.iter())
        .into_grouping_map_by(|s| {
            if TECHOLOGIES_MAP.contains_key(s.extension.as_str()) {
                (*TECHOLOGIES_MAP.get(s.extension.as_str()).unwrap()).to_string()
            } else {
                OTHER_CAT.to_string()
            }
        })
        .fold(0, |acc: u64, _key, _val| acc + 1);

    let mut resulter = Resulter::new(output_as_html);
    resulter.titles(row![bF=> "#", "Technology/Language", "Count"]);

    extensions
        .iter()
        .filter(|s| !s.0.is_empty())
        .sorted_by(|a, b| Ord::cmp(&b.1, &a.1))
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
    let archives = collect_files(root, "bz2")?;

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

fn collect_files(root: &str, extension: &str) -> Result<Vec<FileInfo>, std::io::Error> {
    let dir = std::fs::read_dir(root)?;
    let files = dir
        .filter_map(std::result::Result::ok)
        .filter(|d| d.file_type().is_ok() && d.file_type().unwrap().is_file())
        .filter_map(|file| {
            let full_path = file.path();
            let meta = std::fs::metadata(full_path).ok()?;
            if file.path().as_path().extension().unwrap() == extension {
                Some(FileInfo {
                    path: file.path(),
                    size: meta.len(),
                })
            } else {
                None
            }
        })
        .collect::<Vec<FileInfo>>();
    Ok(files)
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
        .subcommand(
            Command::new("t")
                .aliases(["techologies"])
                .about("Show technologies info")
                .arg(
                    arg!(-t --top <NUMBER>)
                        .required(false)
                        .value_parser(value_parser!(usize))
                        .help("Output only specified number of technologies sorted by count"),
                ),
        )
}
