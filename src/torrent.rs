use anyhow::Result;
use std::{fs, io::Write, path::PathBuf, str::FromStr};

pub struct Torrent {
    storage: PathBuf,
}

impl Torrent {
    pub fn init(path: &str) -> Result<Self> {
        Ok(Self {
            storage: PathBuf::from_str(path)?.canonicalize()?,
        })
    }

    pub fn persist(&self, infohash: &str, data: &[u8]) -> Result<Option<PathBuf>> {
        Ok(if self.path(infohash).exists() {
            None
        } else {
            let p = self.path(infohash);
            let mut f = fs::File::create(&p)?;
            f.write_all(data)?;
            Some(p)
        })
    }

    fn path(&self, infohash: &str) -> PathBuf {
        let mut p = PathBuf::new();
        p.push(&self.storage);
        p.push(format!("{infohash}.torrent"));
        p
    }
}
