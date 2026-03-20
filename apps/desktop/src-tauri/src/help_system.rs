//! Help System for Super Punch-Out!! Editor
//!
//! Provides searchable documentation, contextual help, and article management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Categories for help articles
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HelpCategory {
    GettingStarted,
    Editing,
    Advanced,
    Troubleshooting,
}

impl HelpCategory {
    /// Get display name for the category
    pub fn display_name(&self) -> &'static str {
        match self {
            HelpCategory::GettingStarted => "Getting Started",
            HelpCategory::Editing => "Editing",
            HelpCategory::Advanced => "Advanced",
            HelpCategory::Troubleshooting => "Troubleshooting",
        }
    }

    /// Get icon for the category
    pub fn icon(&self) -> &'static str {
        match self {
            HelpCategory::GettingStarted => "🚀",
            HelpCategory::Editing => "✏️",
            HelpCategory::Advanced => "⚙️",
            HelpCategory::Troubleshooting => "🔧",
        }
    }
}

/// A help article with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelpArticle {
    pub id: String,
    pub title: String,
    pub category: HelpCategory,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub html_content: Option<String>,
    pub tags: Vec<String>,
    #[serde(skip)]
    pub word_count: usize,
}

impl HelpArticle {
    /// Create a new help article
    pub fn new(id: String, title: String, category: HelpCategory, content: String) -> Self {
        let word_count = content.split_whitespace().count();
        Self {
            id,
            title,
            category,
            content,
            html_content: None,
            tags: Vec::new(),
            word_count,
        }
    }

    /// Convert markdown content to HTML
    pub fn render_html(&mut self) {
        // Simple markdown to HTML conversion
        let html = markdown_to_html(&self.content);
        self.html_content = Some(html);
    }

    /// Add a tag to the article
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
        }
    }

    /// Get a summary of the article (first 150 characters)
    pub fn summary(&self) -> String {
        let plain_text = strip_markdown(&self.content);
        if plain_text.len() > 150 {
            format!("{}...", &plain_text[..150])
        } else {
            plain_text
        }
    }
}

/// Summary of a help article for lists
#[derive(Debug, Clone, Serialize)]
pub struct HelpArticleSummary {
    pub id: String,
    pub title: String,
    pub category: HelpCategory,
    pub summary: String,
    pub tags: Vec<String>,
}

impl From<&HelpArticle> for HelpArticleSummary {
    fn from(article: &HelpArticle) -> Self {
        Self {
            id: article.id.clone(),
            title: article.title.clone(),
            category: article.category,
            summary: article.summary(),
            tags: article.tags.clone(),
        }
    }
}

/// Search result with relevance score
#[derive(Debug, Clone, Serialize)]
pub struct SearchResult {
    #[serde(flatten)]
    pub article: HelpArticleSummary,
    pub relevance: f32,
    pub matched_terms: Vec<String>,
}

/// Search index for fast full-text search
#[derive(Debug, Default)]
pub struct SearchIndex {
    /// Maps terms to article IDs with frequency
    term_index: HashMap<String, Vec<(String, f32)>>,
    /// Article titles for boosting
    title_index: HashMap<String, String>,
}

impl SearchIndex {
    /// Create a new empty search index
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an article to the index
    pub fn add_article(&mut self, article: &HelpArticle) {
        // Index title with higher weight
        let title_terms = tokenize(&article.title);
        for term in &title_terms {
            let entry = self.term_index.entry(term.clone()).or_default();
            if !entry.iter().any(|(id, _)| id == &article.id) {
                entry.push((article.id.clone(), 3.0)); // Title weight = 3x
            }
        }

        // Index content
        let content_terms = tokenize(&article.content);
        let term_count = content_terms.len() as f32;
        let mut term_freq: HashMap<String, usize> = HashMap::new();

        for term in content_terms {
            *term_freq.entry(term).or_default() += 1;
        }

        for (term, count) in term_freq {
            let tf = count as f32 / term_count;
            let entry = self.term_index.entry(term).or_default();
            if let Some(existing) = entry.iter_mut().find(|(id, _)| id == &article.id) {
                existing.1 += tf;
            } else {
                entry.push((article.id.clone(), tf));
            }
        }

        // Store title for display
        self.title_index
            .insert(article.id.clone(), article.title.clone());
    }

    /// Search the index for a query
    pub fn search(&self, query: &str, articles: &[HelpArticle]) -> Vec<SearchResult> {
        let query_terms = tokenize(query);
        let mut scores: HashMap<String, f32> = HashMap::new();
        let mut matched_terms: HashMap<String, Vec<String>> = HashMap::new();

        for term in &query_terms {
            if let Some(matches) = self.term_index.get(term) {
                for (article_id, weight) in matches {
                    *scores.entry(article_id.clone()).or_default() += weight;
                    matched_terms
                        .entry(article_id.clone())
                        .or_default()
                        .push(term.clone());
                }
            }
        }

        // Build results
        let mut results: Vec<SearchResult> = scores
            .into_iter()
            .filter_map(|(id, score)| {
                articles
                    .iter()
                    .find(|a| a.id == id)
                    .map(|article| SearchResult {
                        article: HelpArticleSummary::from(article),
                        relevance: score,
                        matched_terms: matched_terms.get(&id).cloned().unwrap_or_default(),
                    })
            })
            .collect();

        // Sort by relevance (descending)
        // Using unwrap_or(Ordering::Equal) to handle potential NaN values safely
        results.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }
}

/// The main help system
#[derive(Debug)]
pub struct HelpSystem {
    articles: Vec<HelpArticle>,
    index: SearchIndex,
    context_mappings: HashMap<String, Vec<String>>,
}

impl HelpSystem {
    /// Load the help system from documentation files
    pub fn load() -> Result<Self, String> {
        let mut system = Self {
            articles: Vec::new(),
            index: SearchIndex::new(),
            context_mappings: Self::default_context_mappings(),
        };

        // Determine the docs directory path
        let docs_dir = Self::docs_directory()?;

        if !docs_dir.exists() {
            // Create embedded articles if docs directory doesn't exist
            system.load_embedded_articles();
        } else {
            // Load from files
            system.load_articles_from_directory(&docs_dir)?;
        }

        // Build search index
        for article in &system.articles {
            system.index.add_article(article);
        }

        Ok(system)
    }

    /// Get the documentation directory path
    fn docs_directory() -> Result<PathBuf, String> {
        // Try to find docs relative to the executable
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                // Check for docs in src-tauri/docs (development)
                let dev_path = exe_dir
                    .parent()
                    .and_then(|p| p.parent())
                    .map(|p| p.join("src-tauri").join("docs"));

                if let Some(ref path) = dev_path {
                    if path.exists() {
                        return Ok(path.clone());
                    }
                }

                // Check for bundled docs
                let bundled_path = exe_dir.join("docs");
                if bundled_path.exists() {
                    return Ok(bundled_path);
                }
            }
        }

        // Fallback: try current directory
        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;

        let docs_path = current_dir.join("docs");
        if docs_path.exists() {
            return Ok(docs_path);
        }

        // Return the path anyway - we'll use embedded content
        Ok(docs_path)
    }

    /// Load articles from markdown files in a directory
    fn load_articles_from_directory(&mut self, dir: &PathBuf) -> Result<(), String> {
        let entries =
            fs::read_dir(dir).map_err(|e| format!("Failed to read docs directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                let content = fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

                let id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let (title, category, body) = Self::parse_markdown(&content);
                let tags = Self::extract_tags(&body);

                let mut article = HelpArticle::new(id, title, category, body);
                article.tags = tags;
                article.render_html();

                self.articles.push(article);
            }
        }

        Ok(())
    }

    /// Parse markdown content to extract metadata
    fn parse_markdown(content: &str) -> (String, HelpCategory, String) {
        let lines: Vec<&str> = content.lines().collect();

        // Extract title from first h1
        let title = lines
            .iter()
            .find(|line| line.starts_with("# "))
            .map(|line| line[2..].trim().to_string())
            .unwrap_or_else(|| "Untitled".to_string());

        // Determine category from content
        let category = if content.contains("troubleshoot") || content.contains("error") {
            HelpCategory::Troubleshooting
        } else if content.contains("advanced") || content.contains("technical") {
            HelpCategory::Advanced
        } else if content.contains("getting started") || content.contains("introduction") {
            HelpCategory::GettingStarted
        } else {
            HelpCategory::Editing
        };

        (title, category, content.to_string())
    }

    /// Extract tags from content
    fn extract_tags(content: &str) -> Vec<String> {
        let mut tags = Vec::new();

        // Look for tags in the content
        let tag_keywords = vec![
            ("palette", "palette"),
            ("sprite", "sprite"),
            ("animation", "animation"),
            ("frame", "frame"),
            ("rom", "rom"),
            ("export", "export"),
            ("import", "import"),
            ("script", "script"),
            ("fighter", "fighter"),
            ("boxer", "boxer"),
            ("keyboard", "shortcut"),
            ("shortcut", "shortcut"),
        ];

        for (keyword, tag) in tag_keywords {
            if content.to_lowercase().contains(keyword) && !tags.contains(&tag.to_string()) {
                tags.push(tag.to_string());
            }
        }

        tags
    }

    /// Load embedded articles as fallback
    fn load_embedded_articles(&mut self) {
        // These are loaded as fallback if file system docs aren't available
        let embedded = vec![
            (
                "getting-started",
                "Getting Started",
                HelpCategory::GettingStarted,
                include_str!("../docs/getting-started.md"),
            ),
            (
                "rom-validation",
                "ROM Validation",
                HelpCategory::GettingStarted,
                include_str!("../docs/rom-validation.md"),
            ),
            (
                "palette-editing",
                "Palette Editing",
                HelpCategory::Editing,
                include_str!("../docs/palette-editing.md"),
            ),
            (
                "sprite-editing",
                "Sprite Editing",
                HelpCategory::Editing,
                include_str!("../docs/sprite-editing.md"),
            ),
            (
                "animation-editor",
                "Animation Editor",
                HelpCategory::Editing,
                include_str!("../docs/animation-editor.md"),
            ),
            (
                "frame-reconstructor",
                "Frame Reconstructor",
                HelpCategory::Advanced,
                include_str!("../docs/frame-reconstructor.md"),
            ),
            (
                "script-editing",
                "Script Editing",
                HelpCategory::Advanced,
                include_str!("../docs/script-editing.md"),
            ),
            (
                "patch-export",
                "Patch Export",
                HelpCategory::Editing,
                include_str!("../docs/patch-export.md"),
            ),
            (
                "troubleshooting",
                "Troubleshooting",
                HelpCategory::Troubleshooting,
                include_str!("../docs/troubleshooting.md"),
            ),
            (
                "keyboard-shortcuts",
                "Keyboard Shortcuts",
                HelpCategory::GettingStarted,
                include_str!("../docs/keyboard-shortcuts.md"),
            ),
        ];

        for (id, title, category, content) in embedded {
            let tags = Self::extract_tags(content);
            let mut article = HelpArticle::new(
                id.to_string(),
                title.to_string(),
                category,
                content.to_string(),
            );
            article.tags = tags;
            article.render_html();
            self.articles.push(article);
        }
    }

    /// Default context-to-article mappings
    fn default_context_mappings() -> HashMap<String, Vec<String>> {
        let mut mappings = HashMap::new();

        mappings.insert(
            "palette-editor".to_string(),
            vec!["palette-editing".to_string(), "getting-started".to_string()],
        );

        mappings.insert(
            "sprite-editor".to_string(),
            vec!["sprite-editing".to_string(), "palette-editing".to_string()],
        );

        mappings.insert(
            "animation-editor".to_string(),
            vec![
                "animation-editor".to_string(),
                "frame-reconstructor".to_string(),
            ],
        );

        mappings.insert(
            "frame-reconstructor".to_string(),
            vec![
                "frame-reconstructor".to_string(),
                "sprite-editing".to_string(),
            ],
        );

        mappings.insert(
            "script-editor".to_string(),
            vec!["script-editing".to_string(), "troubleshooting".to_string()],
        );

        mappings.insert(
            "project-manager".to_string(),
            vec!["patch-export".to_string(), "getting-started".to_string()],
        );

        mappings.insert(
            "export-panel".to_string(),
            vec!["patch-export".to_string(), "troubleshooting".to_string()],
        );

        mappings.insert(
            "rom-loading".to_string(),
            vec!["rom-validation".to_string(), "getting-started".to_string()],
        );

        mappings.insert(
            "keyboard".to_string(),
            vec!["keyboard-shortcuts".to_string()],
        );

        mappings
    }

    /// Get all articles as summaries
    pub fn get_articles(&self) -> Vec<HelpArticleSummary> {
        self.articles.iter().map(HelpArticleSummary::from).collect()
    }

    /// Get articles by category
    pub fn get_articles_by_category(&self, category: HelpCategory) -> Vec<HelpArticleSummary> {
        self.articles
            .iter()
            .filter(|a| a.category == category)
            .map(HelpArticleSummary::from)
            .collect()
    }

    /// Get a specific article by ID
    pub fn get_article(&self, id: &str) -> Option<&HelpArticle> {
        self.articles.iter().find(|a| a.id == id)
    }

    /// Search for articles
    pub fn search(&self, query: &str) -> Vec<SearchResult> {
        if query.trim().is_empty() {
            return Vec::new();
        }
        self.index.search(query, &self.articles)
    }

    /// Get contextual help for a specific context
    pub fn get_context_help(&self, context: &str) -> Vec<&HelpArticle> {
        if let Some(article_ids) = self.context_mappings.get(context) {
            article_ids
                .iter()
                .filter_map(|id| self.get_article(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all categories with counts
    pub fn get_categories(&self) -> Vec<(HelpCategory, usize)> {
        let mut counts: HashMap<HelpCategory, usize> = HashMap::new();

        for article in &self.articles {
            *counts.entry(article.category).or_default() += 1;
        }

        let mut result: Vec<_> = counts.into_iter().collect();
        result.sort_by_key(|(cat, _)| *cat);
        result
    }

    /// Add a custom context mapping
    pub fn add_context_mapping(&mut self, context: String, article_ids: Vec<String>) {
        self.context_mappings.insert(context, article_ids);
    }
}

/// Tokenize text for search indexing
fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| !s.is_empty() && s.len() > 1)
        .map(|s| s.to_string())
        .collect()
}

/// Simple markdown to HTML conversion
fn markdown_to_html(markdown: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut in_list = false;

    for line in markdown.lines() {
        let trimmed = line.trim();

        // Code blocks
        if trimmed.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
                in_code_block = false;
            } else {
                html.push_str("<pre><code>");
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(&html_escape(line));
            html.push('\n');
            continue;
        }

        // Headers
        if trimmed.starts_with("# ") {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str(&format!("<h1>{}</h1>\n", html_escape(&trimmed[2..])));
        } else if trimmed.starts_with("## ") {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str(&format!("<h2>{}</h2>\n", html_escape(&trimmed[3..])));
        } else if trimmed.starts_with("### ") {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str(&format!("<h3>{}</h3>\n", html_escape(&trimmed[4..])));
        }
        // Lists
        else if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            if !in_list {
                html.push_str("<ul>\n");
                in_list = true;
            }
            html.push_str(&format!("<li>{}</li>\n", inline_format(&trimmed[2..])));
        }
        // Empty line
        else if trimmed.is_empty() {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str("<br>\n");
        }
        // Paragraph
        else {
            if in_list {
                html.push_str("</ul>\n");
                in_list = false;
            }
            html.push_str(&format!("<p>{}</p>\n", inline_format(trimmed)));
        }
    }

    if in_list {
        html.push_str("</ul>\n");
    }
    if in_code_block {
        html.push_str("</code></pre>\n");
    }

    html
}

/// Format inline markdown elements
fn inline_format(text: &str) -> String {
    let mut result = html_escape(text);

    // Bold
    result = result.replace("**", "<strong>");
    // Note: This is simplified - proper implementation needs to handle pairs

    // Italic
    result = result.replace('*', "<em>");

    // Inline code
    result = result.replace('`', "<code>");

    result
}

/// Escape HTML special characters
fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Strip markdown formatting for plain text summaries
fn strip_markdown(markdown: &str) -> String {
    markdown
        .replace("# ", "")
        .replace("## ", "")
        .replace("### ", "")
        .replace("**", "")
        .replace('*', "")
        .replace('`', "")
        .replace("[", "")
        .replace("](", " (")
        .replace(')', " ")
        .replace("- ", "")
        .replace("\n\n", " ")
        .replace('\n', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_category_display() {
        assert_eq!(
            HelpCategory::GettingStarted.display_name(),
            "Getting Started"
        );
        assert_eq!(HelpCategory::Editing.display_name(), "Editing");
    }

    #[test]
    fn test_article_summary() {
        let article = HelpArticle::new(
            "test".to_string(),
            "Test Article".to_string(),
            HelpCategory::GettingStarted,
            "This is a test article with some content that should be summarized.".to_string(),
        );

        let summary = article.summary();
        assert!(!summary.is_empty());
        assert!(summary.len() <= 153); // 150 + "..."
    }

    #[test]
    fn test_markdown_to_html() {
        let md = "# Title\n\nThis is a paragraph.\n\n- Item 1\n- Item 2";
        let html = markdown_to_html(md);
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<p>"));
        assert!(html.contains("<ul>"));
        assert!(html.contains("<li>"));
    }
}
