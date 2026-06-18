mod ascii;
mod commands;
mod export;
mod fonts;
mod scraper;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_system_fonts,
            commands::generate_ascii,
            commands::render_preview,
            commands::export_ascii,
            commands::grid_to_colored_html,
            commands::read_text_file,
            commands::fetch_lyrics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
