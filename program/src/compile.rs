//! The stage after writing: ReSkate Studio's `compile-map` of a written map into a mod's Patch folder, with the
//! compile choices the program states ([`crate::settings::studio_compile`]). A visible stage of its own: nothing in it
//! adapts the map.
use std::path::{Path, PathBuf};
use std::process::Command;

/// Where Studio is and which game it compiles against.
pub struct Studio {
    /// `reskate_cli.exe`.
    pub cli: PathBuf,
    /// The skate. install Studio reads the stock data from (read only).
    pub game: PathBuf,
}

/// What a compile left: the Patch folder and its log.
pub struct Compiled {
    pub patch: PathBuf,
    pub log: PathBuf,
    pub seconds: f64,
    pub warnings: usize,
}

/// Compile `map` into `stage` (emptied first) as the mod folder `mod_folder`; the log goes beside it.
pub fn compile(
    studio: &Studio,
    map: &Path,
    stage: &Path,
    mod_folder: &str,
) -> Result<Compiled, String> {
    if stage.exists() {
        std::fs::remove_dir_all(stage).map_err(|e| format!("{}: {e}", stage.display()))?;
    }
    std::fs::create_dir_all(stage).map_err(|e| format!("{}: {e}", stage.display()))?;
    let log = stage.with_extension("compile.log");
    let started = std::time::Instant::now();
    let out = Command::new(&studio.cli)
        .arg("compile-map")
        .arg(&studio.game)
        .arg(map)
        .arg(stage)
        .args(["--mod-folder", mod_folder])
        .args(crate::settings::studio_compile())
        .output()
        .map_err(|e| format!("{}: {e}", studio.cli.display()))?;
    let text = [out.stdout, out.stderr].concat();
    std::fs::write(&log, &text).map_err(|e| format!("{}: {e}", log.display()))?;
    let text = String::from_utf8_lossy(&text);
    if !out.status.success() {
        return Err(format!(
            "Studio compile failed ({}), see {}",
            out.status,
            log.display()
        ));
    }
    Ok(Compiled {
        patch: stage.join("Patch"),
        log,
        seconds: started.elapsed().as_secs_f64(),
        warnings: text.lines().filter(|l| l.starts_with("warning:")).count(),
    })
}
