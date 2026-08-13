//! Local system introspection + integration: installed-theme discovery,
//! current-theme snapshot, and launching external theme creators.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
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
    /// Representative colors (KDE color schemes) when no screenshot exists.
    pub palette: Option<Vec<String>>,
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

/// First image *or video* file in a directory. Theme previews are png/jpg/
/// svg/webp/avif/gif, or mp4/webm for animated SDDM previews.
fn first_media(dir: &Path) -> Option<PathBuf> {
    let rd = fs::read_dir(dir).ok()?;
    let files: Vec<PathBuf> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_file())
        .collect();
    let ext =
        |p: &Path| p.extension().and_then(|x| x.to_str()).map(|s| s.to_ascii_lowercase());
    let is = |p: &Path, set: &[&str]| ext(p).map(|e| set.contains(&e.as_str())).unwrap_or(false);
    // Prefer universally-renderable raster formats over avif/video (WebKitGTK
    // may lack AVIF, and mp4/webm need a <video> element).
    files
        .iter()
        .find(|p| is(p, &["png", "jpg", "jpeg", "webp", "svg", "gif"]))
        .or_else(|| files.iter().find(|p| is(p, &["avif", "mp4", "webm"])))
        .cloned()
}

/// Recursive media lookup (bounded depth) for themes that store their preview
/// or screenshot in a non-standard subdirectory.
fn find_media_deep(dir: &Path, depth: u8) -> Option<PathBuf> {
    if let Some(p) = first_media(dir) {
        return Some(p);
    }
    if depth == 0 {
        return None;
    }
    let rd = fs::read_dir(dir).ok()?;
    let mut subs: Vec<PathBuf> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    subs.sort();
    for d in subs {
        if let Some(p) = find_media_deep(&d, depth - 1) {
            return Some(p);
        }
    }
    None
}

fn find_preview(dir: &Path) -> Option<String> {
    for sub in ["contents/previews", "contents/images", "previews", "images"] {
        if let Some(p) = first_media(&dir.join(sub)) {
            return Some(p.to_string_lossy().to_string());
        }
    }
    for fname in [
        "preview.png",
        "preview.jpg",
        "preview.webp",
        "preview.svg",
        "preview.avif",
        "preview.gif",
        "preview.mp4",
        "preview.webm",
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
    find_media_deep(dir, 3).map(|p| p.to_string_lossy().to_string())
}

/// Cache dir for generated preview thumbnails.
fn thumb_cache_dir() -> PathBuf {
    home().join(".cache/linuxthemer/previews")
}

/// Rasterize svg/video previews to a cached PNG thumbnail (ffmpeg handles both),
/// so WebKitGTK reliably renders them. Raster formats pass through untouched.
fn thumb_path_for(path: &str) -> Option<String> {
    let orig = Path::new(path);
    let ext = orig
        .extension()
        .and_then(|x| x.to_str())
        .map(|s| s.to_ascii_lowercase())?;
    if !matches!(ext.as_str(), "svg" | "mp4" | "webm") {
        return Some(path.to_string());
    }
    let mut hasher = DefaultHasher::new();
    orig.to_string_lossy().hash(&mut hasher);
    let hash = hasher.finish();
    let dir = thumb_cache_dir();
    let _ = fs::create_dir_all(&dir);
    let out = dir.join(format!("{hash:016x}.png"));
    if out.exists() {
        return Some(out.to_string_lossy().to_string());
    }
    let ok = Command::new("ffmpeg")
        .arg("-y")
        .arg("-loglevel")
        .arg("error")
        .arg("-i")
        .arg(orig)
        .arg("-frames:v")
        .arg("1")
        .arg("-vf")
        .arg("scale=640:-2")
        .arg(&out)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        Some(out.to_string_lossy().to_string())
    } else {
        // ffmpeg failed — fall back to the original (browser may still render it).
        Some(path.to_string())
    }
}

/// A real icon theme has `index.theme` plus icon-size/category directories.
/// Cursor themes also ship `index.theme` but only a `cursors/` dir, so this
/// keeps them out of the Icons tab.
fn is_icon_theme(dir: &Path) -> bool {
    if !has_entry(dir, "index.theme") {
        return false;
    }
    const ICON_DIRS: &[&str] = &[
        "scalable", "apps", "actions", "categories", "places", "devices", "mimetypes",
        "status", "emblems", "animations", "panel", "preferences", "symbolic", "16x16",
        "22x22", "24x24", "32x32", "48x48", "64x64", "128x128", "256x256", "512x512",
    ];
    ICON_DIRS.iter().any(|n| has_entry(dir, n))
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
                preview: find_preview(&d).and_then(|p| thumb_path_for(&p)),
                palette: None,
            });
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

/// Scan for themes that live as single files (KDE color schemes are `.colors` files).
fn scan_files(kind: &str, roots: &[PathBuf], ext: &str) -> Vec<InstalledTheme> {
    let mut seen = HashSet::new();
    let mut out = vec![];
    for root in roots {
        if let Ok(rd) = fs::read_dir(root) {
            for e in rd.filter_map(|e| e.ok()) {
                let p = e.path();
                if !p.is_file() || p.extension().and_then(|x| x.to_str()) != Some(ext) {
                    continue;
                }
                let name = p
                    .file_stem()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                if name.is_empty() || !seen.insert(name.clone()) {
                    continue;
                }
                out.push(InstalledTheme {
                    id: format!("{kind}:{name}"),
                    name,
                    kind: kind.to_string(),
                    path: p.to_string_lossy().to_string(),
                    preview: None,
                    palette: if kind == "colors" {
                        read_color_palette(&p)
                    } else {
                        None
                    },
                });
            }
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

fn custom_dir() -> PathBuf {
    home().join(".local/share/linuxthemer/custom")
}

/// Extract representative surface colors from a KDE `.colors` file (INI), so
/// color schemes can render a palette swatch when no screenshot exists.
fn read_color_palette(path: &Path) -> Option<Vec<String>> {
    let text = fs::read_to_string(path).ok()?;
    let mut colors: Vec<String> = vec![];
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == "BackgroundNormal" {
                if let Some(hex) = rgb_to_hex(v.trim()) {
                    if !colors.contains(&hex) {
                        colors.push(hex);
                    }
                }
            }
        }
        if colors.len() >= 6 {
            break;
        }
    }
    if colors.is_empty() {
        None
    } else {
        Some(colors)
    }
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
                    palette: None,
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
        &[
            h.join(".local/share/aurorae/themes"),
            PathBuf::from("/usr/share/aurorae/themes"),
        ],
        |_| true,
    ));
    all.extend(scan_files(
        "colors",
        &[
            h.join(".local/share/color-schemes"),
            PathBuf::from("/usr/share/color-schemes"),
        ],
        "colors",
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
        is_icon_theme,
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

/// Remove an installed theme from disk. User paths are deleted directly;
/// system paths (outside $HOME) are elevated through polkit so the user gets
/// a graphical auth prompt instead of a raw permission error.
#[tauri::command]
pub fn remove_installed(path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    let h = home();

    if !p.starts_with(&h) {
        let status = Command::new("pkexec")
            .arg("rm")
            .arg("-rf")
            .arg(&path)
            .status()
            .map_err(|e| format!("pkexec unavailable: {e}"))?;
        if !status.success() {
            return Err(format!("removal failed (pkexec exited {status})"));
        }
        return Ok(());
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
    pub accent_color: String,
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
    // KDE applies the user's chosen theme through a layered config: the main
    // ~/.config/<file> holds runtime values, and ~/.config/kdedefaults/<file>
    // holds the "defaults" written when a global theme is applied. Some keys
    // (ColorScheme, widgetStyle, icons) live only in kdedefaults, so merge both.
    let kdeglobals = h.join(".config/kdeglobals");
    let kd_kdeglobals = h.join(".config/kdedefaults/kdeglobals");
    let kcminputrc = h.join(".config/kcminputrc");
    let kd_kcminputrc = h.join(".config/kdedefaults/kcminputrc");
    let gtk3 = h.join(".config/gtk-3.0/settings.ini");
    let plasmarc = h.join(".config/plasmarc");
    let kd_plasmarc = h.join(".config/kdedefaults/plasmarc");
    let kvconfig = h.join(".config/Kvantum/kvantum.kvconfig");
    Ok(CurrentTheme {
        widget_style: read_ini_merged(&[&kdeglobals, &kd_kdeglobals], "KDE", "widgetStyle")
            .unwrap_or_else(|| "Breeze".to_string()),
        color_scheme: read_ini_merged(&[&kdeglobals, &kd_kdeglobals], "General", "ColorScheme")
            .unwrap_or_default(),
        icon_theme: read_ini_merged(&[&kdeglobals, &kd_kdeglobals], "Icons", "Theme")
            .unwrap_or_default(),
        cursor_theme: read_ini_merged(&[&kcminputrc, &kd_kcminputrc], "Mouse", "cursorTheme")
            .unwrap_or_default(),
        gtk_theme: read_ini(&gtk3, "Settings", "gtk-theme-name").unwrap_or_default(),
        plasma_theme: read_ini_merged(&[&plasmarc, &kd_plasmarc], "Theme", "name")
            .unwrap_or_default(),
        kvantum: read_ini(&kvconfig, "General", "theme").unwrap_or_default(),
        accent_color: read_ini(&kdeglobals, "General", "AccentColor")
            .and_then(|v| rgb_to_hex(&v))
            .unwrap_or_else(|| "#3daee9".to_string()),
    })
}

/// Read a key from the first file that has it (KDE config layering: main file
/// wins, kdedefaults is the fallback).
fn read_ini_merged(files: &[&Path], section: &str, key: &str) -> Option<String> {
    files.iter().find_map(|f| read_ini(f, section, key))
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

fn rgb_to_hex(rgb: &str) -> Option<String> {
    let parts: Vec<&str> = rgb.split(',').map(|s| s.trim()).collect();
    if parts.len() != 3 {
        return None;
    }
    let r: u8 = parts[0].parse().ok()?;
    let g: u8 = parts[1].parse().ok()?;
    let b: u8 = parts[2].parse().ok()?;
    Some(format!("#{r:02x}{g:02x}{b:02x}"))
}

fn hex_to_rgb(hex: &str) -> Result<String, String> {
    let h = hex.trim_start_matches('#');
    if h.len() != 6 {
        return Err("accent color must be #RRGGBB".into());
    }
    let r = u8::from_str_radix(&h[0..2], 16).map_err(|_| "invalid hex".to_string())?;
    let g = u8::from_str_radix(&h[2..4], 16).map_err(|_| "invalid hex".to_string())?;
    let b = u8::from_str_radix(&h[4..6], 16).map_err(|_| "invalid hex".to_string())?;
    Ok(format!("{r},{g},{b}"))
}

/// Set a single key in an INI file, preserving all other content (kwriteconfig-style).
fn set_ini_key(path: &Path, section: &str, key: &str, value: &str) -> Result<(), String> {
    let text = fs::read_to_string(path).unwrap_or_default();
    let mut out: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut in_section = false;
    let mut wrote = false;

    for line in text.lines() {
        let t = line.trim();
        if t.starts_with('[') && t.ends_with(']') {
            if in_section && !wrote {
                out.push(format!("{key}={value}"));
                wrote = true;
            }
            cur = t[1..t.len() - 1].to_string();
            in_section = cur == section;
            out.push(line.to_string());
        } else if in_section
            && !wrote
            && line
                .split_once('=')
                .map(|(k, _)| k.trim() == key)
                .unwrap_or(false)
        {
            out.push(format!("{key}={value}"));
            wrote = true;
        } else {
            out.push(line.to_string());
        }
    }
    if !wrote {
        if !in_section {
            out.push(format!("[{section}]"));
        }
        out.push(format!("{key}={value}"));
    }
    let mut s = out.join("\n");
    if !s.ends_with('\n') {
        s.push('\n');
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    fs::write(path, s).map_err(|e| e.to_string())
}

/// Apply one theme component to the live system (write config + reload where possible).
#[tauri::command]
pub fn apply_component(kind: String, value: String) -> Result<(), String> {
    let h = home();
    match kind.as_str() {
        "gtk" => {
            for p in ["gtk-3.0/settings.ini", "gtk-4.0/settings.ini"] {
                set_ini_key(&h.join(".config").join(p), "Settings", "gtk-theme-name", &value)?;
            }
        }
        "icons" => set_ini_key(&h.join(".config/kdeglobals"), "Icons", "Theme", &value)?,
        "cursors" => {
            set_ini_key(&h.join(".config/kcminputrc"), "Mouse", "cursorTheme", &value)?;
            let _ = Command::new("plasma-apply-cursortheme").arg(&value).spawn();
        }
        "colors" => {
            set_ini_key(&h.join(".config/kdeglobals"), "General", "ColorScheme", &value)?;
            let _ = Command::new("plasma-apply-colorscheme").arg(&value).spawn();
        }
        "qt" => set_ini_key(&h.join(".config/kdeglobals"), "KDE", "widgetStyle", &value)?,
        "plasma" => {
            set_ini_key(&h.join(".config/plasmarc"), "Theme", "name", &value)?;
            let _ = Command::new("plasma-apply-desktoptheme").arg(&value).spawn();
        }
        "accent" => {
            let rgb = hex_to_rgb(&value)?;
            set_ini_key(&h.join(".config/kdeglobals"), "General", "AccentColor", &rgb)?;
        }
        _ => return Err(format!("unknown component: {kind}")),
    }
    Ok(())
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
