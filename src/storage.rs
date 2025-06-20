use anyhow::{Result, bail};
use std::{fs, io::Write, path::PathBuf, str::FromStr};

pub struct Storage(PathBuf);

impl Storage {
    pub fn init(storage: &str, clear: bool) -> Result<Self> {
        let p = PathBuf::from_str(storage)?;
        if let Ok(t) = fs::metadata(&p) {
            if t.is_file() {
                bail!("Storage destination is not directory!");
            }
            if t.is_dir() && clear {
                for i in fs::read_dir(&p)? {
                    let r = i?.path();
                    if r.is_dir() {
                        fs::remove_dir_all(&r)?;
                    } else {
                        fs::remove_file(&r)?;
                    }
                }
            }
        }
        fs::create_dir_all(&p)?;
        Ok(Self(p))
    }

    pub fn torrent_exists(&self, infohash: &str) -> bool {
        fs::metadata(self.torrent(infohash))
            .is_ok_and(|p| p.is_file() || p.is_dir() || p.is_symlink())
    }

    pub fn save_torrent(&self, infohash: &str, bytes: &[u8]) -> Result<PathBuf> {
        let p = self.torrent(infohash);
        let mut f = fs::File::create(&p)?;
        f.write_all(bytes)?;
        Ok(p)
    }

    pub fn output_folder(&self, infohash: &str, create: bool) -> Result<String> {
        let mut p = PathBuf::new();
        p.push(&self.0);
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
        p.push(&self.0);
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
        self.0.clone()
    }

    fn torrent(&self, infohash: &str) -> PathBuf {
        let mut p = PathBuf::new();
        p.push(&self.0);
        p.push(format!("{infohash}.torrent"));
        p
    }
}
