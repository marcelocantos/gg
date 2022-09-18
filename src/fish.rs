use std::io::Write;
use std::path::Path;

use anyhow::Result;

pub fn fish(
    _arg1: Option<&str>,
    _arg2: Option<&str>,
    _exepath: &Path,
    _ggroot: &Path,
    _out: &mut Box<dyn Write>,
) -> Result<()> {
    eprintln!("fish not supported yet");
    Ok(())
}
