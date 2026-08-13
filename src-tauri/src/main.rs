#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    // WebKitGTK on NVIDIA/Wayland: DMA-BUF renderer retries failed framebuffers
    // every frame -> ~10fps + input lag. Force the fallback renderer, and disable
    // NVIDIA explicit-sync (mismatch with the compositor). tauri-apps/tauri#9394
    #[cfg(target_os = "linux")]
    {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        std::env::set_var("__NV_DISABLE_EXPLICIT_SYNC", "1");
    }

    linux_themer_lib::run()
}
