use std::{
    error::Error,
    fs::{self, File},
    io::{self, BufRead},
    path::Path,
};

fn copy_dir_all<T>(
    src: impl AsRef<Path>,
    dst: impl AsRef<Path>,
    ignore_checker: &T,
) -> io::Result<()>
where
    T: Fn(&Path) -> io::Result<bool>,
{
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() && ignore_checker(&entry.path())? {
            println!(
                "Skipping {}",
                entry.path().to_str().expect("Failed to print msg")
            );
        } else if ty.is_dir() {
            copy_dir_all(
                entry.path(),
                dst.as_ref().join(entry.file_name()),
                ignore_checker,
            )?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
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
    println!("{:?}", ignores);
    Ok(copy_dir_all(config.source, config.target, &|path| {
        Ok(should_ignore_dir(
            &ignores.iter().map(AsRef::as_ref).collect(),
            path,
        )?)
    })?)
}
