use super::umi_errors::RuntimeErrors;
use anyhow::{anyhow, Result};
use dialoguer::Confirm;
use std::{fs, path::PathBuf, time::Instant};

pub fn timedrun<F, R>(msg: &str, func: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let measure = func();
    println!("{msg} after {:.1} seconds", start.elapsed().as_secs_f32());
    measure
}

pub fn check_outputpath(path: PathBuf) -> Result<PathBuf> {
    let exists = fs::metadata(&path).is_ok();

    if exists {
        if Confirm::new()
            .with_prompt(format!("{} exists. Overwrite?", path.display()))
            .interact()?
        {
            println!("File will be overwritten.");
            return Ok(path);
        } else {
            return Err(anyhow!(RuntimeErrors::FileExistsError));
        }
    } else {
        return Ok(path);
    }
}
