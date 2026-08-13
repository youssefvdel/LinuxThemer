//! Local system introspection + integration: installed-theme discovery,
//! current-theme snapshot, and launching external theme creators.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;

#[derive(Serialize)]
pub struct InstalledTheme {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub path: String,
}

fn home() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("/"))
}

fn list_dirs(root: &Path) -> Vec<PathBuf> {
    match fs::read_dir(root) {
        Ok(rd) => rd
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .filter(|p| {
                p.file_name()
                    .map(|n| !n.to_string_lossy().starts_with('.'))
                    .unwrap_or(false)
            })
            .collect(),
        Err(_) => vec![],
    }
}

fn has_entry(dir: &Path, name: &str) -> bool {
    dir.join(name).exists()
}

fn scan(kind: &str, roots: &[PathBuf], filter: impl Fn(&Path) -> bool) -> Vec<InstalledTheme> {
    let mut seen = HashSet::new();
    let mut out = vec![];
    for root in roots {
        for d in list_dirs(root) {
            if !filter(&d) {
                continue;
            }
            let name = d
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if name.is_empty() || !seen.insert(name.clone()) {
                continue;
            }
            out.push(InstalledTheme {
                id: format!("{kind}:{name}"),
                name,
                kind: kind.to_string(),
                path: d.to_string_lossy().to_string(),
            });
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

fn custom_dir() -> PathBuf {
    home().join(".local/share/linuxthemer/custom")
}

fn custom_themes() -> Vec<InstalledTheme> {
    let dir = custom_dir();
    let mut out = vec![];
    if let Ok(rd) = fs::read_dir(&dir) {
        for e in rd.filter_map(|e| e.ok()) {
            let p = e.path();
            if p.extension().map(|x| x == "json").unwrap_or(false) {
                let name = p
                    .file_stem()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                out.push(InstalledTheme {
                    id: format!("custom:{name}"),
                    name,
                    kind: "custom".to_string(),
                    path: p.to_string_lossy().to_string(),
                });
            }
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

/// Discover themes actually installed on this machine, across every location
/// (KDE Get New Stuff installs included), grouped by kind.
#[tauri::command]
pub fn list_installed() -> Result<Vec<InstalledTheme>, String> {
    let h = home();
    let mut all: Vec<InstalledTheme> = vec![];

    all.extend(scan(
        "global",
        &[h.join(".local/share/plasma/look-and-feel")],
        |_| true,
    ));
    all.extend(scan(
        "plasma",
        &[h.join(".local/share/plasma/desktoptheme")],
        |_| true,
    ));
    all.extend(scan(
        "decorations",
        &[h.join(".local/share/aurorae")],
        |_| true,
    ));
    all.extend(scan(
        "colors",
        &[
            h.join(".local/share/color-schemes"),
            PathBuf::from("/usr/share/color-schemes"),
        ],
        |_| true,
    ));
    all.extend(scan(
        "wallpapers",
        &[h.join(".local/share/wallpapers")],
        |_| true,
    ));
    all.extend(scan("kvantum", &[h.join(".config/Kvantum")], |_| true));
    all.extend(scan(
        "sddm",
        &[PathBuf::from("/usr/share/sddm/themes")],
        |_| true,
    ));
    all.extend(scan(
        "gtk",
        &[h.join(".themes"), PathBuf::from("/usr/share/themes")],
        |d| has_entry(d, "gtk-3.0") || has_entry(d, "gtk-4.0") || has_entry(d, "index.theme"),
    ));
    all.extend(scan(
        "icons",
        &[
            h.join(".icons"),
            h.join(".local/share/icons"),
            PathBuf::from("/usr/share/icons"),
        ],
        |d| has_entry(d, "index.theme"),
    ));
    all.extend(scan(
        "cursors",
        &[
            h.join(".icons"),
            h.join(".local/share/icons"),
            PathBuf::from("/usr/share/icons"),
        ],
        |d| has_entry(d, "cursors"),
    ));
    all.extend(custom_themes());

    Ok(all)
}

fn read_ini(path: &Path, section: &str, key: &str) -> Option<String> {
    let text = fs::read_to_string(path).ok()?;
    let mut cur: &str = "";
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            cur = &line[1..line.len() - 1];
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if cur == section && k.trim() == key {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

/// Snapshot the currently-applied theme (KDE + GTK settings) as a named
/// custom profile under ~/.local/share/linuxthemer/custom.
#[tauri::command]
pub fn save_current_theme(name: String) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("theme name is empty".into());
    }
    let h = home();
    let kdeglobals = h.join(".config/kdeglobals");
    let gtk3 = h.join(".config/gtk-3.0/settings.ini");

    let mut snap = serde_json::Map::new();
    snap.insert(
        "lookAndFeel".into(),
        serde_json::json!(read_ini(&kdeglobals, "KDE", "LookAndFeelPackage")),
    );
    snap.insert(
        "colorScheme".into(),
        serde_json::json!(read_ini(&kdeglobals, "General", "ColorScheme")),
    );
    snap.insert(
        "widgetStyle".into(),
        serde_json::json!(read_ini(&kdeglobals, "KDE", "widgetStyle")),
    );
    snap.insert(
        "iconTheme".into(),
        serde_json::json!(read_ini(&kdeglobals, "Icons", "Theme")),
    );
    snap.insert(
        "gtkTheme".into(),
        serde_json::json!(read_ini(&gtk3, "Settings", "gtk-theme-name")),
    );
    snap.insert(
        "cursorTheme".into(),
        serde_json::json!(read_ini(&gtk3, "Settings", "gtk-cursor-theme-name")),
    );

    let slug: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();

    let dir = custom_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join(format!("{slug}.json"));
    let pretty = serde_json::to_string_pretty(&snap).map_err(|e| e.to_string())?;
    fs::write(&path, pretty).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

/// Launch the desktop's own theme creators. Gtk/icon → Oomox; KDE panels →
/// systemsettings KCMs.
#[tauri::command]
pub fn launch_studio(kind: String) -> Result<String, String> {
    let (bin, args): (&str, &[&str]) = match kind.as_str() {
        "gtk" => ("oomox-gui", &[]),
        "icons" => ("oomox-gui", &[]),
        "plasma" => ("systemsettings", &["kcm_lookandfeel"]),
        "colors" => ("systemsettings", &["kcm_colors"]),
        "cursors" => ("systemsettings", &["kcm_cursortheme"]),
        _ => return Err(format!("unknown studio kind: {kind}")),
    };
    Command::new(bin)
        .args(args)
        .spawn()
        .map_err(|e| format!("couldn't launch {bin}: {e}"))?;
    Ok(format!("launched {bin}"))
}
