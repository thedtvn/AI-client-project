#[tauri::command]
pub fn md_to_html(text: String) -> Result<String, String> {
    Ok(markdown::to_html(&text))
}
