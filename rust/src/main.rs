use anyhow::{anyhow, Context, Result};
use clap::Parser;
use std::fs::{self, File};
use std::io::{self, BufReader, Read, Seek};
use std::path::PathBuf;
use std::process::exit;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Extract tar/tar.gz/tgz/tar.xz/tar.bz2/tar.zip packages
#[derive(Parser, Debug)]
#[command(name = "untar")]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output directory
    #[arg(short = 'd', long, value_name = "DIR")]
    directory: Option<String>,

    /// Verbose mode
    #[arg(short, long)]
    verbose: bool,

    /// Show help
    #[arg(short, long)]
    help: bool,

    /// Archive file to extract
    #[arg(value_name = "FILE", index = 1)]
    file: Option<String>,
}

fn main() {
    let args = Args::parse();

    if args.help {
        print!("{}", HELP);
        exit(0);
    }

    let file = match args.file {
        Some(f) => f,
        None => {
            eprintln!("Error: No archive file specified");
            print!("{}", HELP);
            exit(1);
        }
    };

    let verbose = args.verbose;
    let directory = args.directory.unwrap_or_else(|| ".".to_string());

    if let Err(e) = extract_archive(&file, &directory, verbose) {
        eprintln!("Error: {}", e);
        if verbose {
            eprintln!("{:?}", e);
        }
        exit(1);
    }

    if verbose {
        println!("Done: {}", file);
    }
}

const HELP: &str = "untar - Extract tar/tar.gz/tgz/tar.xz/tar.bz2/tar.zip packages

Usage: untar [OPTIONS] FILE

Options:
  -d, --directory DIR    Extract files into DIR (default: current directory)
  -v, --verbose          Show progress
  -h, --help             Show this help

Supported formats:
  .tar, .tar.gz, .tgz, .tar.xz, .tar.bz2, .zip
";

fn extract_archive(file_path: &str, output_dir: &str, verbose: bool) -> Result<()> {
    let file = File::open(file_path)
        .with_context(|| format!("Cannot open file: {}", file_path))?;

    let file_name_lower = file_path.to_lowercase();

    // Create output directory
    fs::create_dir_all(output_dir)
        .with_context(|| format!("Cannot create directory: {}", output_dir))?;

    // Detect format by extension and extract
    if file_name_lower.ends_with(".tar.gz") || file_name_lower.ends_with(".tgz") {
        extract_tar_gz(file, output_dir, verbose)?;
    } else if file_name_lower.ends_with(".tar.xz") {
        extract_tar_xz(file, output_dir, verbose)?;
    } else if file_name_lower.ends_with(".tar.bz2") {
        extract_tar_bz2(file, output_dir, verbose)?;
    } else if file_name_lower.ends_with(".zip") {
        extract_zip(file, output_dir, verbose)?;
    } else if file_name_lower.ends_with(".tar") {
        extract_tar(file, output_dir, verbose)?;
    } else {
        return Err(anyhow!(
            "Unsupported archive format. Please use a known extension (.tar, .tar.gz, .tgz, .tar.xz, .tar.bz2, .zip)"
        ));
    }

    Ok(())
}

fn extract_tar_gz<R: Read>(reader: R, output_dir: &str, verbose: bool) -> Result<()> {
    let decoder = flate2::read::GzDecoder::new(reader);
    extract_tar_reader(decoder, output_dir, verbose)
}

fn extract_tar_xz<R: Read>(reader: R, output_dir: &str, verbose: bool) -> Result<()> {
    let decoder = xz2::read::XzDecoder::new(reader);
    extract_tar_reader(decoder, output_dir, verbose)
}

fn extract_tar_bz2<R: Read>(reader: R, output_dir: &str, verbose: bool) -> Result<()> {
    let decoder = bzip2::read::BzDecoder::new(reader);
    extract_tar_reader(decoder, output_dir, verbose)
}

fn extract_tar<R: Read>(reader: R, output_dir: &str, verbose: bool) -> Result<()> {
    extract_tar_reader(BufReader::new(reader), output_dir, verbose)
}

fn extract_tar_reader<R: Read>(reader: R, output_dir: &str, verbose: bool) -> Result<()> {
    let mut archive = tar::Archive::new(reader);

    for entry in archive.entries()? {
        let mut entry = entry?;

        let path = entry.path()?.into_owned();
        let entry_path = PathBuf::from(output_dir).join(&path);

        if let Some(parent) = entry_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if entry.header().entry_type() == tar::EntryType::Directory {
            if verbose {
                println!("Extracting: {}", path.display());
            }
            fs::create_dir_all(&entry_path)?;
        } else {
            if verbose {
                println!("Extracting: {}", path.display());
            }

            let mut file = File::create(&entry_path)?;
            io::copy(&mut entry, &mut file)?;

            // Preserve file permissions (Unix mode)
            if let Some(mode) = entry.header().mode().ok().filter(|&m| m != 0) {
                #[cfg(unix)]
                {
                    let permissions = PermissionsExt::from_mode(mode);
                    if let Err(e) = fs::set_permissions(&entry_path, permissions) {
                        eprintln!("Warning: Could not set permissions: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}

fn extract_zip<R: Read + Seek>(reader: R, output_dir: &str, verbose: bool) -> Result<()> {
    let mut archive = zip::ZipArchive::new(reader)?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;

        let entry_path = PathBuf::from(output_dir).join(entry.name());

        if let Some(parent) = entry_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        if entry.is_dir() {
            if verbose {
                println!("Extracting: {}", entry.name());
            }
            fs::create_dir_all(&entry_path)?;
        } else {
            if verbose {
                println!("Extracting: {}", entry.name());
            }

            let mut file = File::create(&entry_path)?;
            io::copy(&mut entry, &mut file)?;
        }
    }

    Ok(())
}
