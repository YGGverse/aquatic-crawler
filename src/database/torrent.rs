use anyhow::{Result, bail};
use std::{fs, io::Write, path::PathBuf, str::FromStr};

pub struct Storage(PathBuf);

impl Storage {
    pub fn init(storage: &str, clear: bool) -> Result<Self> {
        let p = PathBuf::from_str(storage)?;
        if let Ok(t) = fs::metadata(&p) {
            if t.is_file() {
                bail!("Target destination is not directory!")
            }
            if t.is_dir() && clear {
                fs::remove_dir_all(&p)?;
            }
        }
        fs::create_dir_all(&p)?;
        Ok(Self(p))
    }

    pub fn exists(&self, infohash: &str) -> bool {
        fs::metadata(self.filename(infohash)).is_ok_and(|p| p.is_file())
    }

    pub fn save(&self, infohash: &str, bytes: &[u8]) -> Result<PathBuf> {
        let p = self.filename(infohash);
        let mut f = fs::File::create(&p)?;
        f.write_all(bytes)?;
        Ok(p)
    }

    fn filename(&self, infohash: &str) -> PathBuf {
        let mut p = PathBuf::new();
        p.push(&self.0);
        p.push(format!("{infohash}.torrent"));
        p
    }
}
