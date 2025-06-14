use anyhow::{Result, bail};

#[derive(PartialEq)]
pub enum Level {
    Error,
    Info,
    Trace,
}

impl Level {
    pub fn parse(value: char) -> Result<Self> {
        match value {
            'e' => Ok(Self::Error),
            'i' => Ok(Self::Info),
            't' => {
                tracing_subscriber::fmt::init();
                Ok(Self::Trace)
            }
            _ => bail!("Unsupported debug value `{value}`!"),
        }
    }
}
