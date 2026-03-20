//! Tauri commands for Text/Dialog Editor
//!
//! Provides backend commands for editing in-game text including:
//! - Cornerman advice texts
//! - Boxer intros
//! - Victory/defeat quotes
//! - Menu text
//! - Credits

use serde::{Deserialize, Serialize};
use tauri::State;

use rom_core::{
    SpoTextEncoder, TextEncoder,
    text::{
        MenuCategory, MenuText, TextCondition, TextDatabase,
        TextPreviewRenderer, TextValidationSummary, VictoryCondition,
        MAX_CORNERMAN_TEXT_LENGTH, MAX_MENU_TEXT_LENGTH, MAX_VICTORY_QUOTE_LENGTH,
    },
    roster::{
        BoxerIntro, CornermanText, VictoryQuote,
    },
};

use crate::AppState;

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// Response for cornerman texts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CornermanTextsResponse {
    pub boxer_key: String,
    pub texts: Vec<CornermanTextDto>,
}

/// DTO for cornerman text (frontend-friendly)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CornermanTextDto {
    pub id: u8,
    pub boxer_key: String,
    pub round: u8,
    pub condition: String,
    pub condition_value: u8,
    pub text: String,
    pub byte_length: usize,
    pub max_length: usize,
    pub is_valid: bool,
}

impl From<CornermanText> for CornermanTextDto {
    fn from(text: CornermanText) -> Self {
        let encoder = SpoTextEncoder::new();
        let byte_length = encoder.encode(&text.text).len();
        let is_valid = byte_length <= text.max_length && encoder.can_encode(&text.text);

        Self {
            id: text.id,
            boxer_key: text.boxer_key,
            round: text.round,
            condition: text.condition.display_name().to_string(),
            condition_value: text.condition.to_byte(),
            text: text.text,
            byte_length,
            max_length: text.max_length,
            is_valid,
        }
    }
}

/// Request to update cornerman text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCornermanRequest {
    pub id: u8,
    pub text: String,
    pub condition: Option<u8>,
    pub round: Option<u8>,
}

/// Response for boxer intro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoxerIntroResponse {
    pub boxer_key: String,
    pub name_text: String,
    pub origin_text: String,
    pub record_text: String,
    pub rank_text: String,
    pub intro_quote: String,
    pub validation: IntroValidation,
}

/// Validation info for intro fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntroValidation {
    pub name_valid: bool,
    pub name_length: usize,
    pub origin_valid: bool,
    pub origin_length: usize,
    pub record_valid: bool,
    pub record_length: usize,
    pub rank_valid: bool,
    pub rank_length: usize,
    pub quote_valid: bool,
    pub quote_length: usize,
    pub all_valid: bool,
    pub unsupported_chars: Vec<char>,
}

/// Request to update boxer intro
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIntroRequest {
    pub boxer_key: String,
    pub name_text: Option<String>,
    pub origin_text: Option<String>,
    pub record_text: Option<String>,
    pub rank_text: Option<String>,
    pub intro_quote: Option<String>,
}

/// Response for victory quotes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictoryQuotesResponse {
    pub boxer_key: String,
    pub quotes: Vec<VictoryQuoteDto>,
}

/// DTO for victory quote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VictoryQuoteDto {
    pub id: u8,
    pub boxer_key: String,
    pub text: String,
    pub condition: String,
    pub condition_value: u8,
    pub is_loss_quote: bool,
    pub byte_length: usize,
    pub max_length: usize,
    pub is_valid: bool,
}

impl From<VictoryQuote> for VictoryQuoteDto {
    fn from(quote: VictoryQuote) -> Self {
        let encoder = SpoTextEncoder::new();
        let byte_length = encoder.encode(&quote.text).len();
        let is_valid = byte_length <= quote.max_length && encoder.can_encode(&quote.text);

        Self {
            id: quote.id,
            boxer_key: quote.boxer_key,
            text: quote.text,
            condition: (if quote.is_loss_quote { "Loss" } else { "Victory" }).to_string(),
            condition_value: if quote.is_loss_quote { 1 } else { 0 },
            is_loss_quote: quote.is_loss_quote,
            byte_length,
            max_length: quote.max_length,
            is_valid,
        }
    }
}

/// Request to update victory quote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVictoryQuoteRequest {
    pub id: u8,
    pub text: String,
}

/// Response for menu texts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuTextsResponse {
    pub category: Option<String>,
    pub texts: Vec<MenuTextDto>,
}

/// DTO for menu text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuTextDto {
    pub id: String,
    pub category: String,
    pub text: String,
    pub byte_length: usize,
    pub max_length: usize,
    pub is_valid: bool,
    pub is_modified: bool,
    pub is_shared: bool,
}

impl From<MenuText> for MenuTextDto {
    fn from(menu: MenuText) -> Self {
        let encoder = SpoTextEncoder::new();
        let byte_length = encoder.encode(&menu.text).len();
        let is_valid = byte_length <= menu.max_length && encoder.can_encode(&menu.text);

        Self {
            id: menu.id.clone(),
            category: menu.category.display_name().to_string(),
            text: menu.text.clone(),
            byte_length,
            max_length: menu.max_length,
            is_valid,
            is_modified: menu.is_modified(),
            is_shared: menu.is_shared,
        }
    }
}

/// Request to update menu text
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMenuTextRequest {
    pub id: String,
    pub text: String,
}

/// Text encoding info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEncodingInfoResponse {
    pub supported_chars: Vec<char>,
    pub max_cornerman_length: usize,
    pub max_victory_length: usize,
    pub max_menu_length: usize,
    pub max_intro_name_length: usize,
    pub max_intro_origin_length: usize,
    pub max_intro_record_length: usize,
    pub max_intro_rank_length: usize,
}

/// Text preview request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewTextRequest {
    pub text: String,
    pub font: String, // "spo_default", "spo_title", "spo_small"
    pub max_width: Option<usize>,
}

/// Text preview response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPreviewResponse {
    pub rendered_text: String,
    pub line_count: usize,
    pub fits_on_screen: bool,
    pub estimated_width: usize,
    pub estimated_height: usize,
}

/// Validation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateTextRequest {
    pub text: String,
    pub text_type: String, // "cornerman", "intro_name", "intro_origin", etc.
}

/// Validation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextValidationResponse {
    pub valid: bool,
    pub byte_length: usize,
    pub max_length: usize,
    pub can_encode: bool,
    pub unsupported_chars: Vec<char>,
    pub error: Option<String>,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Get text database (with defaults for now)
fn get_text_db() -> TextDatabase {
    TextDatabase::with_defaults()
}

/// Get encoder
fn get_encoder() -> SpoTextEncoder {
    SpoTextEncoder::new()
}

/// Validate text and return response
fn validate_text_internal(text: &str, max_length: usize) -> TextValidationResponse {
    let encoder = get_encoder();
    let encoded = encoder.encode(text);
    let byte_length = encoded.len();
    let can_encode = encoder.can_encode(text);
    let unsupported_chars = encoder.get_unsupported_chars(text);

    let mut error = None;
    if byte_length > max_length {
        error = Some(format!(
            "Text too long: {} bytes (max {})",
            byte_length, max_length
        ));
    } else if !can_encode {
        error = Some(format!("Unsupported characters: {:?}", unsupported_chars));
    }

    TextValidationResponse {
        valid: error.is_none() && can_encode,
        byte_length,
        max_length,
        can_encode,
        unsupported_chars,
        error,
    }
}

// ============================================================================
// TAURI COMMANDS - CORNERMAN TEXTS
// ============================================================================

// Note: get_cornerman_texts command is defined in roster_commands.rs

/// Get a single cornerman text by ID
#[tauri::command]
pub fn get_cornerman_text(
    _state: State<AppState>,
    id: u8,
) -> Result<Option<CornermanTextDto>, String> {
    let db = get_text_db();

    Ok(db.get_cornerman_text(id).map(|t| t.clone().into()))
}

/// Update a cornerman text
#[tauri::command]
pub fn update_cornerman_text(
    _state: State<AppState>,
    request: UpdateCornermanRequest,
) -> Result<CornermanTextDto, String> {
    let mut db = get_text_db();
    let encoder = get_encoder();

    let text = db
        .get_cornerman_text_mut(request.id)
        .ok_or_else(|| format!("Cornerman text with ID {} not found", request.id))?;

    // Update text
    text.text = request.text.clone();

    // Update condition if provided
    if let Some(condition_byte) = request.condition {
        text.condition = TextCondition::from_byte(condition_byte);
    }

    // Update round if provided
    if let Some(round) = request.round {
        text.round = round;
    }

    // Validate
    text.validate(&encoder).map_err(|e| e.to_string())?;

    Ok(text.clone().into())
}

/// Add a new cornerman text
#[tauri::command]
pub fn add_cornerman_text(
    _state: State<AppState>,
    boxer_key: String,
    text: String,
    condition: u8,
    round: u8,
) -> Result<CornermanTextDto, String> {
    let mut db = get_text_db();
    let encoder = get_encoder();

    let id = db.cornerman_texts.len() as u8;
    let mut new_text = CornermanText::new(id, boxer_key, text);
    new_text.condition = TextCondition::from_byte(condition);
    new_text.round = round;

    // Validate
    new_text.validate(&encoder).map_err(|e| e.to_string())?;

    db.add_cornerman_text(new_text.clone());

    Ok(new_text.into())
}

/// Delete a cornerman text
#[tauri::command]
pub fn delete_cornerman_text(_state: State<AppState>, id: u8) -> Result<(), String> {
    let mut db = get_text_db();

    db.remove_cornerman_text(id)
        .ok_or_else(|| format!("Cornerman text with ID {} not found", id))?;

    Ok(())
}

/// Get available text conditions (for dropdown)
#[tauri::command]
pub fn get_text_conditions() -> Vec<serde_json::Value> {
    TextCondition::all_conditions()
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "value": c.to_byte(),
                "label": c.display_name(),
            })
        })
        .collect()
}

// ============================================================================
// TAURI COMMANDS - BOXER INTROS
// ============================================================================

// Note: get_boxer_intro command is defined in roster_commands.rs

/// Validate intro fields and return validation info
fn validate_intro(intro: &BoxerIntro, encoder: &TextEncoder) -> IntroValidation {
    use rom_core::text::{
        MAX_INTRO_NAME_LENGTH, MAX_INTRO_ORIGIN_LENGTH, MAX_INTRO_RANK_LENGTH,
        MAX_INTRO_RECORD_LENGTH, MAX_VICTORY_QUOTE_LENGTH,
    };

    let name_len = encoder.encode(&intro.name_text).len();
    let origin_len = encoder.encode(&intro.origin_text).len();
    let record_len = encoder.encode(&intro.record_text).len();
    let rank_len = encoder.encode(&intro.rank_text).len();
    let quote_len = encoder.encode(&intro.intro_quote).len();

    let name_valid = name_len <= MAX_INTRO_NAME_LENGTH && encoder.can_encode(&intro.name_text);
    let origin_valid =
        origin_len <= MAX_INTRO_ORIGIN_LENGTH && encoder.can_encode(&intro.origin_text);
    let record_valid =
        record_len <= MAX_INTRO_RECORD_LENGTH && encoder.can_encode(&intro.record_text);
    let rank_valid = rank_len <= MAX_INTRO_RANK_LENGTH && encoder.can_encode(&intro.rank_text);
    let quote_valid =
        quote_len <= MAX_VICTORY_QUOTE_LENGTH && encoder.can_encode(&intro.intro_quote);

    // Collect all unsupported characters
    let mut unsupported: Vec<char> = Vec::new();
    unsupported.extend(encoder.get_unsupported_chars(&intro.name_text));
    unsupported.extend(encoder.get_unsupported_chars(&intro.origin_text));
    unsupported.extend(encoder.get_unsupported_chars(&intro.record_text));
    unsupported.extend(encoder.get_unsupported_chars(&intro.rank_text));
    unsupported.extend(encoder.get_unsupported_chars(&intro.intro_quote));
    unsupported.sort();
    unsupported.dedup();

    IntroValidation {
        name_valid,
        name_length: name_len,
        origin_valid,
        origin_length: origin_len,
        record_valid,
        record_length: record_len,
        rank_valid,
        rank_length: rank_len,
        quote_valid,
        quote_length: quote_len,
        all_valid: name_valid && origin_valid && record_valid && rank_valid && quote_valid,
        unsupported_chars: unsupported,
    }
}

/// Update boxer intro data
#[tauri::command]
pub fn update_boxer_intro(
    _state: State<AppState>,
    request: UpdateIntroRequest,
) -> Result<BoxerIntroResponse, String> {
    let mut db = get_text_db();
    let encoder = get_encoder();

    let intro = db
        .get_boxer_intro_mut(&request.boxer_key)
        .ok_or_else(|| format!("Boxer intro for '{}' not found", request.boxer_key))?;

    if let Some(name) = request.name_text {
        intro.name_text = name;
    }
    if let Some(origin) = request.origin_text {
        intro.origin_text = origin;
    }
    if let Some(record) = request.record_text {
        intro.record_text = record;
    }
    if let Some(rank) = request.rank_text {
        intro.rank_text = rank;
    }
    if let Some(quote) = request.intro_quote {
        intro.intro_quote = quote;
    }

    // Validate
    let errors = intro.validate(&encoder);
    if !errors.is_empty() {
        return Err(format!("Validation failed: {:?}", errors));
    }

    let validation = validate_intro(intro, &encoder);

    Ok(BoxerIntroResponse {
        boxer_key: request.boxer_key,
        name_text: intro.name_text.clone(),
        origin_text: intro.origin_text.clone(),
        record_text: intro.record_text.clone(),
        rank_text: intro.rank_text.clone(),
        intro_quote: intro.intro_quote.clone(),
        validation,
    })
}

// ============================================================================
// TAURI COMMANDS - VICTORY QUOTES
// ============================================================================

// Note: get_victory_quotes command is defined in roster_commands.rs

/// Update a victory quote
#[tauri::command]
pub fn update_victory_quote(
    _state: State<AppState>,
    request: UpdateVictoryQuoteRequest,
) -> Result<VictoryQuoteDto, String> {
    // TODO: Implement persistence
    // For now, just return a mock response

    let encoder = get_encoder();
    let encoded = encoder.encode(&request.text);

    if encoded.len() > MAX_VICTORY_QUOTE_LENGTH {
        return Err(format!(
            "Quote too long: {} bytes (max {})",
            encoded.len(),
            MAX_VICTORY_QUOTE_LENGTH
        ));
    }

    if !encoder.can_encode(&request.text) {
        return Err("Quote contains unsupported characters".to_string());
    }

    // Return mock DTO (actual implementation would update storage)
    Ok(VictoryQuoteDto {
        id: request.id,
        boxer_key: "mock".to_string(),
        text: request.text,
        condition: "Knockout".to_string(),
        condition_value: 0,
        is_loss_quote: false,
        byte_length: encoded.len(),
        max_length: MAX_VICTORY_QUOTE_LENGTH,
        is_valid: true,
    })
}

/// Get available victory conditions
#[tauri::command]
pub fn get_victory_conditions() -> Vec<serde_json::Value> {
    VictoryCondition::all_conditions()
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "value": c.to_byte(),
                "label": c.display_name(),
            })
        })
        .collect()
}

// ============================================================================
// TAURI COMMANDS - MENU TEXTS
// ============================================================================

/// Get menu texts
#[tauri::command]
pub fn get_menu_texts(
    _state: State<AppState>,
    category: Option<String>,
) -> Result<Vec<MenuTextDto>, String> {
    let db = get_text_db();

    let texts: Vec<MenuTextDto> = if let Some(cat) = category {
        // Parse category string
        let cat_enum = match cat.to_lowercase().as_str() {
            "mainmenu" => MenuCategory::MainMenu,
            "options" => MenuCategory::Options,
            "pausemenu" => MenuCategory::PauseMenu,
            "gameover" => MenuCategory::GameOver,
            "continueprompt" => MenuCategory::ContinuePrompt,
            "profile" => MenuCategory::Profile,
            _ => MenuCategory::MainMenu,
        };

        db.get_menu_texts(cat_enum)
            .into_iter()
            .map(|t| t.clone().into())
            .collect()
    } else {
        db.menu_texts.into_iter().map(|t| t.into()).collect()
    };

    Ok(texts)
}

/// Update a menu text
#[tauri::command]
pub fn update_menu_text(
    _state: State<AppState>,
    request: UpdateMenuTextRequest,
) -> Result<MenuTextDto, String> {
    let encoder = get_encoder();
    let encoded = encoder.encode(&request.text);

    if encoded.len() > MAX_MENU_TEXT_LENGTH {
        return Err(format!(
            "Menu text too long: {} bytes (max {})",
            encoded.len(),
            MAX_MENU_TEXT_LENGTH
        ));
    }

    if !encoder.can_encode(&request.text) {
        return Err("Menu text contains unsupported characters".to_string());
    }

    // Return mock DTO
    Ok(MenuTextDto {
        id: request.id,
        category: "Main Menu".to_string(),
        text: request.text,
        byte_length: encoded.len(),
        max_length: MAX_MENU_TEXT_LENGTH,
        is_valid: true,
        is_modified: true,
        is_shared: false,
    })
}

/// Get menu categories
#[tauri::command]
pub fn get_menu_categories() -> Vec<serde_json::Value> {
    MenuCategory::all_categories()
        .into_iter()
        .map(|c| {
            serde_json::json!({
                "value": format!("{:?}", c).to_lowercase(),
                "label": c.display_name(),
            })
        })
        .collect()
}

// ============================================================================
// TAURI COMMANDS - PREVIEW & VALIDATION
// ============================================================================

/// Preview how text will render in-game
#[tauri::command]
pub fn preview_text_render(
    _state: State<AppState>,
    request: PreviewTextRequest,
) -> Result<TextPreviewResponse, String> {
    let max_width = request.max_width.unwrap_or(28); // Default SPO text box width

    let rendered = TextPreviewRenderer::render_preview(&request.text, max_width);
    let line_count = rendered.lines().count();
    let fits = line_count <= 3; // Typically 3 lines max

    Ok(TextPreviewResponse {
        rendered_text: rendered.clone(),
        line_count,
        fits_on_screen: fits,
        estimated_width: TextPreviewRenderer::estimate_display_width(&request.text),
        estimated_height: line_count * 16, // Approximate pixel height per line
    })
}

/// Validate arbitrary text
#[tauri::command]
pub fn validate_text(
    _state: State<AppState>,
    request: ValidateTextRequest,
) -> Result<TextValidationResponse, String> {
    let max_length = match request.text_type.as_str() {
        "cornerman" => MAX_CORNERMAN_TEXT_LENGTH,
        "victory" => MAX_VICTORY_QUOTE_LENGTH,
        "menu" => MAX_MENU_TEXT_LENGTH,
        "intro_name" => rom_core::text::MAX_INTRO_NAME_LENGTH,
        "intro_origin" => rom_core::text::MAX_INTRO_ORIGIN_LENGTH,
        "intro_record" => rom_core::text::MAX_INTRO_RECORD_LENGTH,
        "intro_rank" => rom_core::text::MAX_INTRO_RANK_LENGTH,
        "intro_quote" => MAX_VICTORY_QUOTE_LENGTH,
        _ => MAX_CORNERMAN_TEXT_LENGTH,
    };

    Ok(validate_text_internal(&request.text, max_length))
}

/// Get detailed text encoding information for the text editor
#[tauri::command]
pub fn get_text_editor_encoding_info() -> TextEncodingInfoResponse {
    use rom_core::text::{
        MAX_INTRO_NAME_LENGTH, MAX_INTRO_ORIGIN_LENGTH, MAX_INTRO_RANK_LENGTH,
        MAX_INTRO_RECORD_LENGTH,
    };

    let encoder = get_encoder();

    TextEncodingInfoResponse {
        supported_chars: encoder.supported_chars(),
        max_cornerman_length: MAX_CORNERMAN_TEXT_LENGTH,
        max_victory_length: MAX_VICTORY_QUOTE_LENGTH,
        max_menu_length: MAX_MENU_TEXT_LENGTH,
        max_intro_name_length: MAX_INTRO_NAME_LENGTH,
        max_intro_origin_length: MAX_INTRO_ORIGIN_LENGTH,
        max_intro_record_length: MAX_INTRO_RECORD_LENGTH,
        max_intro_rank_length: MAX_INTRO_RANK_LENGTH,
    }
}

/// Encode text and return bytes (for debugging)
#[tauri::command]
pub fn encode_text(_state: State<AppState>, text: String) -> Result<Vec<u8>, String> {
    let encoder = get_encoder();
    Ok(encoder.encode(&text))
}

/// Decode bytes to text (for debugging)
#[tauri::command]
pub fn decode_text(_state: State<AppState>, bytes: Vec<u8>) -> Result<String, String> {
    let encoder = get_encoder();
    Ok(encoder.decode(&bytes))
}

// ============================================================================
// TAURI COMMANDS - BULK OPERATIONS
// ============================================================================

/// Validate all text in database
#[tauri::command]
pub fn validate_all_texts(_state: State<AppState>) -> Result<TextValidationSummary, String> {
    let db = get_text_db();
    let errors = db.validate();

    Ok(TextValidationSummary {
        total_entries: db.cornerman_texts.len()
            + db.boxer_intros.len()
            + db.victory_quotes.len()
            + db.menu_texts.len(),
        valid_entries: 0, // Would calculate based on validation
        invalid_entries: errors.len(),
        warnings: Vec::new(),
        errors,
    })
}

/// Search text database
#[tauri::command]
pub fn search_texts(
    _state: State<AppState>,
    query: String,
) -> Result<Vec<serde_json::Value>, String> {
    let db = get_text_db();
    let mut results = Vec::new();

    // Search cornerman texts
    for text in &db.cornerman_texts {
        if text.text.to_lowercase().contains(&query.to_lowercase()) {
            results.push(serde_json::json!({
                "type": "cornerman",
                "id": text.id,
                "boxer_key": text.boxer_key,
                "text_preview": &text.text[..text.text.len().min(50)],
            }));
        }
    }

    // Search victory quotes
    for quote in &db.victory_quotes {
        if quote.text.to_lowercase().contains(&query.to_lowercase()) {
            results.push(serde_json::json!({
                "type": "victory",
                "id": quote.id,
                "boxer_key": quote.boxer_key,
                "text_preview": &quote.text[..quote.text.len().min(50)],
            }));
        }
    }

    // Search menu texts
    for menu in &db.menu_texts {
        if menu.text.to_lowercase().contains(&query.to_lowercase()) {
            results.push(serde_json::json!({
                "type": "menu",
                "id": menu.id,
                "text_preview": &menu.text[..menu.text.len().min(50)],
            }));
        }
    }

    Ok(results)
}

/// Reset text to defaults
#[tauri::command]
pub fn reset_text_to_defaults(
    _state: State<AppState>,
    _text_type: String,
    _id: String,
) -> Result<(), String> {
    // TODO: Implement reset functionality
    // This would restore original ROM values
    Ok(())
}

/// Get text statistics
#[tauri::command]
pub fn get_text_statistics(_state: State<AppState>) -> Result<serde_json::Value, String> {
    let db = get_text_db();
    let encoder = get_encoder();

    // Calculate total bytes used by each category
    let cornerman_bytes: usize = db
        .cornerman_texts
        .iter()
        .map(|t| encoder.encode(&t.text).len())
        .sum();

    let victory_bytes: usize = db
        .victory_quotes
        .iter()
        .map(|q| encoder.encode(&q.text).len())
        .sum();

    let menu_bytes: usize = db
        .menu_texts
        .iter()
        .map(|m| encoder.encode(&m.text).len())
        .sum();

    Ok(serde_json::json!({
        "cornerman_count": db.cornerman_texts.len(),
        "cornerman_bytes": cornerman_bytes,
        "boxer_intro_count": db.boxer_intros.len(),
        "victory_quote_count": db.victory_quotes.len(),
        "victory_bytes": victory_bytes,
        "menu_text_count": db.menu_texts.len(),
        "menu_bytes": menu_bytes,
        "credits_line_count": db.credits_text.len(),
        "total_text_entries": db.cornerman_texts.len()
            + db.victory_quotes.len()
            + db.menu_texts.len()
            + db.credits_text.len(),
        "total_bytes_used": cornerman_bytes + victory_bytes + menu_bytes,
    }))
}
