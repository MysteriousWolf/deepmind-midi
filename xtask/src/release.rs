//! Version bumping for the release workflow.
//!
//! The workspace `Cargo.toml` is the single source of truth for the version.
//! Versions are calendar-flavoured: `YY.RELEASE.PATCH`, where `YY` is the
//! two-digit year, `RELEASE` counts releases within that year from 1, and
//! `PATCH` counts patches to that release from 0. So `26.1.0` is the first
//! release of 2026 and `26.1.1` its first patch.

use std::path::Path;

/// What to advance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bump {
    /// Start a new release: `26.1.3` becomes `26.2.0`, or `27.1.0` in a new year.
    Release,
    /// Patch the current release: `26.1.3` becomes `26.1.4`.
    Patch,
}

/// A parsed `YY.RELEASE.PATCH` version.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version {
    /// Two-digit year.
    pub year: u16,
    /// Release number within the year, counting from 1.
    pub release: u16,
    /// Patch number within the release, counting from 0.
    pub patch: u16,
}

impl Version {
    /// Parses `YY.RELEASE.PATCH`.
    ///
    /// # Errors
    ///
    /// Returns a message when the string is not three dot-separated numbers.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut parts = text.trim().split('.');
        let mut next = |what: &str| -> Result<u16, String> {
            parts
                .next()
                .ok_or_else(|| format!("version {text:?} has no {what}"))?
                .parse()
                .map_err(|e| format!("version {text:?} has a bad {what}: {e}"))
        };
        let version = Self {
            year: next("year")?,
            release: next("release")?,
            patch: next("patch")?,
        };
        if parts.next().is_some() {
            return Err(format!("version {text:?} has more than three components"));
        }
        Ok(version)
    }

    /// Returns the version that follows this one for the given bump and year.
    #[must_use]
    pub const fn next(self, bump: Bump, year: u16) -> Self {
        match bump {
            Bump::Patch => Self {
                patch: self.patch + 1,
                ..self
            },
            Bump::Release if year == self.year => Self {
                year,
                release: self.release + 1,
                patch: 0,
            },
            Bump::Release => Self {
                year,
                release: 1,
                patch: 0,
            },
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.year, self.release, self.patch)
    }
}

/// Reads the workspace version from the root `Cargo.toml`.
///
/// # Errors
///
/// Returns a message when the file cannot be read or has no
/// `[workspace.package] version` key.
pub fn current(root: &Path) -> Result<Version, String> {
    let path = root.join("Cargo.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let raw = version_line(&text)
        .ok_or_else(|| format!("{}: no [workspace.package] version", path.display()))?;
    Version::parse(raw)
}

/// Writes `version` into the root `Cargo.toml`, replacing the existing value.
///
/// # Errors
///
/// Returns a message when the file cannot be read or written, or has no
/// `[workspace.package] version` key.
pub fn write(root: &Path, version: Version) -> Result<(), String> {
    let path = root.join("Cargo.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let raw = version_line(&text)
        .ok_or_else(|| format!("{}: no [workspace.package] version", path.display()))?;
    let old = format!("version = \"{raw}\"");
    let new = format!("version = \"{version}\"");
    let updated = text.replacen(&old, &new, 1);
    if updated == text {
        return Err(format!("{}: version line did not change", path.display()));
    }
    std::fs::write(&path, updated).map_err(|e| format!("{}: {e}", path.display()))
}

/// Extracts the quoted value of the first `version = "..."` line.
fn version_line(text: &str) -> Option<&str> {
    text.lines()
        .map(str::trim)
        .find_map(|line| line.strip_prefix("version = \"")?.strip_suffix('"'))
}

/// Returns the current two-digit year from the system clock (UTC).
#[must_use]
pub fn current_year() -> u16 {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    year_from_unix_days(i64::try_from(secs / 86_400).unwrap_or(0)) % 100
}

/// Civil year for a day count since 1970-01-01, by Howard Hinnant's algorithm.
fn year_from_unix_days(days: i64) -> u16 {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let year = yoe + era * 400 + i64::from(mp >= 10);
    u16::try_from(year).unwrap_or(0)
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "a failed expectation is the test failure"
)]
mod tests {
    use super::*;

    #[test]
    fn patch_bumps_only_the_patch_component() {
        let v = Version::parse("26.1.3").expect("valid version");
        assert_eq!(v.next(Bump::Patch, 26).to_string(), "26.1.4");
    }

    #[test]
    fn a_release_in_the_same_year_advances_the_release_counter() {
        let v = Version::parse("26.1.3").expect("valid version");
        assert_eq!(v.next(Bump::Release, 26).to_string(), "26.2.0");
    }

    #[test]
    fn a_release_in_a_new_year_restarts_the_release_counter() {
        let v = Version::parse("26.4.2").expect("valid version");
        assert_eq!(v.next(Bump::Release, 27).to_string(), "27.1.0");
        // A patch in a new year still belongs to the old release.
        assert_eq!(v.next(Bump::Patch, 27).to_string(), "26.4.3");
    }

    #[test]
    fn parsing_rejects_malformed_versions() {
        assert!(Version::parse("26.1").is_err());
        assert!(Version::parse("26.1.0.1").is_err());
        assert!(Version::parse("twenty.six.one").is_err());
    }

    #[test]
    fn unix_days_convert_to_civil_years() {
        assert_eq!(year_from_unix_days(0), 1970);
        assert_eq!(year_from_unix_days(365), 1971);
        // 2000-01-01 and 2024-02-29, the leap-day case.
        assert_eq!(year_from_unix_days(10_957), 2000);
        assert_eq!(year_from_unix_days(19_782), 2024);
    }
}
