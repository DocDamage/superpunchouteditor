//! Frame Tag Commands
//!
//! Commands for working with frame tags and annotations.
//! Tags and annotations are editor-side metadata stored in AppState (not in ROM).

use asset_core::frame_tags::{FrameAnnotation, FrameTag};
use tauri::State;

use crate::app_state::AppState;

/// Result type for frame tag operations
pub type FrameTagResult<T> = Result<T, String>;

// ============================================================================
// TAG DEFINITION COMMANDS
// ============================================================================

/// Get all available frame tag definitions
#[tauri::command]
pub fn get_frame_tags(state: State<AppState>) -> FrameTagResult<Vec<FrameTag>> {
    let manager = state.frame_tag_manager.lock();
    Ok(manager.get_all_tags().to_vec())
}

/// Add a new frame tag definition
#[tauri::command]
pub fn add_frame_tag(state: State<AppState>, tag: FrameTag) -> FrameTagResult<()> {
    let mut manager = state.frame_tag_manager.lock();
    manager.add_tag(tag);
    Ok(())
}

/// Remove a frame tag definition by ID
#[tauri::command]
pub fn remove_frame_tag(state: State<AppState>, tag_id: String) -> FrameTagResult<()> {
    let mut manager = state.frame_tag_manager.lock();
    manager.remove_tag(&tag_id);
    Ok(())
}

// ============================================================================
// ANNOTATION COMMANDS
// ============================================================================

/// Get the annotation for a specific frame of a boxer.
/// Returns null if no annotation exists yet.
#[tauri::command]
pub fn get_frame_annotation(
    state: State<AppState>,
    fighter_id: String,
    frame_index: usize,
) -> FrameTagResult<Option<FrameAnnotation>> {
    let manager = state.frame_tag_manager.lock();
    Ok(manager.get_frame_annotation(&fighter_id, frame_index).cloned())
}

/// Save (create or replace) the annotation for a specific frame.
#[tauri::command]
pub fn update_frame_annotation(
    state: State<AppState>,
    fighter_id: String,
    fighter_name: String,
    frame_index: usize,
    annotation: FrameAnnotation,
) -> FrameTagResult<()> {
    let mut manager = state.frame_tag_manager.lock();
    manager.update_frame_annotation(&fighter_id, &fighter_name, frame_index, annotation);
    Ok(())
}

/// Add a tag to a frame's annotation. Creates the annotation if it does not exist.
/// Returns the updated annotation.
#[tauri::command]
pub fn add_tag_to_frame(
    state: State<AppState>,
    fighter_id: String,
    fighter_name: String,
    frame_index: usize,
    tag_id: String,
) -> FrameTagResult<FrameAnnotation> {
    let mut manager = state.frame_tag_manager.lock();
    let boxer_annotations =
        manager.get_or_create_boxer_annotations(&fighter_id, &fighter_name);
    let annotation = boxer_annotations.get_or_create_annotation(frame_index);
    annotation.add_tag(&tag_id);
    Ok(annotation.clone())
}

/// Remove a tag from a frame's annotation.
/// Returns the updated annotation, or a default empty annotation if none existed.
#[tauri::command]
pub fn remove_tag_from_frame(
    state: State<AppState>,
    fighter_id: String,
    fighter_name: String,
    frame_index: usize,
    tag_id: String,
) -> FrameTagResult<FrameAnnotation> {
    let mut manager = state.frame_tag_manager.lock();
    let boxer_annotations =
        manager.get_or_create_boxer_annotations(&fighter_id, &fighter_name);
    let annotation = boxer_annotations.get_or_create_annotation(frame_index);
    annotation.remove_tag(&tag_id);
    Ok(annotation.clone())
}

/// Get all annotations for a boxer.
#[tauri::command]
pub fn get_boxer_annotations(
    state: State<AppState>,
    fighter_id: String,
) -> FrameTagResult<Vec<FrameAnnotation>> {
    let manager = state.frame_tag_manager.lock();
    if let Some(boxer_annotations) = manager.get_boxer_annotations(&fighter_id) {
        let mut annotations: Vec<FrameAnnotation> =
            boxer_annotations.frame_annotations.values().cloned().collect();
        annotations.sort_by_key(|a| a.frame_index);
        Ok(annotations)
    } else {
        Ok(vec![])
    }
}
