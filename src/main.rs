// use std::env;
use flate2::read::GzDecoder;
use std::fs::File;
use std::io;
use std::path::PathBuf;
use tar::Archive;
use zip::ZipArchive;

use clap::{CommandFactory, Parser};

/// Partial implementation of `tar xf`
#[derive(Parser, Debug)]
#[command(version, about, author, long_about = None)]
struct Args {
    #[arg(short('C'), long("directory"), help("change to directory DIR"), default_value_t = String::from("."))]
    directory: String,

    files: Vec<PathBuf>,
}

fn main() -> io::Result<()> {
    // let args: Vec<String> = env::args().collect();
    // println!("{:?}", args);

    let args = Args::parse();
    let working_dir = args.directory;
    // println!("Working directory: {}", working_dir);

    if args.files.is_empty() {
        Args::command().print_help()?;
        std::process::exit(0);
    }

    for file in args.files {
        if file.extension().is_none() {
            // println!("Unsupported file extension: {}", file.display());
            continue;
        }
        let ext = file.extension().unwrap().to_str().unwrap();
        match ext {
            "gz" => {
                // println!("Extracting {:?} -> {}", file, working_dir);
                let stem = file.file_stem().unwrap().to_string_lossy().into_owned();
                if !stem.ends_with(".tar") {
                    continue;
                }
                let temp_dir = tempfile::tempdir()?;
                let tar_file = temp_dir.path().join(stem);
                decompress(file.to_str().unwrap(), tar_file.to_str().unwrap())?;
                extract(tar_file.to_str().unwrap(), &working_dir)?;
            }
            "tgz" => {
                // println!("Extracting {:?} -> {}", file, working_dir);
                let stem = file.file_stem().unwrap().to_string_lossy().into_owned();
                let temp_dir = tempfile::tempdir()?;
                let tar_file = temp_dir.path().join(stem + ".tar");
                decompress(file.to_str().unwrap(), tar_file.to_str().unwrap())?;
                extract(tar_file.to_str().unwrap(), &working_dir)?;
            }
            "tar" => {
                // println!("Extracting {:?} -> {}", file, working_dir);
                extract(file.to_str().unwrap(), &working_dir)?;
            }
            "zip" => {
                // println!("Extracting {:?} -> {}", file, working_dir);
                unzip(file.to_str().unwrap(), &working_dir)?;
            }
            _ => {
                // println!("Unsupported file extension: {}", file.display());
            }
        }
    }
    Ok(())
}

fn decompress(input: &str, output: &str) -> io::Result<()> {
    let gzip_file = File::open(input)?;
    let mut decoder = GzDecoder::new(gzip_file);
    let mut tar_file = File::create(output)?;

    io::copy(&mut decoder, &mut tar_file)?;
    Ok(())
}

fn extract(tar_path: &str, dest: &str) -> io::Result<()> {
    let file = File::open(tar_path)?;
    let mut archive = Archive::new(file);

    archive.unpack(dest)?;
    Ok(())
}

fn unzip(zip_path: &str, dest: &str) -> io::Result<()> {
    let zip_file = File::open(zip_path)?;

    let mut archive = ZipArchive::new(zip_file)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;

        let mut extract_path = PathBuf::from(dest);
        extract_path.push(entry.name());

        if let Some(parent) = extract_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if !entry.is_dir() {
            // println!("Extracting: {}", extract_path.display());
            let mut output_file = File::create(extract_path)?;
            std::io::copy(&mut entry, &mut output_file)?;
        } else {
            // println!("Creating directory: {}", extract_path.display());
            std::fs::create_dir_all(extract_path)?;
        }
    }
    Ok(())
}
