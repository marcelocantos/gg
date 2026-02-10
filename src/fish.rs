use std::path::Path;

use anyhow::Result;

pub fn fish(
    _command: Option<&str>,
    _prefix: Option<&str>,
    _exepath: &Path,
    _ggroot: &Path,
) -> Result<()> {
    eprintln!("fish not supported yet");
    Ok(())
}
