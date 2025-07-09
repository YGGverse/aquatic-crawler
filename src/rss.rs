use anyhow::{Result, bail};
use std::{collections::HashSet, fs::File, io::Write, path::PathBuf, str::FromStr};
use url::Url;

/// Export crawl index to the RSS file
pub struct Rss {
    file: File,
    target: PathBuf,
    tmp: PathBuf,
    trackers: Option<HashSet<Url>>,
}

impl Rss {
    /// Create writable file for given `filepath`
    pub fn new(
        filepath: &str,
        title: &str,
        link: &Option<String>,
        description: &Option<String>,
        trackers: Option<HashSet<Url>>,
    ) -> Result<Self> {
        // prevent from reading of the incomplete file
        let tmp = PathBuf::from_str(&format!("{filepath}.tmp"))?;

        // init public destination
        let target = PathBuf::from_str(filepath)?;
        if target.is_dir() {
            bail!("RSS path `{}` is directory", target.to_string_lossy())
        }
        // init temporary file to write
        let mut file = File::create(&tmp)?;
        file.write_all(
            b"<?xml version=\"1.0\" encoding=\"UTF-8\"?><rss version=\"2.0\"><channel><title>",
        )?;
        file.write_all(escape(title).as_bytes())?;
        file.write_all(b"</title>")?;

        if let Some(s) = link {
            file.write_all(b"<link>")?;
            file.write_all(escape(s).as_bytes())?;
            file.write_all(b"</link>")?
        }
        if let Some(s) = description {
            file.write_all(b"<description>")?;
            file.write_all(escape(s).as_bytes())?;
            file.write_all(b"</description>")?
        }

        Ok(Self {
            file,
            target,
            trackers,
            tmp,
        })
    }

    /// Append `item` to the feed `channel`
    pub fn push(
        &mut self,
        infohash: &str,
        title: &str,
        description: Option<String>,
        pub_date: Option<&str>,
    ) -> Result<()> {
        self.file.write_all(
            format!(
                "<item><guid>{infohash}</guid><title>{}</title><link>{}</link>",
                escape(title),
                escape(&crate::magnet(infohash, self.trackers.as_ref()))
            )
            .as_bytes(),
        )?;
        if let Some(s) = description {
            self.file.write_all(b"<description>")?;
            self.file.write_all(escape(&s).as_bytes())?;
            self.file.write_all(b"</description>")?
        }
        if let Some(s) = pub_date {
            self.file.write_all(b"<pubDate>")?;
            self.file.write_all(escape(s).as_bytes())?;
            self.file.write_all(b"</pubDate>")?
        }
        self.file.write_all(b"</item>")?;
        Ok(())
    }

    /// Write final bytes, replace public file with temporary one
    pub fn commit(mut self) -> Result<()> {
        self.file.write_all(b"</channel></rss>")?;
        std::fs::rename(self.tmp, self.target)?;
        Ok(())
    }
}

pub fn item_description(size: Option<u64>, list: Option<&Vec<(String, u64)>>) -> Option<String> {
    if size.is_none() && list.is_none() {
        return None;
    }
    let mut b = Vec::with_capacity(list.map(|l| l.len()).unwrap_or_default() + 1);
    if let Some(s) = size {
        use crate::format::Format;
        b.push(s.bytes())
    }
    if let Some(l) = list {
        for (path, size) in l {
            b.push(format!("* {path}: {size}"))
        }
    }
    Some(b.join("\n"))
}

fn escape(subject: &str) -> String {
    subject
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&apos;")
}
