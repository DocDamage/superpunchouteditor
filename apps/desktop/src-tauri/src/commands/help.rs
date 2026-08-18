//! Help commands backed by packaged or embedded documentation.

use std::fs::OpenOptions;
use std::io::Write;

use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::help_system::{
    HelpArticle, HelpArticleSummary, HelpSystem, SearchResult,
};

#[tauri::command]
pub fn get_help_articles() -> Result<Vec<HelpArticleSummary>, String> {
    Ok(HelpSystem::load()?.get_articles())
}

#[tauri::command]
pub fn get_help_article(id: String) -> Result<Option<HelpArticle>, String> {
    let system = HelpSystem::load()?;
    Ok(system.get_article(&id).cloned())
}

#[tauri::command]
pub fn search_help(query: String) -> Result<Vec<SearchResult>, String> {
    Ok(HelpSystem::load()?.search(&query))
}

#[tauri::command]
pub fn get_context_help(context: String) -> Result<Vec<HelpArticleSummary>, String> {
    let system = HelpSystem::load()?;
    Ok(system
        .get_context_help(&context)
        .into_iter()
        .map(HelpArticleSummary::from)
        .collect())
}

#[derive(Serialize)]
struct HelpFeedbackRecord {
    article_id: String,
    helpful: bool,
    comment: Option<String>,
    recorded_at: String,
}

/// Store feedback locally. The application does not transmit help feedback implicitly.
#[tauri::command]
pub fn submit_help_feedback(
    app: AppHandle,
    article_id: String,
    helpful: bool,
    comment: Option<String>,
) -> Result<(), String> {
    if article_id.trim().is_empty() || article_id.len() > 128 {
        return Err("Invalid help article ID".to_string());
    }
    if comment.as_ref().is_some_and(|value| value.len() > 4_096) {
        return Err("Help feedback comment is too large".to_string());
    }

    let app_data = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&app_data).map_err(|e| e.to_string())?;
    let path = app_data.join("help-feedback.jsonl");
    let record = HelpFeedbackRecord {
        article_id,
        helpful,
        comment,
        recorded_at: Utc::now().to_rfc3339(),
    };
    let line = serde_json::to_string(&record).map_err(|e| e.to_string())?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| e.to_string())?;
    writeln!(file, "{}", line).map_err(|e| e.to_string())
}
