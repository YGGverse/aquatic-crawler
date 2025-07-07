use anyhow::{Result, bail};
use regex::Regex;
use std::{fs, path::PathBuf, str::FromStr};

pub struct Preload {
    path: PathBuf,
    pub max_filecount: Option<usize>,
    pub max_filesize: Option<u64>,
    pub total_size: Option<u64>,
    pub regex: Option<Regex>,
}

impl Preload {
    fn init(
        directory: &str,
        regex: Option<String>,
        max_filecount: Option<usize>,
        max_filesize: Option<u64>,
        total_size: Option<u64>,
        is_clear: bool,
    ) -> Result<Self> {
        let path = PathBuf::from_str(directory)?;
        if let Ok(t) = fs::metadata(&path) {
            if t.is_file() {
                bail!("Storage destination is not directory!");
            }
            if t.is_dir() && is_clear {
                for i in fs::read_dir(&path)? {
                    let r = i?.path();
                    if r.is_dir() {
                        fs::remove_dir_all(&r)?;
                    } else {
                        fs::remove_file(&r)?;
                    }
                }
            }
        }
        fs::create_dir_all(&path)?;
        Ok(Self {
            max_filecount,
            max_filesize,
            path,
            regex: regex.map(|r| Regex::new(&r).unwrap()),
            total_size,
        })
    }

    pub fn output_folder(&self, infohash: &str, create: bool) -> Result<String> {
        let mut p = PathBuf::new();
        p.push(&self.path);
        p.push(infohash);
        if p.is_file() {
            bail!("File destination is not directory!");
        }
        if create {
            fs::create_dir_all(&p)?;
        }
        if !p.is_dir() {
            bail!("Destination directory not exists!")
        }
        Ok(p.to_string_lossy().to_string())
    }

    pub fn absolute(&self, infohash: &str, file: &PathBuf) -> PathBuf {
        let mut p = PathBuf::new();
        p.push(&self.path);
        p.push(infohash);
        p.push(file);
        p
    }

    /// Recursively remove all files under the `infohash` location (see rqbit#408)
    pub fn cleanup(&self, infohash: &str, keep_filenames: Option<Vec<PathBuf>>) -> Result<()> {
        for e in walkdir::WalkDir::new(self.output_folder(infohash, false)?) {
            let e = e?;
            let p = e.into_path();
            if p.is_file() && keep_filenames.as_ref().is_none_or(|k| !k.contains(&p)) {
                fs::remove_file(p)?;
            }
        }
        Ok(())
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn matches(&self, pattern: &str) -> bool {
        self.regex.as_ref().is_some_and(|r| r.is_match(pattern))
    }
}

/// Init `Preload` with validate related argument options
pub fn init(
    path: Option<String>,
    regex: Option<String>,
    max_filecount: Option<usize>,
    max_filesize: Option<u64>,
    total_size: Option<u64>,
    is_clear: bool,
) -> Result<Option<Preload>> {
    Ok(match path {
        Some(ref p) => Some(Preload::init(
            p,
            regex,
            max_filecount,
            max_filesize,
            total_size,
            is_clear,
        )?),
        None => {
            if regex.is_some()
                || max_filecount.is_some()
                || max_filesize.is_some()
                || total_size.is_some()
                || is_clear
            {
                bail!("`--preload` directory is required for this configuration!")
            }
            None
        }
    })
}
