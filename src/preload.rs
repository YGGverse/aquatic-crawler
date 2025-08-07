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

    /// Persist torrent bytes and preloaded content,
    /// cleanup tmp data on success (see rqbit#408)
    pub fn commit(
        &self,
        info_hash: &str,
        torrent_bytes: Vec<u8>,
        persist_files: Option<HashSet<PathBuf>>,
    ) -> Result<()> {
        if !is_info_hash(info_hash) {
            bail!("Invalid info-hash `{info_hash}`")
        }
        // persist torrent bytes to file
        let t = self.torrent(info_hash);
        fs::write(&t, torrent_bytes)?;
        log::debug!("persist torrent bytes for `{}`", t.to_string_lossy());
        // persist preload files
        let mut d = PathBuf::from(&self.root);
        d.push(info_hash);
        if d.exists() {
            // clean previous data
            fs::remove_dir_all(&d)?;
            log::debug!("clean preload content `{}`", d.to_string_lossy())
        }
        if let Some(f) = persist_files {
            let r = d.components().count(); // count root offset once
            for p in f {
                // make sure preload path is referring to the expected location
                let o = p.canonicalize()?;
                if !o.starts_with(&self.root) || o.is_dir() {
                    bail!("Unexpected canonical path `{}`", o.to_string_lossy())
                }
                // build new permanent path /root/info-hash
                let mut n = PathBuf::from(&d);
                for component in o.components().skip(r) {
                    n.push(component)
                }
                // make sure segments count is same to continue
                if o.components().count() != n.components().count() {
                    bail!(
                        "Unexpected components count: `{}` > `{}`",
                        o.to_string_lossy(),
                        n.to_string_lossy(),
                    )
                }
                // move `persist_files` from temporary to permanent location
                fs::create_dir_all(n.parent().unwrap())?;
                fs::rename(&o, &n)?;
                log::debug!(
                    "persist tmp file `{}` to `{}`",
                    o.to_string_lossy(),
                    n.to_string_lossy()
                );
            }
        }
        // cleanup temporary data
        let tmp = self.tmp(info_hash, false)?;
        if tmp.exists() {
            fs::remove_dir_all(&tmp)?;
            log::debug!("clean tmp data `{}`", t.to_string_lossy())
        }
        Ok(())
    }

    // Getters

    /// * creates new temporary directory if not exists
    pub fn tmp(&self, info_hash: &str, is_create: bool) -> Result<PathBuf> {
        if !is_info_hash(info_hash) {
            bail!("Invalid info-hash `{info_hash}`")
        }
        let mut p = PathBuf::from(&self.root);
        p.push(tmp(info_hash));
        if p.is_file() {
            bail!("Output directory `{}` is file", p.to_string_lossy())
        }
        if is_create && !p.exists() {
            fs::create_dir(&p)?
        }
        Ok(p)
    }

    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    pub fn contains_torrent(&self, info_hash: &str) -> Result<bool> {
        if !is_info_hash(info_hash) {
            bail!("Invalid info-hash `{info_hash}`")
        }
        Ok(fs::exists(self.torrent(info_hash))?)
    }

    fn torrent(&self, info_hash: &str) -> PathBuf {
        let mut p = PathBuf::from(&self.root);
        p.push(format!("{info_hash}.torrent"));
        p
    }
}

fn is_info_hash(value: &str) -> bool {
    value.len() == 40 && value.chars().all(|c| c.is_ascii_hexdigit())
}

fn tmp(info_hash: &str) -> String {
    format!(".{info_hash}")
}
