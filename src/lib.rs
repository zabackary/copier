use std::{
    error::Error,
    fmt::Write,
    fs::{self, File},
    io::{self, BufRead},
    path::Path,
};

use indicatif::{ProgressBar, ProgressState, ProgressStyle};

fn copy_dir_all<T, U>(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    ignore_checker: &mut T,
    bytes_callback: &mut U,
    should_copy: bool,
) -> io::Result<()>
where
    T: FnMut(&Path) -> io::Result<bool>,
    U: FnMut(u64) -> (),
{
    if should_copy {
        fs::create_dir_all(&dst)?;
    }
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() && ignore_checker(&entry.path())? {
            // Pass over directory
        } else if ty.is_dir() {
            copy_dir_all(
                entry.path(),
                dst.as_ref().join(entry.file_name()),
                ignore_checker,
                bytes_callback,
                should_copy,
            )?;
        } else {
            bytes_callback(entry.metadata()?.len());
            if should_copy {
                if let Err(err) = fs::copy(entry.path(), dst.as_ref().join(entry.file_name())) {
                    eprintln!(
                        "[err] Failed to copy {}",
                        entry.path().to_str().expect("Failed to print msg")
                    );
                    return Err(err);
                }
            }
        }
    }
    Ok(())
}

fn should_ignore_dir(ignores: &Vec<&str>, path: impl AsRef<Path>) -> io::Result<bool> {
    if ignores.contains(
        &path
            .as_ref()
            .file_name()
            .expect("path is directory")
            .to_str()
            .expect("path is utf good"),
    ) {
        Ok(true)
    } else {
        for item in ignores {
            if item.starts_with("/") && path.as_ref().join(Path::new(&item[1..])).exists() {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn parse_ignore_list(path: impl AsRef<Path>) -> io::Result<Vec<String>> {
    let mut list = vec![];
    for line in io::BufReader::new(File::open(path)?).lines() {
        if let Ok(line) = line {
            if !line.starts_with("#") && line != "" {
                list.push(line.clone());
            }
        }
    }
    Ok(list)
}

pub struct Config<'a> {
    pub source: &'a Path,
    pub target: &'a Path,
    pub ignore_path: Option<&'a Path>,
}

impl<'a> Config<'a> {
    pub fn new(args: &[String]) -> Config {
        let source = Path::new(&args[1]);
        let target = Path::new(&args[2]);
        let ignore_path = if args.len() > 3 {
            Some(Path::new(&args[3]))
        } else {
            None
        };

        Config {
            source,
            target,
            ignore_path,
        }
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    let ignores = if let Some(ignore_path) = config.ignore_path {
        parse_ignore_list(ignore_path)?
    } else {
        vec![]
    };
    println!("Ignoring {} directories", ignores.len());
    println!("Discovering files...");
    let mut total_size = 0u64;
    let mut total_files = 0u64;
    copy_dir_all(
        config.source,
        config.target,
        &mut |path| {
            Ok(should_ignore_dir(
                &ignores.iter().map(AsRef::as_ref).collect(),
                path,
            )?)
        },
        &mut |size| {
            total_size += size;
            total_files += 1;
        },
        false,
    )?;
    println!(
        "Discovered {} files with a total size of {} bytes",
        total_files, total_size
    );
    let mut finished_size = 0u64;
    let progress_bar = ProgressBar::new(total_size);
    progress_bar.set_style(
      ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})"
      ).unwrap()
      .with_key("eta", |state: &ProgressState, w: &mut dyn Write| write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap())
      .progress_chars("█>-")
    );
    copy_dir_all(
        config.source,
        config.target,
        &mut |path| {
            Ok(
                if should_ignore_dir(&ignores.iter().map(AsRef::as_ref).collect(), path)? {
                    progress_bar.suspend(|| {
                        println!("Ignoring {}", path.to_str().expect("Failed to print msg"));
                    });

                    true
                } else {
                    false
                },
            )
        },
        &mut |size| {
            finished_size += size;
            progress_bar.set_position(finished_size);
        },
        true,
    )?;
    progress_bar.finish_and_clear();
    Ok(())
}
