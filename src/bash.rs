use std::io::Write;
use std::path::Path;

use anyhow::Result;

pub fn bash(
    _command: Option<&str>,
    _prefix: Option<&str>,
    _exepath: &Path,
    _ggroot: &Path,
    _out: &mut Box<dyn Write>,
) -> Result<()> {
    eprintln!("bash not supported yet");
    Ok(())
}
