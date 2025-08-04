use anyhow::{Result, bail};
use regex::Regex;
use std::{collections::HashSet, fs, path::PathBuf};

pub struct Preload {
    root: PathBuf,
    pub max_filecount: Option<usize>,
    pub max_filesize: Option<u64>,
    pub regex: Option<Regex>,
}

impl Preload {
    // Constructors

    pub fn init(
        root: PathBuf,
        regex: Option<Regex>,
        max_filecount: Option<usize>,
        max_filesize: Option<u64>,
    ) -> Result<Self> {
        if !root.is_dir() {
            bail!("Preload root is not directory")
        }
        Ok(Self {
            max_filecount,
            max_filesize,
            regex,
            root: root.canonicalize()?,
        })
    }

    // Actions

    /// Recursively remove all files under the `infohash` location (see rqbit#408)
    pub fn cleanup(&self, info_hash: &str, keep_filenames: Option<HashSet<PathBuf>>) -> Result<()> {
        for e in walkdir::WalkDir::new(self.output_folder(info_hash)?) {
            let e = e?;
            let p = e.into_path();
            if p.is_file() && keep_filenames.as_ref().is_none_or(|k| !k.contains(&p)) {
                fs::remove_file(p)?;
            }
        } // remove empty directories @TODO
        Ok(())
    }

    pub fn persist_torrent_bytes(&self, info_hash: &str, contents: &[u8]) -> Result<PathBuf> {
        let p = self.torrent(info_hash)?;
        fs::write(&p, contents)?;
        Ok(p)
    }

    // Getters

    /// * creates new directory if not exists
    pub fn output_folder(&self, info_hash: &str) -> Result<PathBuf> {
        if !is_info_hash(info_hash) {
            bail!("Invalid info-hash `{info_hash}`")
        }
        let mut p = PathBuf::from(&self.root);
        p.push(info_hash);
        if p.is_file() {
            bail!("Output directory for info-hash `{info_hash}` is file")
        }
        if !p.exists() {
            fs::create_dir(&p)?
        }
        Ok(p)
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn contains_torrent(&self, info_hash: &str) -> Result<bool> {
        Ok(fs::exists(self.torrent(info_hash)?)?)
    }

    fn torrent(&self, info_hash: &str) -> Result<PathBuf> {
        if !is_info_hash(info_hash) {
            bail!("Invalid info-hash `{info_hash}`")
        }
        let mut p = PathBuf::from(&self.root);
        p.push(format!("{info_hash}.torrent"));
        Ok(p)
    }
}

fn is_info_hash(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|c| c.is_ascii_hexdigit())
}
