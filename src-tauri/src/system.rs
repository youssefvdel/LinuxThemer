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
    /// Representative colors (KDE color schemes / GTK css) for a mock-window.
    pub palette: Option<Vec<String>>,
    /// Sample images (icon themes: a few icons; cursor themes: rendered cursors).
    pub samples: Option<Vec<String>>,
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
        .find(|p| is(p, &["png", "jpg", "jpeg", "webp", "svg", "svgz", "gif"]))
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

fn named_file(dir: &Path, names: &[&str]) -> Option<PathBuf> {
    names.iter().map(|n| dir.join(n)).find(|p| p.is_file())
}

/// Kind-specific preview discovery, based on how each KDE/XDG theme type
/// actually stores its preview on disk (verified against real installs):
/// - global:     contents/previews/*.{png,jpg}
/// - sddm:       preview.png/screenshot.png at root, else backgrounds/*.mp4
/// - wallpapers: contents/screenshot.png, else contents/images/* (raster first)
/// - plasma:     root preview.*, else dialogs/background.svg (theme look)
/// - decorations: root preview.*, else decoration.svg
/// - kvantum:    <theme>.svg at root
/// - icons:      a representative icon (folder/home/known app) as a sample
/// - gtk/colors/cursors: no preview convention -> None (palette/label instead)
fn find_preview(dir: &Path, kind: &str) -> Option<String> {
    let found = match kind {
        "global" => first_media(&dir.join("contents/previews")).or_else(|| {
            named_file(dir, &["preview.png", "fullscreenpreview.jpg", "screenshot.png"])
        }),
        "sddm" => named_file(dir, &["preview.png", "preview.jpg", "screenshot.png"])
            .or_else(|| first_media(&dir.join("backgrounds")))
            .or_else(|| find_media_deep(dir, 3)),
        "wallpapers" => named_file(&dir.join("contents"), &["screenshot.png", "preview.png"])
            .or_else(|| first_media(&dir.join("contents/images")))
            .or_else(|| first_media(&dir.join("contents/images_dark"))),
        "plasma" => named_file(dir, &["preview.png", "preview.jpg", "screenshot.png"])
            .or_else(|| named_file(&dir.join("dialogs"), &["background.svg", "background.svgz"]))
            .or_else(|| first_media(&dir.join("dialogs"))),
        "kvantum" => first_media(dir),
        _ => None, // icons/decorations render from samples/palette, not a single preview
    };
    found.map(|p| p.to_string_lossy().to_string())
}

/// Find one icon by name across both common icon-theme layouts:
/// `category/size/name.ext` (e.g. Amy-Dark: `places/32/folder.svg`) and
/// Resolve the theme's lookup chain: the theme itself, its `Inherits=` parents
/// (declared in `index.theme`), then the system fallbacks hicolor/breeze.
/// This mirrors KIconLoader, so sparse themes (hicolor) still resolve icons.
fn icon_chain(dir: &Path) -> Vec<PathBuf> {
    let mut chain = vec![dir.to_path_buf()];
    let mut inherits: Vec<String> = Vec::new();
    if let Ok(text) = fs::read_to_string(dir.join("index.theme")) {
        for line in text.lines() {
            if let Some(rest) = line.trim().strip_prefix("Inherits=") {
                inherits = rest
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                break;
            }
        }
    }
    // Fallbacks: Inherits parents + hicolor/breeze. Themes can live under either
    // ~/.local/share/icons or /usr/share/icons, but the fallback theme (breeze)
    // is only in the system root — so search both roots for each fallback name.
    let mut roots: Vec<&Path> = Vec::new();
    if let Some(root) = dir.parent() {
        roots.push(root);
    }
    let system_root = Path::new("/usr/share/icons");
    if system_root.is_dir() && !roots.contains(&system_root) {
        roots.push(system_root);
    }
    for root in roots {
        for name in inherits.iter().map(String::as_str).chain(["hicolor", "breeze", "breeze-dark"]) {
            let p = root.join(name);
            if p.is_dir() && !chain.contains(&p) {
                chain.push(p);
            }
        }
    }
    chain
}

/// Find one icon by name across both common icon-theme layouts (`category/size/`
/// and `size/category/`) and the theme's inheritance chain, matching KDE's
/// PNG-first → SVG → SVGZ preference.
fn find_icon(dir: &Path, name: &str) -> Option<PathBuf> {
    const SUBDIRS: &[&str] = &[
        "places",
        "apps",
        "devices",
        "actions",
        "mimetypes",
        "categories",
        "status",
        "emblems",
    ];
    const SIZE_DIRS: &[&str] = &[
        "scalable",
        "symbolic",
        "48",
        "64",
        "32",
        "24",
        "22",
        "16",
        "128",
        "256",
        "16x16",
        "22x22",
        "24x24",
        "32x32",
        "48x48",
        "64x64",
        "128x128",
        "256x256",
        "512x512",
    ];
    for theme_dir in icon_chain(dir) {
        for ext in ["png", "svg", "svgz"] {
            let fname = format!("{name}.{ext}");
            // layout 1: category/size/name
            for sub in SUBDIRS {
                for size in SIZE_DIRS {
                    let p = theme_dir.join(sub).join(size).join(&fname);
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
            // layout 2: size/category/name
            for size in SIZE_DIRS {
                for sub in SUBDIRS {
                    let p = theme_dir.join(size).join(sub).join(&fname);
                    if p.is_file() {
                        return Some(p);
                    }
                }
            }
        }
    }
    None
}

/// Extract colors from a GTK theme's `gtk-3.0/gtk.css` (@define-color) for
/// palette swatches — GTK themes have no screenshot convention.
fn read_gtk_palette(dir: &Path) -> Option<Vec<String>> {
    let text = fs::read_to_string(dir.join("gtk-3.0/gtk.css")).ok()?;
    let mut colors: Vec<String> = vec![];
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("@define-color") {
            continue;
        }
        let rest = line.trim_start_matches("@define-color");
        let value = rest.split_whitespace().nth(1).unwrap_or("").trim_end_matches(';');
        let v = value.to_ascii_lowercase();
        let hex = if v.starts_with('#') && v.len() == 7 {
            Some(v)
        } else if v == "white" {
            Some("#ffffff".to_string())
        } else if v == "black" {
            Some("#000000".to_string())
        } else {
            None
        };
        if let Some(h) = hex {
            if !colors.contains(&h) {
                colors.push(h);
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
    if !matches!(ext.as_str(), "svg" | "svgz" | "mp4" | "webm" | "avif") {
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
        .arg("-pix_fmt")
        .arg("rgba")
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
                preview: find_preview(&d, kind).and_then(|p| thumb_path_for(&p)),
                palette: match kind {
                    "gtk" => read_gtk_palette(&d),
                    "decorations" => read_deco_color(&d),
                    _ => None,
                },
                samples: match kind {
                    "icons" => find_representative_icons(&d),
                    "cursors" => find_cursor_samples(&d),
                    "decorations" => find_deco_buttons(&d),
                    _ => None,
                },
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
                    samples: None,
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

/// Extract colors from a KDE `.colors` file (INI) in a fixed semantic order:
/// [window_bg, view_bg, button_bg, selection_bg, window_fg, view_fg] — enough
/// to render a mock window like KDE System Settings' Colors page.
fn read_color_palette(path: &Path) -> Option<Vec<String>> {
    let text = fs::read_to_string(path).ok()?;
    let mut cur = String::new();
    let mut window = (String::new(), String::new());
    let mut view = (String::new(), String::new());
    let mut button = (String::new(), String::new());
    let mut selection = (String::new(), String::new());
    for raw in text.lines() {
        let line = raw.trim();
        if line.starts_with('[') && line.ends_with(']') {
            cur = line[1..line.len() - 1].to_string();
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let hex = match rgb_to_hex(v.trim()) {
                Some(h) => h,
                None => continue,
            };
            let slot = match (cur.as_str(), k.trim()) {
                ("Colors:Window", "BackgroundNormal") => &mut window.0,
                ("Colors:Window", "ForegroundNormal") => &mut window.1,
                ("Colors:View", "BackgroundNormal") => &mut view.0,
                ("Colors:View", "ForegroundNormal") => &mut view.1,
                ("Colors:Button", "BackgroundNormal") => &mut button.0,
                ("Colors:Button", "ForegroundNormal") => &mut button.1,
                ("Colors:Selection", "BackgroundNormal") => &mut selection.0,
                ("Colors:Selection", "ForegroundNormal") => &mut selection.1,
                _ => continue,
            };
            if slot.is_empty() {
                *slot = hex;
            }
        }
    }
    let mut out = vec![];
    for s in [&window, &view, &button, &selection] {
        if !s.0.is_empty() {
            out.push(s.0.clone());
        }
    }
    if !window.1.is_empty() {
        out.push(window.1.clone());
    }
    if !view.1.is_empty() {
        out.push(view.1.clone());
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Decode one X11 cursor theme file (libXcursor "Xcur" format) and cache a PNG
/// render. Layout (verified byte-for-byte against real themes + libXcursor):
/// 16-byte file header, TOC of 12-byte entries, then chunks. An image chunk
/// (type 0xfffd0002) = 16-byte chunk header (header, type, subtype=nominal
/// size, version) + 20-byte image header (size, width, xhot, yhot, delay) +
/// raw ARGB32 pixels (width × subtype) at pos+36.
fn xcursor_to_png(path: &Path) -> Option<String> {
    let bytes = fs::read(path).ok()?;
    if bytes.len() < 16 || u32::from_le_bytes(bytes[0..4].try_into().ok()?) != 0x72756358 {
        return None; // not an Xcursor file
    }
    let ntoc = u32::from_le_bytes(bytes[12..16].try_into().ok()?) as usize;
    if bytes.len() < 16 + ntoc * 12 {
        return None;
    }
    // Prefer the chunk whose nominal size is closest to 32 (good preview size).
    let mut best: Option<(i32, u32, usize)> = None; // (score, subtype, pos)
    for i in 0..ntoc {
        let off = 16 + i * 12;
        let ctype = u32::from_le_bytes(bytes[off..off + 4].try_into().ok()?);
        if ctype != 0xfffd_0002 {
            continue;
        }
        let subtype = u32::from_le_bytes(bytes[off + 4..off + 8].try_into().ok()?);
        let pos = u32::from_le_bytes(bytes[off + 8..off + 12].try_into().ok()?) as usize;
        if subtype == 0 || pos + 36 > bytes.len() {
            continue;
        }
        let score = if subtype >= 16 {
            -(subtype as i32 - 32).abs()
        } else {
            i32::MAX
        };
        if best.map(|(s, _, _)| score > s).unwrap_or(true) {
            best = Some((score, subtype, pos));
        }
    }
    let (_, _, pos) = best?;
    // Image header (20B at pos+16): width, height, xhot, yhot, delay.
    // Nominal size = chunk subtype; actual pixel dims = width × height.
    let width = u32::from_le_bytes(bytes[pos + 16..pos + 20].try_into().ok()?) as usize;
    let height = u32::from_le_bytes(bytes[pos + 20..pos + 24].try_into().ok()?) as usize;
    let px_start = pos + 36;
    let n = width.saturating_mul(height);
    if width == 0 || height == 0 || px_start + n * 4 > bytes.len() {
        return None;
    }
    let mut rgba = Vec::with_capacity(n * 4);
    for i in 0..n {
        let o = px_start + i * 4;
        let px = u32::from_le_bytes(bytes[o..o + 4].try_into().ok()?);
        rgba.extend_from_slice(&[(px >> 16) as u8, (px >> 8) as u8, px as u8, (px >> 24) as u8]);
    }
    let img = image::RgbaImage::from_raw(width as u32, height as u32, rgba)?;
    // Key the cache by canonical path so aliases (symlinks) share one PNG.
    let canon = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let mut hasher = DefaultHasher::new();
    canon.to_string_lossy().hash(&mut hasher);
    let dir = thumb_cache_dir();
    fs::create_dir_all(&dir).ok()?;
    let out = dir.join(format!("{:016x}.png", hasher.finish()));
    if !out.exists() {
        let f = fs::File::create(&out).ok()?;
        let mut w = std::io::BufWriter::new(f);
        image::DynamicImage::ImageRgba8(img)
            .write_to(&mut w, image::ImageFormat::Png)
            .ok()?;
    }
    Some(out.to_string_lossy().to_string())
}

/// Render real cursors from a cursor theme so the card shows a 3×3 cursor grid.
/// Names are KDE's own preview set (plasma-workspace cursortheme PreviewWidget).
fn find_cursor_samples(dir: &Path) -> Option<Vec<String>> {
    const NAMES: &[&str] = &[
        "left_ptr", "left_ptr_watch", "wait", "pointer", "help", "ibeam",
        "size_all", "size_fdiag", "cross", "split_h", "size_ver", "size_hor",
        "size_bdiag", "split_v",
    ];
    let cd = dir.join("cursors");
    let mut out: Vec<String> = vec![];
    let mut seen: HashSet<PathBuf> = HashSet::new();
    for name in NAMES {
        let p = cd.join(name);
        if !p.is_file() {
            continue;
        }
        let canon = fs::canonicalize(&p).unwrap_or_else(|_| p.clone());
        if !seen.insert(canon) {
            continue; // alias (symlink) of an already-picked cursor
        }
        if let Some(png) = xcursor_to_png(&p) {
            out.push(png);
        }
        if out.len() >= 9 {
            break;
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Collect an Aurorae decoration theme's titlebar button art (close/minimize/
/// maximize) so the card can render a KDE-style window mockup. KDE composites
/// these button SVGs onto the titlebar; the raw `decoration.svg` alone is just
/// an abstract frame and looks wrong as a preview.
fn find_deco_buttons(dir: &Path) -> Option<Vec<String>> {
    const NAMES: &[&str] = &["close", "minimize", "maximize", "restore"];
    let out: Vec<String> = NAMES
        .iter()
        .filter_map(|n| {
            let p = dir.join(format!("{n}.svg"));
            if p.is_file() {
                thumb_path_for(&p.to_string_lossy())
            } else {
                None
            }
        })
        .take(3)
        .collect();
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Extract the titlebar background color from an Aurorae `decoration.svg`
/// (`.ColorScheme-Background { color:#RRGGBB; ... }`).
fn read_deco_color(dir: &Path) -> Option<Vec<String>> {
    let text = fs::read_to_string(dir.join("decoration.svg")).ok()?;
    let block = text.split(".ColorScheme-Background").nth(1)?;
    let block = &block[..block.find('}').unwrap_or(block.len())];
    let c = block
        .split("color:")
        .nth(1)?
        .trim()
        .chars()
        .take_while(|ch| *ch != ';' && *ch != '}')
        .collect::<String>();
    let c = c.trim().to_ascii_lowercase();
    if c.starts_with('#') && c.len() == 7 {
        Some(vec![c])
    } else {
        None
    }
}

/// Collect the 6 preview icons KDE shows for an icon theme (3×2 grid), using
/// KDE's own icon slots with per-slot fallbacks (plasma-workspace icons KCM).
fn find_representative_icons(dir: &Path) -> Option<Vec<String>> {
    const SLOTS: &[&[&str]] = &[
        &["system-run", "exec"],
        &["folder"],
        &["document", "text-x-generic"],
        &["user-trash", "user-trash-empty"],
        &["help-browser", "system-help", "help-about", "help-contents"],
        &["preferences-system", "systemsettings", "configure"],
    ];
    let out: Vec<String> = SLOTS
        .iter()
        .filter_map(|names| names.iter().find_map(|name| find_icon(dir, name)))
        .take(6)
        .filter_map(|p| thumb_path_for(&p.to_string_lossy()))
        .collect();
    if out.is_empty() {
        None
    } else {
        Some(out)
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
                    samples: None,
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
    let mut cur; // first use is assignment (section header), so no init needed
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a synthetic "Xcur" file matching the empirically-verified layout:
    /// 16B file header, 12B TOC, chunk = 16B header + 20B image header + pixels.
    fn make_xcursor() -> Vec<u8> {
        let mut b = Vec::new();
        b.extend_from_slice(&0x72756358u32.to_le_bytes()); // "Xcur"
        b.extend_from_slice(&16u32.to_le_bytes()); // header size
        b.extend_from_slice(&0x0001_0000u32.to_le_bytes()); // version (as real files store it)
        b.extend_from_slice(&1u32.to_le_bytes()); // ntoc
        // TOC: one image chunk, nominal size 16, at position 28
        b.extend_from_slice(&0xfffd_0002u32.to_le_bytes());
        b.extend_from_slice(&16u32.to_le_bytes());
        b.extend_from_slice(&28u32.to_le_bytes());
        // chunk header (16B): header=36, type, subtype=16, version=1
        b.extend_from_slice(&36u32.to_le_bytes());
        b.extend_from_slice(&0xfffd_0002u32.to_le_bytes());
        b.extend_from_slice(&16u32.to_le_bytes());
        b.extend_from_slice(&1u32.to_le_bytes());
        // image header (20B): width, height, xhot, yhot, delay — NON-square to catch
        // any width/height swap (this exact bug shipped and broke KDE_Classic)
        b.extend_from_slice(&10u32.to_le_bytes()); // width
        b.extend_from_slice(&16u32.to_le_bytes()); // height
        b.extend_from_slice(&3u32.to_le_bytes()); // xhot
        b.extend_from_slice(&1u32.to_le_bytes()); // yhot
        b.extend_from_slice(&40u32.to_le_bytes()); // delay
        // pixels: 10x16 ARGB (first pixel transparent, rest opaque teal)
        for i in 0..10 * 16 {
            let a = if i == 0 { 0 } else { 0xff };
            b.extend_from_slice(&(a << 24 | 0x10_20_30u32).to_le_bytes());
        }
        b
    }

    #[test]
    fn xcursor_decode_roundtrip() {
        let dir = std::env::temp_dir().join("lt-xcursor-test");
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("test_cursor");
        std::fs::write(&f, make_xcursor()).unwrap();

        let png = xcursor_to_png(&f).expect("should decode the synthetic Xcursor file");
        assert!(std::path::Path::new(&png).exists(), "PNG should be written");

        // second call must hit the cache (same canonical path => same key)
        let again = xcursor_to_png(&f).unwrap();
        assert_eq!(png, again, "cached PNG path should be reused");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn xcursor_rejects_garbage() {
        let dir = std::env::temp_dir().join("lt-xcursor-test2");
        std::fs::create_dir_all(&dir).unwrap();
        let f = dir.join("not_a_cursor");
        std::fs::write(&f, b"this is not an Xcursor file at all").unwrap();
        assert!(xcursor_to_png(&f).is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
