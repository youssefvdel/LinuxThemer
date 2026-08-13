//! Local system introspection + integration: installed-theme discovery,
//! current-theme snapshot, and launching external theme creators.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct InstalledTheme {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub path: String,
    pub preview: Option<String>,
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

fn first_image(dir: &Path) -> Option<PathBuf> {
    let rd = fs::read_dir(dir).ok()?;
    for e in rd.filter_map(|e| e.ok()) {
        let p = e.path();
        if !p.is_file() {
            continue;
        }
        if let Some(ext) = p.extension().and_then(|x| x.to_str()) {
            let e = ext.to_ascii_lowercase();
            if matches!(e.as_str(), "png" | "jpg" | "jpeg" | "webp") {
                return Some(p);
            }
        }
    }
    None
}

fn find_preview(dir: &Path, kind: &str) -> Option<String> {
    if kind == "wallpapers" {
        return first_image(dir).map(|p| p.to_string_lossy().to_string());
    }
    for sub in ["contents/previews", "previews"] {
        if let Some(p) = first_image(&dir.join(sub)) {
            return Some(p.to_string_lossy().to_string());
        }
    }
    for fname in [
        "preview.png",
        "preview.jpg",
        "preview.webp",
        "screenshot.png",
        "screenshot.jpg",
        "theme-preview.png",
        "theme-preview.jpg",
    ] {
        let p = dir.join(fname);
        if p.is_file() {
            return Some(p.to_string_lossy().to_string());
        }
    }
    None
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
                preview: find_preview(&d, kind),
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
                    preview: None,
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

/// Remove an installed theme from disk. Only paths under $HOME are removable —
/// system themes under /usr/share are never touched.
#[tauri::command]
pub fn remove_installed(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    let h = home();
    if !p.starts_with(&h) {
        return Err("refusing to remove anything outside your home directory".into());
    }
    if p.is_dir() {
        fs::remove_dir_all(&p).map_err(|e| e.to_string())
    } else if p.is_file() {
        fs::remove_file(&p).map_err(|e| e.to_string())
    } else {
        Err("path does not exist".into())
    }
}

/// The user's currently-applied theme, read from the live config files.
#[derive(Serialize)]
pub struct CurrentTheme {
    pub widget_style: String,
    pub color_scheme: String,
    pub icon_theme: String,
    pub cursor_theme: String,
    pub gtk_theme: String,
    pub plasma_theme: String,
    pub kvantum: String,
}

/// Components assembled into a new global theme by the Studio.
#[derive(Serialize, Deserialize)]
pub struct GlobalThemeSpec {
    pub gtk: String,
    pub widget_style: String,
    pub kvantum: String,
    pub icons: String,
    pub cursors: String,
    pub colors: String,
    pub plasma: String,
}

fn slugify(name: &str) -> String {
    let joined: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if joined.is_empty() {
        "custom-theme".to_string()
    } else {
        joined
    }
}

/// Snapshot the current theme values (Qt style, colors, icons, cursor, GTK,
/// Plasma desktop theme, Kvantum).
#[tauri::command]
pub fn current_theme() -> Result<CurrentTheme, String> {
    let h = home();
    let kdeglobals = h.join(".config/kdeglobals");
    let kcminputrc = h.join(".config/kcminputrc");
    let gtk3 = h.join(".config/gtk-3.0/settings.ini");
    let plasmarc = h.join(".config/plasmarc");
    let kvconfig = h.join(".config/Kvantum/kvantum.kvconfig");
    Ok(CurrentTheme {
        widget_style: read_ini(&kdeglobals, "KDE", "widgetStyle")
            .unwrap_or_else(|| "Breeze".to_string()),
        color_scheme: read_ini(&kdeglobals, "General", "ColorScheme").unwrap_or_default(),
        icon_theme: read_ini(&kdeglobals, "Icons", "Theme").unwrap_or_default(),
        cursor_theme: read_ini(&kcminputrc, "Mouse", "cursorTheme").unwrap_or_default(),
        gtk_theme: read_ini(&gtk3, "Settings", "gtk-theme-name").unwrap_or_default(),
        plasma_theme: read_ini(&plasmarc, "Theme", "name").unwrap_or_default(),
        kvantum: read_ini(&kvconfig, "General", "theme").unwrap_or_default(),
    })
}

/// Write a KDE look-and-feel (global theme) package from the assembled spec,
/// so it appears in System Settings → Global Theme. GTK + Kvantum aren't
/// settable via look-and-feel defaults, so they're kept in a linuxthemer.json
/// manifest for the apply engine.
#[tauri::command]
pub fn save_global_theme(name: String, spec: GlobalThemeSpec) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("theme name is empty".into());
    }
    let slug = slugify(name);
    let dir = home()
        .join(".local/share/plasma/look-and-feel")
        .join(&slug);
    fs::create_dir_all(dir.join("contents")).map_err(|e| e.to_string())?;

    let metadata = format!(
        "[Desktop Entry]\nEncoding=UTF-8\nName={name}\nComment=Created with LinuxThemer\nType=Service\nX-KDE-ServiceTypes=Plasma/LookAndFeel\nX-KDE-PluginInfo-Author=LinuxThemer\nX-KDE-PluginInfo-Name={slug}\nX-KDE-PluginInfo-Version=1.0\nX-KDE-PluginInfo-Category=Plasma Look And Feel\nX-KDE-PluginInfo-EnabledByDefault=true\n"
    );
    fs::write(dir.join("metadata.desktop"), metadata).map_err(|e| e.to_string())?;

    let mut defaults = String::new();
    if !spec.colors.is_empty() {
        defaults.push_str(&format!("[kdeglobals][General]\nColorScheme={}\n\n", spec.colors));
    }
    if !spec.icons.is_empty() {
        defaults.push_str(&format!("[kdeglobals][Icons]\nTheme={}\n\n", spec.icons));
    }
    if !spec.widget_style.is_empty() {
        defaults.push_str(&format!(
            "[kdeglobals][KDE]\nwidgetStyle={}\n\n",
            spec.widget_style
        ));
    }
    if !spec.cursors.is_empty() {
        defaults.push_str(&format!(
            "[kcminputrc][Mouse]\ncursorTheme={}\n\n",
            spec.cursors
        ));
    }
    if !spec.plasma.is_empty() {
        defaults.push_str(&format!("[plasmarc][Theme]\nname={}\n\n", spec.plasma));
    }
    fs::write(dir.join("contents/defaults"), defaults).map_err(|e| e.to_string())?;

    let manifest = serde_json::json!({
        "gtk": spec.gtk,
        "kvantum": spec.kvantum,
        "widgetStyle": spec.widget_style,
        "icons": spec.icons,
        "cursors": spec.cursors,
        "colors": spec.colors,
        "plasma": spec.plasma,
    });
    fs::write(
        dir.join("linuxthemer.json"),
        serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;

    Ok(dir.to_string_lossy().to_string())
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
