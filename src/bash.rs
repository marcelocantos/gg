use std::path::Path;

use anyhow::Result;

pub fn bash(
    _command: Option<&str>,
    _prefix: Option<&str>,
    _exepath: &Path,
    _ggroot: &Path,
) -> Result<()> {
    eprintln!("bash not supported yet");
    Ok(())
}
