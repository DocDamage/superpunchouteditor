use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Category for frame tags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TagCategory {
    Idle,
    Attack,
    Defense,
    Damage,
    Knockdown,
    Special,
    Misc,
}

impl TagCategory {
    /// Get the display name for this category
    pub fn display_name(&self) -> &'static str {
        match self {
            TagCategory::Idle => "Idle",
            TagCategory::Attack => "Attack",
            TagCategory::Defense => "Defense",
            TagCategory::Damage => "Damage",
            TagCategory::Knockdown => "Knockdown",
            TagCategory::Special => "Special",
            TagCategory::Misc => "Misc",
        }
    }

    /// Get the default color for this category
    pub fn default_color(&self) -> &'static str {
        match self {
            TagCategory::Idle => "#4ade80",      // Green
            TagCategory::Attack => "#f87171",    // Red
            TagCategory::Defense => "#60a5fa",   // Blue
            TagCategory::Damage => "#fbbf24",    // Yellow
            TagCategory::Knockdown => "#a78bfa", // Purple
            TagCategory::Special => "#f472b6",   // Pink
            TagCategory::Misc => "#9ca3af",      // Gray
        }
    }

    /// Get an icon for this category
    pub fn icon(&self) -> &'static str {
        match self {
            TagCategory::Idle => "😐",
            TagCategory::Attack => "👊",
            TagCategory::Defense => "🛡️",
            TagCategory::Damage => "💥",
            TagCategory::Knockdown => "💫",
            TagCategory::Special => "✨",
            TagCategory::Misc => "📌",
        }
    }
}

impl Default for TagCategory {
    fn default() -> Self {
        TagCategory::Misc
    }
}

/// A frame tag definition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FrameTag {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub category: TagCategory,
    pub color: String,
    pub created_at: String,
    pub created_by: String,
}

impl FrameTag {
    /// Create a new frame tag
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        display_name: impl Into<String>,
        category: TagCategory,
    ) -> Self {
        let id = id.into();
        let name = name.into();
        let display_name = display_name.into();
        let color = category.default_color().to_string();

        Self {
            id: id.clone(),
            name,
            display_name,
            description: String::new(),
            category,
            color,
            created_at: chrono::Utc::now().to_rfc3339(),
            created_by: "user".to_string(),
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set a custom color
    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = color.into();
        self
    }
}

/// Annotation for a specific frame
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct FrameAnnotation {
    pub frame_index: usize,
    pub tags: Vec<String>, // Tag IDs
    pub notes: String,     // Freeform notes
    pub hitbox_description: Option<String>,
}

impl FrameAnnotation {
    /// Create a new frame annotation
    pub fn new(frame_index: usize) -> Self {
        Self {
            frame_index,
            tags: Vec::new(),
            notes: String::new(),
            hitbox_description: None,
        }
    }

    /// Add a tag to this annotation
    pub fn add_tag(&mut self, tag_id: impl Into<String>) {
        let tag_id = tag_id.into();
        if !self.tags.contains(&tag_id) {
            self.tags.push(tag_id);
        }
    }

    /// Remove a tag from this annotation
    pub fn remove_tag(&mut self, tag_id: &str) {
        self.tags.retain(|t| t != tag_id);
    }

    /// Check if this annotation has a specific tag
    pub fn has_tag(&self, tag_id: &str) -> bool {
        self.tags.contains(&tag_id.to_string())
    }

    /// Set notes
    pub fn set_notes(&mut self, notes: impl Into<String>) {
        self.notes = notes.into();
    }

    /// Set hitbox description
    pub fn set_hitbox_description(&mut self, description: impl Into<String>) {
        self.hitbox_description = Some(description.into());
    }
}

/// Collection of annotations for a boxer
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct BoxerAnnotations {
    pub boxer_id: String,
    pub boxer_name: String,
    /// Map of frame index (as string) to annotation
    pub frame_annotations: HashMap<String, FrameAnnotation>,
}

/// Deprecated: Use `BoxerAnnotations` instead
#[deprecated(since = "0.1.0", note = "Use BoxerAnnotations instead")]
pub type FighterAnnotations = BoxerAnnotations;

impl BoxerAnnotations {
    /// Create a new boxer annotations collection
    pub fn new(boxer_id: impl Into<String>, boxer_name: impl Into<String>) -> Self {
        Self {
            boxer_id: boxer_id.into(),
            boxer_name: boxer_name.into(),
            frame_annotations: HashMap::new(),
        }
    }

    /// Get or create an annotation for a frame
    pub fn get_or_create_annotation(&mut self, frame_index: usize) -> &mut FrameAnnotation {
        let key = frame_index.to_string();
        self.frame_annotations
            .entry(key)
            .or_insert_with(|| FrameAnnotation::new(frame_index))
    }

    /// Get an annotation for a frame (immutable)
    pub fn get_annotation(&self, frame_index: usize) -> Option<&FrameAnnotation> {
        self.frame_annotations.get(&frame_index.to_string())
    }

    /// Remove an annotation for a frame
    pub fn remove_annotation(&mut self, frame_index: usize) {
        self.frame_annotations.remove(&frame_index.to_string());
    }

    /// Find all frames with a specific tag
    pub fn find_frames_by_tag(&self, tag_id: &str) -> Vec<usize> {
        self.frame_annotations
            .iter()
            .filter(|(_, annotation)| annotation.has_tag(tag_id))
            .map(|(key, _)| key.parse::<usize>().unwrap_or(0))
            .collect()
    }

    /// Search annotations by notes content
    pub fn search_by_notes(&self, query: &str) -> Vec<(usize, &FrameAnnotation)> {
        let query_lower = query.to_lowercase();
        self.frame_annotations
            .iter()
            .filter(|(_, annotation)| annotation.notes.to_lowercase().contains(&query_lower))
            .map(|(key, annotation)| (key.parse::<usize>().unwrap_or(0), annotation))
            .collect()
    }
}

/// Manager for frame tags and annotations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FrameTagManager {
    /// All defined tags
    pub tags: Vec<FrameTag>,
    /// Annotations per boxer (boxer_id -> BoxerAnnotations)
    pub boxer_annotations: HashMap<String, BoxerAnnotations>,
}

impl FrameTagManager {
    /// Get annotations per boxer (deprecated field access)
    #[deprecated(since = "0.1.0", note = "Use boxer_annotations instead")]
    pub fn fighter_annotations(&self) -> &HashMap<String, BoxerAnnotations> {
        &self.boxer_annotations
    }
}

impl FrameTagManager {
    /// Create a new tag manager with predefined tags
    pub fn with_default_tags() -> Self {
        let mut manager = Self::default();
        manager.add_default_tags();
        manager
    }

    /// Add all default Super Punch-Out!! tags
    pub fn add_default_tags(&mut self) {
        let default_tags = vec![
            // Idle tags
            FrameTag::new(
                "idle_stance",
                "idle_stance",
                "Idle Stance",
                TagCategory::Idle,
            )
            .with_description("Neutral standing pose"),
            FrameTag::new("idle_alt", "idle_alt", "Idle Alt", TagCategory::Idle)
                .with_description("Alternative idle pose"),
            // Attack tags
            FrameTag::new("left_jab", "left_jab", "Left Jab", TagCategory::Attack)
                .with_description("Quick left jab attack"),
            FrameTag::new("right_jab", "right_jab", "Right Jab", TagCategory::Attack)
                .with_description("Quick right jab attack"),
            FrameTag::new("left_hook", "left_hook", "Left Hook", TagCategory::Attack)
                .with_description("Powerful left hook attack"),
            FrameTag::new(
                "right_hook",
                "right_hook",
                "Right Hook",
                TagCategory::Attack,
            )
            .with_description("Powerful right hook attack"),
            FrameTag::new("uppercut", "uppercut", "Uppercut", TagCategory::Attack)
                .with_description("Uppercut attack"),
            // Defense tags
            FrameTag::new(
                "dodge_left",
                "dodge_left",
                "Dodge Left",
                TagCategory::Defense,
            )
            .with_description("Dodge to the left"),
            FrameTag::new(
                "dodge_right",
                "dodge_right",
                "Dodge Right",
                TagCategory::Defense,
            )
            .with_description("Dodge to the right"),
            FrameTag::new("duck", "duck", "Duck", TagCategory::Defense)
                .with_description("Duck down to avoid high attacks"),
            FrameTag::new("block", "block", "Block", TagCategory::Defense)
                .with_description("Generic block pose"),
            FrameTag::new(
                "block_high",
                "block_high",
                "Block High",
                TagCategory::Defense,
            )
            .with_description("Block high attacks"),
            FrameTag::new("block_low", "block_low", "Block Low", TagCategory::Defense)
                .with_description("Block low attacks"),
            // Damage tags
            FrameTag::new("hit_face", "hit_face", "Hit Face", TagCategory::Damage)
                .with_description("Hit in the face"),
            FrameTag::new("hit_body", "hit_body", "Hit Body", TagCategory::Damage)
                .with_description("Hit in the body"),
            FrameTag::new(
                "knockdown",
                "knockdown",
                "Knockdown",
                TagCategory::Knockdown,
            )
            .with_description("Getting knocked down"),
            FrameTag::new("getup", "getup", "Get Up", TagCategory::Knockdown)
                .with_description("Getting up from knockdown"),
            // Special tags
            FrameTag::new("taunt", "taunt", "Taunt", TagCategory::Special)
                .with_description("Taunting the player"),
            FrameTag::new(
                "special_move",
                "special_move",
                "Special Move",
                TagCategory::Special,
            )
            .with_description("Special attack or move"),
            // Misc tags
            FrameTag::new("neutral", "neutral", "Neutral", TagCategory::Misc)
                .with_description("Neutral expression/stance"),
            FrameTag::new("intro", "intro", "Intro", TagCategory::Misc)
                .with_description("Introduction pose"),
            FrameTag::new("victory", "victory", "Victory", TagCategory::Misc)
                .with_description("Victory celebration"),
            FrameTag::new("defeat", "defeat", "Defeat", TagCategory::Misc)
                .with_description("Defeat pose"),
        ];

        for tag in default_tags {
            self.add_tag(tag);
        }
    }

    /// Add a tag
    pub fn add_tag(&mut self, tag: FrameTag) {
        // Remove existing tag with same ID if present
        self.tags.retain(|t| t.id != tag.id);
        self.tags.push(tag);
    }

    /// Remove a tag by ID
    pub fn remove_tag(&mut self, tag_id: &str) {
        self.tags.retain(|t| t.id != tag_id);
        // Also remove from all annotations
        for (_, boxer_annotations) in &mut self.boxer_annotations {
            for (_, annotation) in &mut boxer_annotations.frame_annotations {
                annotation.remove_tag(tag_id);
            }
        }
    }

    /// Get a tag by ID
    pub fn get_tag(&self, tag_id: &str) -> Option<&FrameTag> {
        self.tags.iter().find(|t| t.id == tag_id)
    }

    /// Get all tags
    pub fn get_all_tags(&self) -> &[FrameTag] {
        &self.tags
    }

    /// Get tags by category
    pub fn get_tags_by_category(&self, category: TagCategory) -> Vec<&FrameTag> {
        self.tags
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    /// Get or create annotations for a boxer
    pub fn get_or_create_boxer_annotations(
        &mut self,
        boxer_id: impl Into<String>,
        boxer_name: impl Into<String>,
    ) -> &mut BoxerAnnotations {
        let boxer_id = boxer_id.into();
        let boxer_name = boxer_name.into();

        self.boxer_annotations
            .entry(boxer_id.clone())
            .or_insert_with(|| BoxerAnnotations::new(boxer_id, boxer_name))
    }

    /// Deprecated: Use `get_or_create_boxer_annotations` instead
    #[deprecated(since = "0.1.0", note = "Use get_or_create_boxer_annotations instead")]
    pub fn get_or_create_fighter_annotations(
        &mut self,
        fighter_id: impl Into<String>,
        fighter_name: impl Into<String>,
    ) -> &mut BoxerAnnotations {
        self.get_or_create_boxer_annotations(fighter_id, fighter_name)
    }

    /// Get annotations for a boxer
    pub fn get_boxer_annotations(&self, boxer_id: &str) -> Option<&BoxerAnnotations> {
        self.boxer_annotations.get(boxer_id)
    }

    /// Deprecated: Use `get_boxer_annotations` instead
    #[deprecated(since = "0.1.0", note = "Use get_boxer_annotations instead")]
    pub fn get_fighter_annotations(&self, fighter_id: &str) -> Option<&BoxerAnnotations> {
        self.get_boxer_annotations(fighter_id)
    }

    /// Get annotation for a specific frame
    pub fn get_frame_annotation(
        &self,
        boxer_id: &str,
        frame_index: usize,
    ) -> Option<&FrameAnnotation> {
        self.boxer_annotations
            .get(boxer_id)
            .and_then(|ba| ba.get_annotation(frame_index))
    }

    /// Update annotation for a specific frame
    pub fn update_frame_annotation(
        &mut self,
        boxer_id: impl Into<String>,
        boxer_name: impl Into<String>,
        frame_index: usize,
        annotation: FrameAnnotation,
    ) {
        let boxer_id = boxer_id.into();
        let boxer_annotations = self.get_or_create_boxer_annotations(boxer_id, boxer_name);
        boxer_annotations
            .frame_annotations
            .insert(frame_index.to_string(), annotation);
    }

    /// Search frames by tag
    pub fn search_frames_by_tag(&self, boxer_id: &str, tag_id: &str) -> Vec<usize> {
        self.boxer_annotations
            .get(boxer_id)
            .map(|ba| ba.find_frames_by_tag(tag_id))
            .unwrap_or_default()
    }

    /// Export to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Import from JSON string
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Merge another manager into this one (for loading project files)
    pub fn merge(&mut self, other: FrameTagManager) {
        // Merge tags (other takes precedence for same IDs)
        for tag in other.tags {
            self.add_tag(tag);
        }

        // Merge annotations
        for (boxer_id, annotations) in other.boxer_annotations {
            if let Some(existing) = self.boxer_annotations.get_mut(&boxer_id) {
                // Merge frame annotations
                for (frame_key, annotation) in annotations.frame_annotations {
                    existing.frame_annotations.insert(frame_key, annotation);
                }
            } else {
                self.boxer_annotations.insert(boxer_id, annotations);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_tag_creation() {
        let tag = FrameTag::new("test", "test_tag", "Test Tag", TagCategory::Idle);
        assert_eq!(tag.id, "test");
        assert_eq!(tag.name, "test_tag");
        assert_eq!(tag.display_name, "Test Tag");
        assert_eq!(tag.category, TagCategory::Idle);
    }

    #[test]
    fn test_frame_annotation() {
        let mut annotation = FrameAnnotation::new(5);
        assert_eq!(annotation.frame_index, 5);
        assert!(annotation.tags.is_empty());

        annotation.add_tag("idle_stance");
        annotation.add_tag("idle_stance"); // Duplicate should not be added
        assert_eq!(annotation.tags.len(), 1);
        assert!(annotation.has_tag("idle_stance"));

        annotation.remove_tag("idle_stance");
        assert!(!annotation.has_tag("idle_stance"));
    }

    #[test]
    fn test_tag_manager() {
        let mut manager = FrameTagManager::with_default_tags();
        assert!(!manager.tags.is_empty());

        // Test adding a custom tag
        let custom_tag = FrameTag::new("custom", "custom_tag", "Custom Tag", TagCategory::Misc);
        manager.add_tag(custom_tag);
        assert!(manager.get_tag("custom").is_some());

        // Test annotations
        let annotation = FrameAnnotation::new(0);
        manager.update_frame_annotation("boxer_0", "Gabby Jay", 0, annotation);

        assert!(manager.get_frame_annotation("boxer_0", 0).is_some());
    }
}
