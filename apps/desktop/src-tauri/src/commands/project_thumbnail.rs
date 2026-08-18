//! Project thumbnail commands.
//!
//! Thumbnail bytes are project metadata only; they never touch ROM state. They are kept in the
//! current project DTO and are persisted by the next project-v2 save.

use std::io::Cursor;
use std::path::PathBuf;

use image::ImageReader;
use tauri::State;

use crate::app_state::AppState;
use project_core::{load_project_v2, Project, ProjectThumbnail, PROJECT_V2_FILENAME};

const MAX_THUMBNAIL_PNG_BYTES: usize = 5 * 1024 * 1024;
const MAX_THUMBNAIL_DIMENSION: u32 = 4096;

#[tauri::command]
pub fn capture_project_thumbnail(
    png_bytes: Vec<u8>,
    view_type: String,
) -> Result<ProjectThumbnail, String> {
    if png_bytes.is_empty() || png_bytes.len() > MAX_THUMBNAIL_PNG_BYTES {
        return Err(format!(
            "Thumbnail PNG must be between 1 and {MAX_THUMBNAIL_PNG_BYTES} bytes"
        ));
    }
    if view_type.trim().is_empty() || view_type.len() > 64 {
        return Err("Invalid thumbnail view identifier".to_string());
    }
    let image = ImageReader::new(Cursor::new(&png_bytes))
        .with_guessed_format()
        .map_err(|error| error.to_string())?
        .decode()
        .map_err(|error| format!("Invalid thumbnail image: {error}"))?;
    let width = image.width();
    let height = image.height();
    if width == 0
        || height == 0
        || width > MAX_THUMBNAIL_DIMENSION
        || height > MAX_THUMBNAIL_DIMENSION
    {
        return Err("Thumbnail dimensions exceed safety limits".to_string());
    }
    Ok(ProjectThumbnail::from_png_bytes(
        &png_bytes,
        width,
        height,
        view_type,
    ))
}

#[tauri::command]
pub fn save_project_thumbnail(
    state: State<AppState>,
    thumbnail_data: ProjectThumbnail,
) -> Result<(), String> {
    let bytes = thumbnail_data.to_png_bytes().map_err(|error| error.to_string())?;
    if bytes.len() > MAX_THUMBNAIL_PNG_BYTES {
        return Err("Thumbnail exceeds safety limit".to_string());
    }
    let mut current = state.current_project.lock();
    let project = current
        .as_mut()
        .ok_or("No project open; create or load a project before saving a thumbnail")?;
    project.file.thumbnail = Some(thumbnail_data);
    project.file.metadata.modified_at = chrono::Utc::now();
    Ok(())
}

#[tauri::command]
pub fn get_project_thumbnail(state: State<AppState>) -> Option<ProjectThumbnail> {
    state
        .current_project
        .lock()
        .as_ref()
        .and_then(|project| project.file.thumbnail.clone())
}

#[tauri::command]
pub fn clear_project_thumbnail(state: State<AppState>) -> Result<(), String> {
    let mut current = state.current_project.lock();
    let project = current.as_mut().ok_or("No project open")?;
    project.file.thumbnail = None;
    project.file.metadata.modified_at = chrono::Utc::now();
    Ok(())
}

#[tauri::command]
pub fn load_project_thumbnail_from_path(
    project_path: String,
) -> Result<Option<ProjectThumbnail>, String> {
    let path = PathBuf::from(project_path);
    if path.join(PROJECT_V2_FILENAME).exists() {
        return load_project_v2(&path)
            .map(|document| document.thumbnail)
            .map_err(|error| error.to_string());
    }
    Project::load(&path)
        .map(|project| project.file.thumbnail)
        .map_err(|error| error.to_string())
}
