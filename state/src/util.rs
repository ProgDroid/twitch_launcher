use std::path::PathBuf;

// Resolve the app's data directory the same way dotman does: the platform config
// dir (BaseDirs::config_dir) plus the app name. On Windows this is
// %APPDATA%\twitch_launcher, on Linux ~/.config/twitch_launcher. This must match
// dotman's `{config}/twitch_launcher/...` manifest entries so the symlinked
// favourites.json and lists/ are found when the binary runs from anywhere
// (e.g. ~/.cargo/bin). Falls back to the current directory if no home dir exists.
#[must_use]
pub fn config_dir() -> PathBuf {
    directories::BaseDirs::new().map_or_else(
        || PathBuf::from("."),
        |base| base.config_dir().join("twitch_launcher"),
    )
}

pub const fn index_add(current_value: usize, size: usize) -> usize {
    match size {
        0 => current_value,
        _ => (current_value + 1) % size,
    }
}

pub const fn index_subtract(current_value: usize, size: usize) -> usize {
    match size {
        0 => current_value,
        _ => (current_value + size - 1) % size,
    }
}
