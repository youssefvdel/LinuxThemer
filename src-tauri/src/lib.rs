use reqwest::Url;

mod system;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        builder = builder.plugin(tauri_plugin_mcp_bridge::init());
    }

    builder
        .invoke_handler(tauri::generate_handler![
            fetch_themes,
            system::list_installed,
            system::save_current_theme,
            system::launch_studio
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

const OCS_BASE: &str = "https://api.kde-look.org/ocs/v1/content/data";

/// Fetch themes from the KDE-Look / opendesktop OCS API (proxied through Rust
/// to bypass CORS). Returns the raw OCS JSON payload.
#[tauri::command]
async fn fetch_themes(
    category: String,
    page: u32,
    search: String,
    sortmode: String,
) -> Result<serde_json::Value, String> {
    let mut url = Url::parse(OCS_BASE).map_err(|e| e.to_string())?;
    {
        let mut q = url.query_pairs_mut();
        q.append_pair("categories", &category);
        q.append_pair("page", &page.to_string());
        q.append_pair("pagesize", "30");
        q.append_pair("sortmode", &sortmode);
        q.append_pair("format", "json");
        if !search.trim().is_empty() {
            q.append_pair("search", search.trim());
        }
    }

    let client = reqwest::Client::new();
    client
        .get(url)
        .header("User-Agent", "LinuxThemer/0.1")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json::<serde_json::Value>()
        .await
        .map_err(|e| e.to_string())
}
