//! Step definitions for the AppImage desktop integration scenarios
//! (features/generated/appimage_desktop_integration.feature, TASK-140). A
//! module of the bdd test: tests/bdd.rs declares it with `#[path]`, and its
//! World keeps these steps' state in `LedgerWorld::appimage`.

use super::*;
use emuaccounts::appimage_integration::{self, AppImageLaunch};
use std::collections::{BTreeMap, HashMap};
use std::ffi::OsString;
use std::fs;
use std::time::{Duration, SystemTime};

#[derive(Debug, Default)]
pub struct AppImageWorld {
    /// Holds `home/` (the user's home folder) and `tmp/.mount_EmuAcc/` (the
    /// mounted AppImage).
    root: Option<tempfile::TempDir>,
    product_name: String,
    /// The environment EmuAcc starts with.
    env: HashMap<String, OsString>,
    /// The running binary: inside the AppImage's mount unless a step moves it.
    exe: PathBuf,
    /// Whether the last start was an AppImage launch.
    launched: Option<bool>,
    /// The files the last start wrote.
    written: Vec<PathBuf>,
}

impl AppImageWorld {
    fn root(&self) -> &Path {
        self.root
            .as_ref()
            .expect("the Background sets up an AppImage")
            .path()
    }

    fn home(&self) -> PathBuf {
        self.root().join("home")
    }

    fn appdir(&self) -> PathBuf {
        self.root().join("tmp/.mount_EmuAcc")
    }

    /// `{home}` in a feature stands for the scenario's home folder.
    fn expand(&self, text: &str) -> String {
        text.replace("{home}", &self.home().to_string_lossy())
    }

    fn start(&mut self) {
        let env = |key: &str| self.env.get(key).cloned();
        let written = AppImageLaunch::detect(env, &self.exe).map(|launch| {
            let data_dir =
                appimage_integration::user_data_dir(env).expect("HOME is set, so is the data dir");
            launch
                .install(&self.product_name, &data_dir)
                .expect("the desktop entry and icons install")
        });
        self.launched = Some(written.is_some());
        self.written = written.unwrap_or_default();
    }

    /// The `[Desktop Entry]` keys of the entry at `rel` under the home folder.
    fn desktop_entry(&self, rel: &str) -> BTreeMap<String, String> {
        let path = self.home().join(rel);
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("no desktop entry at {}: {e}", path.display()));
        let mut lines = text
            .lines()
            .filter(|l| !l.starts_with('#') && !l.is_empty());
        assert_eq!(lines.next(), Some("[Desktop Entry]"), "{text}");
        lines
            .map(|line| {
                let (key, value) = line
                    .split_once('=')
                    .unwrap_or_else(|| panic!("not a key: {line}"));
                (key.to_string(), value.to_string())
            })
            .collect()
    }
}

fn modified(path: &Path) -> SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn set_modified(path: &Path, time: SystemTime) {
    fs::File::open(path)
        .and_then(|f| f.set_modified(time))
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
}

#[given(expr = "an AppImage of {string} at {string} carrying the emu icons:")]
fn an_appimage(
    world: &mut LedgerWorld,
    product_name: String,
    path: String,
    step: &cucumber::gherkin::Step,
) {
    let w = &mut world.appimage;
    w.root = Some(tempfile::tempdir().expect("tempdir"));
    w.product_name = product_name;
    let (home, appdir) = (w.home(), w.appdir());

    let appimage = home.join(&path);
    fs::create_dir_all(appimage.parent().unwrap()).unwrap();
    fs::write(&appimage, "an AppImage").unwrap();

    w.exe = appdir.join("usr/bin/emuaccounts");
    fs::create_dir_all(w.exe.parent().unwrap()).unwrap();
    fs::write(&w.exe, "the app").unwrap();
    let table = step.table.as_ref().expect("a table of icon sizes");
    for row in &table.rows {
        let icon = appdir.join(format!(
            "usr/share/icons/hicolor/{}/apps/emuaccounts.png",
            row[0]
        ));
        fs::create_dir_all(icon.parent().unwrap()).unwrap();
        fs::write(&icon, format!("the {} emu", row[0])).unwrap();
    }

    w.env = HashMap::from([
        ("HOME".to_string(), home.into_os_string()),
        ("APPIMAGE".to_string(), appimage.into_os_string()),
        ("APPDIR".to_string(), appdir.into_os_string()),
    ]);
}

#[given("EmuAcc started")]
#[when("EmuAcc starts")]
fn emuacc_starts(world: &mut LedgerWorld) {
    world.appimage.start();
}

#[when(expr = "the AppImage moves to {string}")]
fn appimage_moves(world: &mut LedgerWorld, path: String) {
    let w = &mut world.appimage;
    let to = w.home().join(path);
    fs::create_dir_all(to.parent().unwrap()).unwrap();
    fs::rename(&w.env["APPIMAGE"], &to).unwrap();
    w.env.insert("APPIMAGE".to_string(), to.into_os_string());
}

#[when(expr = "the AppImage's {string} icon changes")]
fn appimage_icon_changes(world: &mut LedgerWorld, size: String) {
    let icon = world.appimage.appdir().join(format!(
        "usr/share/icons/hicolor/{size}/apps/emuaccounts.png"
    ));
    fs::write(icon, format!("the {size} emu, redrawn")).unwrap();
}

#[given(expr = "{string} has an icon-theme cache from an hour ago")]
fn icon_theme_cache(world: &mut LedgerWorld, rel: String) {
    let dir = world.appimage.home().join(rel);
    fs::create_dir_all(&dir).unwrap();
    let cache = dir.join("icon-theme.cache");
    fs::write(&cache, "a cache without the emu").unwrap();
    let now = SystemTime::now();
    set_modified(&cache, now - Duration::from_secs(3600));
    // Current until something changes the folder.
    set_modified(&dir, now - Duration::from_secs(7200));
}

#[given(expr = "XDG_DATA_HOME is {string}")]
fn xdg_data_home(world: &mut LedgerWorld, path: String) {
    let w = &mut world.appimage;
    let path = w.expand(&path);
    w.env.insert("XDG_DATA_HOME".to_string(), path.into());
}

#[given("APPIMAGE is not set")]
fn appimage_not_set(world: &mut LedgerWorld) {
    world.appimage.env.remove("APPIMAGE");
}

#[given("EmuAcc's binary is outside the AppImage's mount")]
fn binary_outside_mount(world: &mut LedgerWorld) {
    let w = &mut world.appimage;
    w.exe = w.root().join("usr/bin/emuaccounts");
    fs::create_dir_all(w.exe.parent().unwrap()).unwrap();
    fs::write(&w.exe, "the .deb's app").unwrap();
}

#[then(expr = "the desktop entry {string} reads:")]
fn desktop_entry_reads(world: &mut LedgerWorld, rel: String, step: &cucumber::gherkin::Step) {
    let w = &world.appimage;
    let table = step.table.as_ref().expect("a table of keys and values");
    let expected: BTreeMap<String, String> = table
        .rows
        .iter()
        .map(|row| (row[0].clone(), w.expand(&row[1])))
        .collect();
    assert_eq!(w.desktop_entry(&rel), expected);
}

#[then(expr = "the desktop entry {string} runs {string}")]
fn desktop_entry_runs(world: &mut LedgerWorld, rel: String, appimage: String) {
    let w = &world.appimage;
    let entry = w.desktop_entry(&rel);
    let expected = format!("\"{}\"", w.expand(&appimage));
    assert_eq!(entry.get("Exec"), Some(&expected));
}

#[then("these icons are the AppImage's:")]
fn icons_are_the_appimages(world: &mut LedgerWorld, step: &cucumber::gherkin::Step) {
    let w = &world.appimage;
    let table = step.table.as_ref().expect("a table of icon paths");
    for row in &table.rows {
        let installed = w.home().join(&row[0]);
        let in_share = row[0]
            .strip_prefix(".local/share/")
            .expect("an icon under .local/share");
        let carried = w.appdir().join("usr/share").join(in_share);
        assert_eq!(
            fs::read(&installed).unwrap_or_else(|e| panic!("{}: {e}", installed.display())),
            fs::read(&carried).unwrap(),
            "{}",
            row[0]
        );
    }
}

#[then("nothing is written")]
fn nothing_written(world: &mut LedgerWorld) {
    assert_eq!(world.appimage.written, Vec::<PathBuf>::new());
}

#[then("only these files are written:")]
fn only_these_written(world: &mut LedgerWorld, step: &cucumber::gherkin::Step) {
    let w = &world.appimage;
    let table = step.table.as_ref().expect("a table of paths");
    let mut expected: Vec<PathBuf> = table
        .rows
        .iter()
        .map(|row| w.home().join(&row[0]))
        .collect();
    expected.sort();
    let mut written = w.written.clone();
    written.sort();
    assert_eq!(written, expected);
}

#[then(expr = "{string} is newer than its icon-theme cache")]
fn newer_than_cache(world: &mut LedgerWorld, rel: String) {
    let dir = world.appimage.home().join(rel);
    assert!(modified(&dir) > modified(&dir.join("icon-theme.cache")));
}

#[then("it is not an AppImage launch")]
fn not_an_appimage_launch(world: &mut LedgerWorld) {
    assert_eq!(world.appimage.launched, Some(false));
}

#[then(expr = "{string} does not exist")]
fn does_not_exist(world: &mut LedgerWorld, rel: String) {
    let path = world.appimage.home().join(rel);
    assert!(!path.exists(), "{} exists", path.display());
}

#[then(expr = "the first PNG in the bundle's icons is {int}x{int}")]
fn first_png_icon(_world: &mut LedgerWorld, width: u32, height: u32) {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let conf: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(manifest_dir.join("tauri.conf.json")).expect("tauri.conf.json"),
    )
    .expect("tauri.conf.json is JSON");
    let first_png = conf["bundle"]["icon"]
        .as_array()
        .expect("bundle.icon is a list")
        .iter()
        .filter_map(|icon| icon.as_str())
        .find(|icon| icon.ends_with(".png"))
        .expect("bundle.icon lists a PNG");
    let png = fs::read(manifest_dir.join(first_png)).expect("the PNG");
    // The IHDR chunk follows the 8-byte signature: length, type, width, height.
    assert_eq!(&png[12..16], b"IHDR", "{first_png} is not a PNG");
    let size = |at: usize| u32::from_be_bytes(png[at..at + 4].try_into().unwrap());
    assert_eq!((size(16), size(20)), (width, height), "{first_png}");
}
