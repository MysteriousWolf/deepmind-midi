//! Writes generated files, and records which checked-in copies are stale.
//!
//! Every generator writes through an [`Output`]: a file is rewritten only when
//! its contents differ, nothing is written at all in check mode, and the paths
//! that differed come back so the caller can name them.

use std::path::Path;

/// Where generated files go, and what has been found stale so far.
pub struct Output<'a> {
    root: &'a Path,
    check: bool,
    stale: Vec<String>,
}

impl<'a> Output<'a> {
    /// Starts an output rooted at the repository root.
    ///
    /// With `check` set nothing is written; staleness is only recorded.
    pub const fn new(root: &'a Path, check: bool) -> Self {
        Self {
            root,
            check,
            stale: Vec::new(),
        }
    }

    /// Returns the paths, relative to the root, whose checked-in copy differed.
    pub fn stale(self) -> Vec<String> {
        self.stale
    }

    /// Writes one file, unless the copy on disk already holds `contents`.
    ///
    /// # Errors
    ///
    /// Returns a message when the file or its directory cannot be written.
    pub fn write(&mut self, path: &str, contents: &str) -> Result<(), String> {
        let file = self.root.join(path);
        if std::fs::read_to_string(&file).is_ok_and(|found| found == contents) {
            return Ok(());
        }
        self.stale.push(path.to_owned());
        if self.check {
            return Ok(());
        }
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("{}: {e}", parent.display()))?;
        }
        std::fs::write(&file, contents).map_err(|e| format!("{}: {e}", file.display()))
    }

    /// Makes a directory hold exactly `files`, each a name and its contents.
    ///
    /// Each file is written as by [`Output::write`]. A file already in the
    /// directory with one of `extensions` and a name not in `files` is left
    /// behind by something renamed, and is removed; anything else there is left
    /// alone.
    ///
    /// # Errors
    ///
    /// Returns a message when a file cannot be written or removed.
    pub fn sync_dir<S: AsRef<str>>(
        &mut self,
        dir: &str,
        files: &[(String, S)],
        extensions: &[&str],
    ) -> Result<(), String> {
        for (name, contents) in files {
            self.write(&format!("{dir}/{name}"), contents.as_ref())?;
        }

        let Ok(entries) = std::fs::read_dir(self.root.join(dir)) else {
            return Ok(());
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let generated = path
                .extension()
                .is_some_and(|e| extensions.iter().any(|wanted| e == *wanted));
            if !generated || !path.is_file() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if files.iter().any(|(wanted, _)| *wanted == name) {
                continue;
            }
            self.stale.push(format!("{dir}/{name}"));
            if !self.check {
                std::fs::remove_file(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            }
        }
        Ok(())
    }
}
