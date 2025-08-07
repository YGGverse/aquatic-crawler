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
        // make sure given path is valid and exist
        if !root.canonicalize()?.is_dir() {
            bail!("Preload root is not directory")
        }
        Ok(Self {
            max_filecount,
            max_filesize,
            regex,
            root,
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
        validate_info_hash(info_hash)?;
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
            log::debug!("clean tmp data `{}`", tmp.to_string_lossy())
        }
        Ok(())
    }

    // Getters

    /// Get absolute path to the temporary directory
    /// * optionally creates directory if not exists
    pub fn tmp(&self, info_hash: &str, is_create: bool) -> Result<PathBuf> {
        validate_info_hash(info_hash)?;
        let mut p = PathBuf::from(&self.root);
        p.push(tmp_component(info_hash));
        if p.is_file() {
            bail!("Output directory `{}` is file", p.to_string_lossy())
        }
        if is_create && !p.exists() {
            fs::create_dir(&p)?
        }
        Ok(p)
    }

    /// Get root location for `Self`
    pub fn root(&self) -> &PathBuf {
        &self.root
    }

    /// Check the given hash is contain resolved torrent file
    pub fn contains_torrent(&self, info_hash: &str) -> Result<bool> {
        validate_info_hash(info_hash)?;
        Ok(fs::exists(self.torrent(info_hash))?)
    }

    /// Get absolute path to the torrent file
    fn torrent(&self, info_hash: &str) -> PathBuf {
        let mut p = PathBuf::from(&self.root);
        p.push(format!("{info_hash}.torrent"));
        p
    }
}

/// Non-expensive method to make sure the given string is safe to use in path builders
/// @TODO implement custom type?
fn validate_info_hash(value: &str) -> Result<()> {
    if value.len() == 40 && value.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(())
    } else {
        bail!("Invalid info-hash value `{value}`")
    }
}

/// Build constant path component
fn tmp_component(info_hash: &str) -> String {
    format!(".{info_hash}")
}
