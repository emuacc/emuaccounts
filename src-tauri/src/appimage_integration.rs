//! Desktop integration for the Linux AppImage (TASK-140).
//!
//! On Wayland a GTK3 window sends no icon of its own. Hyprland, Waybar, GNOME
//! Shell and KDE find a window's icon through the installed desktop entry
//! named after its app_id, which GTK takes from the program name: the
//! binary's file name, `emuaccounts`. The .deb installs that entry; a
//! downloaded AppImage installs nothing, so its window showed the generic
//! cog. Run from an AppImage, the app installs the entry and the icons the
//! AppImage carries into the user's data folder before its window opens. It
//! rewrites only the files that differ, so a relaunch writes nothing and a
//! moved or updated AppImage refreshes the entry or the icons.

use std::ffi::OsString;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// The process is the app inside a mounted AppImage.
#[derive(Debug)]
pub struct AppImageLaunch {
    /// The AppImage file, where the desktop entry starts the app from.
    appimage: PathBuf,
    /// The AppImage's mounted contents.
    appdir: PathBuf,
    /// The binary's file name: GTK's app_id and the bundled icons' name.
    app_id: String,
}

impl AppImageLaunch {
    /// The AppImage runtime sets `APPIMAGE` (the file) and `APPDIR` (its
    /// mount) for the app, and every process the app starts inherits them.
    /// So a .deb or dev build started from an AppImage's terminal sees them
    /// too; it is the AppImage's app only when its binary is inside `APPDIR`.
    pub fn detect(env: impl Fn(&str) -> Option<OsString>, exe: &Path) -> Option<Self> {
        let appimage = PathBuf::from(env("APPIMAGE")?);
        let appdir = PathBuf::from(env("APPDIR")?);
        let canonical = |path: &Path| fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        if !canonical(exe).starts_with(canonical(&appdir)) {
            return None;
        }
        let app_id = exe.file_name()?.to_str()?.to_string();
        Some(Self {
            appimage,
            appdir,
            app_id,
        })
    }

    /// Installs the desktop entry and the icons under `data_dir`, writing
    /// only the files that differ, and returns the files written.
    pub fn install(&self, product_name: &str, data_dir: &Path) -> io::Result<Vec<PathBuf>> {
        let mut written = Vec::new();

        let entry = data_dir.join(format!("applications/{}.desktop", self.app_id));
        if write_if_changed(&entry, self.desktop_entry(product_name)?.as_bytes())? {
            written.push(entry);
        }

        let hicolor = data_dir.join("icons/hicolor");
        let mut icon_written = false;
        for (carried, size_dir) in self.carried_icons()? {
            let icon = hicolor
                .join(size_dir)
                .join("apps")
                .join(format!("{}.png", self.app_id));
            if write_if_changed(&icon, &fs::read(&carried)?)? {
                written.push(icon);
                icon_written = true;
            }
        }
        if icon_written {
            // GTK ignores a hicolor icon-theme.cache older than its folder.
            // One built before these icons would otherwise hide them.
            fs::File::open(&hicolor)?.set_modified(SystemTime::now())?;
        }

        Ok(written)
    }

    fn desktop_entry(&self, product_name: &str) -> io::Result<String> {
        let appimage = self.appimage.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "the AppImage path is not UTF-8: {}",
                    self.appimage.display()
                ),
            )
        })?;
        let id = &self.app_id;
        Ok(format!(
            "[Desktop Entry]\n\
             # Installed by {name} from its AppImage; rewritten when the AppImage moves.\n\
             Type=Application\n\
             Name={name}\n\
             Exec={exec}\n\
             TryExec={try_exec}\n\
             Icon={id}\n\
             StartupWMClass={id}\n\
             Terminal=false\n",
            name = escape_value(product_name),
            exec = escape_value(&quote_exec_arg(appimage)),
            try_exec = escape_value(appimage),
        ))
    }

    /// The `<size>/apps/<app_id>.png` icons in the AppImage's hicolor theme,
    /// with their size folders, e.g. `128x128`.
    fn carried_icons(&self) -> io::Result<Vec<(PathBuf, OsString)>> {
        let hicolor = self.appdir.join("usr/share/icons/hicolor");
        let mut icons = Vec::new();
        for size_dir in fs::read_dir(hicolor)? {
            let size_dir = size_dir?;
            let icon = size_dir
                .path()
                .join("apps")
                .join(format!("{}.png", self.app_id));
            if icon.is_file() {
                icons.push((icon, size_dir.file_name()));
            }
        }
        icons.sort();
        Ok(icons)
    }
}

/// `$XDG_DATA_HOME`, or `$HOME/.local/share` when it is unset or relative
/// (the XDG Base Directory spec ignores a relative one).
pub fn user_data_dir(env: impl Fn(&str) -> Option<OsString>) -> Option<PathBuf> {
    env("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|dir| dir.is_absolute())
        .or_else(|| env("HOME").map(|home| PathBuf::from(home).join(".local/share")))
}

/// Run from an AppImage, installs its desktop entry and icons for the user.
/// A failure is logged and never stops the app from starting.
pub fn install_from_env(product_name: &str) {
    let env = |key: &str| std::env::var_os(key);
    let Some(launch) = std::env::current_exe()
        .ok()
        .and_then(|exe| AppImageLaunch::detect(env, &exe))
    else {
        return;
    };
    let Some(data_dir) = user_data_dir(env) else {
        eprintln!("AppImage desktop integration skipped: neither XDG_DATA_HOME nor HOME is set");
        return;
    };
    match launch.install(product_name, &data_dir) {
        Ok(written) => {
            for path in written {
                eprintln!("AppImage desktop integration wrote {}", path.display());
            }
        }
        Err(e) => eprintln!("AppImage desktop integration failed: {e}"),
    }
}

/// Writes `contents` to `path` unless it already holds them, through a
/// temporary file so a reader never sees half a file. Returns whether it wrote.
fn write_if_changed(path: &Path, contents: &[u8]) -> io::Result<bool> {
    if fs::read(path).is_ok_and(|current| current == contents) {
        return Ok(false);
    }
    let dir = path.parent().expect("a file under the data folder");
    fs::create_dir_all(dir)?;
    let file_name = path.file_name().expect("a file name").to_string_lossy();
    let temp = dir.join(format!(".{file_name}.tmp"));
    fs::write(&temp, contents)?;
    fs::rename(&temp, path)?;
    Ok(true)
}

/// Quotes one argument of a desktop entry's `Exec` key: in double quotes,
/// with `"`, `` ` ``, `$` and `\` escaped by a backslash, and `%` doubled so
/// it is not read as a field code (Desktop Entry spec, "The Exec key").
fn quote_exec_arg(arg: &str) -> String {
    let mut quoted = String::from("\"");
    for c in arg.chars() {
        match c {
            '"' | '`' | '$' | '\\' => {
                quoted.push('\\');
                quoted.push(c);
            }
            '%' => quoted.push_str("%%"),
            _ => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

/// Escapes a desktop entry string value: `\\`, `\n`, `\t` and `\r` (Desktop
/// Entry spec, "Possible value types"). A reader undoes this before it
/// splits `Exec` into arguments, so a quoted backslash ends up as four.
fn escape_value(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\t' => escaped.push_str("\\t"),
            '\r' => escaped.push_str("\\r"),
            _ => escaped.push(c),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exec_quotes_spaces_and_escapes_reserved_characters() {
        assert_eq!(
            quote_exec_arg("/home/r/My Apps/Emu`$x`\"100%\".AppImage"),
            r#""/home/r/My Apps/Emu\`\$x\`\"100%%\".AppImage""#
        );
    }

    #[test]
    fn a_backslash_in_exec_is_escaped_for_the_quote_and_again_for_the_value() {
        assert_eq!(escape_value(&quote_exec_arg(r"/a\b")), r#""/a\\\\b""#);
    }

    #[test]
    fn string_values_escape_control_characters() {
        assert_eq!(escape_value("a\nb\tc\rd\\e"), r"a\nb\tc\rd\\e");
    }

    #[test]
    fn a_relative_xdg_data_home_falls_back_to_home() {
        let env = |key: &str| match key {
            "XDG_DATA_HOME" => Some(OsString::from("relative/data")),
            "HOME" => Some(OsString::from("/home/r")),
            _ => None,
        };
        assert_eq!(
            user_data_dir(env),
            Some(PathBuf::from("/home/r/.local/share"))
        );
    }

    #[test]
    fn no_home_and_no_xdg_data_home_is_no_data_dir() {
        assert_eq!(user_data_dir(|_: &str| None), None);
    }
}
