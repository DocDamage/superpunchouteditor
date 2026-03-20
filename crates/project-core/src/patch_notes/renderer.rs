//! Rendering logic for patch notes in various formats

use crate::patch_notes::types::{Change, OutputFormat};
use crate::patch_notes::PatchNotes;

impl PatchNotes {
    /// Render patch notes to the specified format
    pub fn render(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Markdown => self.render_markdown(),
            OutputFormat::Html => self.render_html(),
            OutputFormat::PlainText => self.render_plain_text(),
            OutputFormat::Json => self.render_json(),
            OutputFormat::Bbcode => self.render_bbcode(),
        }
    }

    /// Render as Markdown
    fn render_markdown(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("# {}\n\n", self.title));
        output.push_str(&format!("**Author:** {}  \n", self.author));
        output.push_str(&format!("**Version:** {}  \n", self.version));
        output.push_str(&format!("**Date:** {}\n\n", self.date));

        // Summary
        output.push_str("## Summary\n\n");
        output.push_str(&format!(
            "This mod modifies **{}** boxer{} with:\n",
            self.summary.total_boxers_modified,
            if self.summary.total_boxers_modified == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "- {} palette change{}\n",
            self.summary.total_palettes_changed,
            if self.summary.total_palettes_changed == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "- {} sprite edit{}\n",
            self.summary.total_sprites_edited,
            if self.summary.total_sprites_edited == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "- {} animation adjustment{}\n",
            self.summary.total_animations_modified,
            if self.summary.total_animations_modified == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "- {} stat modification{}\n\n",
            self.summary.total_headers_edited,
            if self.summary.total_headers_edited == 1 { "" } else { "s" }
        ));

        // Changes by Boxer
        if !self.boxer_changes.is_empty() {
            output.push_str("## Changes by Boxer\n\n");

            for boxer in &self.boxer_changes {
                output.push_str(&format!("### {}\n\n", boxer.boxer_name));

                for change in &boxer.changes {
                    output.push_str(&format!("- {}\n", self.format_change_markdown(change)));
                }

                output.push('\n');
            }
        }

        // System changes
        if !self.system_changes.is_empty() {
            output.push_str("## System Changes\n\n");

            for change in &self.system_changes {
                output.push_str(&format!("- **{}**: {}\n", change.category, change.description));
            }

            output.push('\n');
        }

        // Installation
        output.push_str("## Installation\n\n");
        output.push_str("Apply the included `.ips` or `.bps` patch to a clean Super Punch-Out!! (USA) ROM.\n\n");
        output.push_str(&format!("SHA1 of target ROM: `{}`\n", self.base_rom_sha1));

        output
    }

    /// Format a single change as markdown
    fn format_change_markdown(&self, change: &Change) -> String {
        match change {
            Change::Palette { name, colors_changed, description } => {
                format!(
                    "**Palette** (`{}`): {} ({} color{} changed)",
                    name,
                    description,
                    colors_changed,
                    if *colors_changed == 1 { "" } else { "s" }
                )
            }
            Change::Sprite { bin_name, tiles_modified, description } => {
                format!(
                    "**Sprite** (`{}`): {} ({} tile{} modified)",
                    bin_name,
                    description,
                    tiles_modified,
                    if *tiles_modified == 1 { "" } else { "s" }
                )
            }
            Change::Stats { field, before, after, significant } => {
                let sig_marker = if *significant { " ⚠️" } else { "" };
                format!("**Stats** (`{}`): {} → {}{}", field, before, after, sig_marker)
            }
            Change::Animation { name, frames_changed, description } => {
                format!(
                    "**Animation** (`{}`): {} ({} frame{} changed)",
                    name,
                    description,
                    frames_changed,
                    if *frames_changed == 1 { "" } else { "s" }
                )
            }
            Change::Other { description } => {
                format!("**Other**: {}", description)
            }
        }
    }

    /// Render as HTML
    fn render_html(&self) -> String {
        let mut output = String::new();

        output.push_str("<!DOCTYPE html>\n");
        output.push_str("<html lang=\"en\">\n<head>\n");
        output.push_str(&format!("<title>{} - Patch Notes</title>\n", self.title));
        output.push_str("<style>\n");
        output.push_str(Self::default_html_css());
        output.push_str("</style>\n");
        output.push_str("</head>\n<body>\n");

        // Header
        output.push_str(&format!("<h1>{}</h1>\n", self.title));
        output.push_str("<div class=\"meta\">\n");
        output.push_str(&format!("<p><strong>Author:</strong> {}</p>\n", self.author));
        output.push_str(&format!("<p><strong>Version:</strong> {}</p>\n", self.version));
        output.push_str(&format!("<p><strong>Date:</strong> {}</p>\n", self.date));
        output.push_str("</div>\n");

        // Summary
        output.push_str("<h2>Summary</h2>\n");
        output.push_str(&format!(
            "<p>This mod modifies <strong>{}</strong> boxer{}.</p>\n",
            self.summary.total_boxers_modified,
            if self.summary.total_boxers_modified == 1 { "" } else { "s" }
        ));

        output.push_str("<ul>\n");
        output.push_str(&format!(
            "<li>{} palette change{}</li>\n",
            self.summary.total_palettes_changed,
            if self.summary.total_palettes_changed == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "<li>{} sprite edit{}</li>\n",
            self.summary.total_sprites_edited,
            if self.summary.total_sprites_edited == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "<li>{} animation adjustment{}</li>\n",
            self.summary.total_animations_modified,
            if self.summary.total_animations_modified == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "<li>{} stat modification{}</li>\n",
            self.summary.total_headers_edited,
            if self.summary.total_headers_edited == 1 { "" } else { "s" }
        ));
        output.push_str("</ul>\n");

        // Changes by Boxer
        if !self.boxer_changes.is_empty() {
            output.push_str("<h2>Changes by Boxer</h2>\n");

            for boxer in &self.boxer_changes {
                output.push_str(&format!("<h3>{}</h3>\n", boxer.boxer_name));
                output.push_str("<ul>\n");

                for change in &boxer.changes {
                    output.push_str(&format!("<li>{}</li>\n", self.format_change_html(change)));
                }

                output.push_str("</ul>\n");
            }
        }

        // Installation
        output.push_str("<h2>Installation</h2>\n");
        output.push_str("<p>Apply the included <code>.ips</code> or <code>.bps</code> patch to a clean Super Punch-Out!! (USA) ROM.</p>\n");
        output.push_str(&format!(
            "<p>SHA1 of target ROM: <code>{}</code></p>\n",
            self.base_rom_sha1
        ));

        output.push_str("</body>\n</html>");
        output
    }

    /// Default CSS for HTML output
    fn default_html_css() -> &'static str {
        r#"
body {
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
    line-height: 1.6;
    color: #333;
    background: #f5f5f5;
}
h1 { color: #2c3e50; border-bottom: 3px solid #3498db; padding-bottom: 0.5rem; }
h2 { color: #34495e; margin-top: 2rem; border-bottom: 1px solid #bdc3c7; padding-bottom: 0.3rem; }
h3 { color: #7f8c8d; margin-top: 1.5rem; }
.meta { background: white; padding: 1rem; border-radius: 8px; margin: 1rem 0; }
ul { background: white; padding: 1rem 1rem 1rem 2rem; border-radius: 8px; }
li { margin: 0.5rem 0; }
code { background: #ecf0f1; padding: 0.2rem 0.4rem; border-radius: 4px; font-family: monospace; }
strong { color: #2c3e50; }
"#
    }

    /// Format a single change as HTML
    fn format_change_html(&self, change: &Change) -> String {
        match change {
            Change::Palette { name, colors_changed, description } => {
                format!(
                    "<strong>Palette</strong> (<code>{}</code>): {} ({} color{} changed)",
                    name,
                    description,
                    colors_changed,
                    if *colors_changed == 1 { "" } else { "s" }
                )
            }
            Change::Sprite { bin_name, tiles_modified, description } => {
                format!(
                    "<strong>Sprite</strong> (<code>{}</code>): {} ({} tile{} modified)",
                    bin_name,
                    description,
                    tiles_modified,
                    if *tiles_modified == 1 { "" } else { "s" }
                )
            }
            Change::Stats { field, before, after, significant } => {
                let sig_marker = if *significant { " ⚠️" } else { "" };
                format!("<strong>Stats</strong> (<code>{}</code>): {} → {}{}", field, before, after, sig_marker)
            }
            Change::Animation { name, frames_changed, description } => {
                format!(
                    "<strong>Animation</strong> (<code>{}</code>): {} ({} frame{} changed)",
                    name,
                    description,
                    frames_changed,
                    if *frames_changed == 1 { "" } else { "s" }
                )
            }
            Change::Other { description } => {
                format!("<strong>Other</strong>: {}", description)
            }
        }
    }

    /// Render as plain text
    fn render_plain_text(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("{}\n", self.title));
        output.push_str(&"=".repeat(self.title.len()));
        output.push_str("\n\n");
        output.push_str(&format!("Author: {}\n", self.author));
        output.push_str(&format!("Version: {}\n", self.version));
        output.push_str(&format!("Date: {}\n\n", self.date));

        // Summary
        output.push_str("SUMMARY\n");
        output.push_str("-------\n");
        output.push_str(&format!(
            "This mod modifies {} boxer{} with:\n",
            self.summary.total_boxers_modified,
            if self.summary.total_boxers_modified == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "- {} palette change{}\n",
            self.summary.total_palettes_changed,
            if self.summary.total_palettes_changed == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "- {} sprite edit{}\n",
            self.summary.total_sprites_edited,
            if self.summary.total_sprites_edited == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "- {} animation adjustment{}\n",
            self.summary.total_animations_modified,
            if self.summary.total_animations_modified == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "- {} stat modification{}\n\n",
            self.summary.total_headers_edited,
            if self.summary.total_headers_edited == 1 { "" } else { "s" }
        ));

        // Changes by Boxer
        if !self.boxer_changes.is_empty() {
            output.push_str("CHANGES BY BOXER\n");
            output.push_str("----------------\n\n");

            for boxer in &self.boxer_changes {
                output.push_str(&format!("{}\n", boxer.boxer_name));
                output.push_str(&"-".repeat(boxer.boxer_name.len()));
                output.push('\n');

                for change in &boxer.changes {
                    output.push_str(&format!("* {}\n", self.format_change_text(change)));
                }

                output.push('\n');
            }
        }

        // Installation
        output.push_str("INSTALLATION\n");
        output.push_str("------------\n");
        output.push_str("Apply the included .ips or .bps patch to a clean Super Punch-Out!! (USA) ROM.\n\n");
        output.push_str(&format!("SHA1 of target ROM: {}\n", self.base_rom_sha1));

        output
    }

    /// Format a single change as plain text
    fn format_change_text(&self, change: &Change) -> String {
        match change {
            Change::Palette { name, colors_changed, description } => {
                format!(
                    "Palette [{}]: {} ({} color{} changed)",
                    name,
                    description,
                    colors_changed,
                    if *colors_changed == 1 { "" } else { "s" }
                )
            }
            Change::Sprite { bin_name, tiles_modified, description } => {
                format!(
                    "Sprite [{}]: {} ({} tile{} modified)",
                    bin_name,
                    description,
                    tiles_modified,
                    if *tiles_modified == 1 { "" } else { "s" }
                )
            }
            Change::Stats { field, before, after, significant } => {
                let sig_marker = if *significant { " [!]" } else { "" };
                format!("Stats [{}]: {} -> {}{}", field, before, after, sig_marker)
            }
            Change::Animation { name, frames_changed, description } => {
                format!(
                    "Animation [{}]: {} ({} frame{} changed)",
                    name,
                    description,
                    frames_changed,
                    if *frames_changed == 1 { "" } else { "s" }
                )
            }
            Change::Other { description } => {
                format!("Other: {}", description)
            }
        }
    }

    /// Render as JSON
    fn render_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Render as BBCode (for forums)
    fn render_bbcode(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str(&format!("[b][size=large]{}[/size][/b]\n\n", self.title));
        output.push_str(&format!("[b]Author:[/b] {}\n", self.author));
        output.push_str(&format!("[b]Version:[/b] {}\n", self.version));
        output.push_str(&format!("[b]Date:[/b] {}\n\n", self.date));

        // Summary
        output.push_str("[b][size=medium]Summary[/size][/b]\n\n");
        output.push_str(&format!(
            "This mod modifies [b]{}[/b] boxer{} with:\n",
            self.summary.total_boxers_modified,
            if self.summary.total_boxers_modified == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "[list]\n[*] {} palette change{}\n",
            self.summary.total_palettes_changed,
            if self.summary.total_palettes_changed == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "[*] {} sprite edit{}\n",
            self.summary.total_sprites_edited,
            if self.summary.total_sprites_edited == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "[*] {} animation adjustment{}\n",
            self.summary.total_animations_modified,
            if self.summary.total_animations_modified == 1 { "" } else { "s" }
        ));
        output.push_str(&format!(
            "[*] {} stat modification{}\n[/list]\n\n",
            self.summary.total_headers_edited,
            if self.summary.total_headers_edited == 1 { "" } else { "s" }
        ));

        // Changes by Boxer
        if !self.boxer_changes.is_empty() {
            output.push_str("[b][size=medium]Changes by Boxer[/size][/b]\n\n");

            for boxer in &self.boxer_changes {
                output.push_str(&format!("[b]{}[/b]\n\n", boxer.boxer_name));
                output.push_str("[list]\n");

                for change in &boxer.changes {
                    output.push_str(&format!("[*] {}\n", self.format_change_bbcode(change)));
                }

                output.push_str("[/list]\n\n");
            }
        }

        // Installation
        output.push_str("[b][size=medium]Installation[/size][/b]\n\n");
        output.push_str("Apply the included [code].ips[/code] or [code].bps[/code] patch to a clean Super Punch-Out!! (USA) ROM.\n\n");
        output.push_str(&format!(
            "SHA1 of target ROM: [code]{}[/code]\n",
            self.base_rom_sha1
        ));

        output
    }

    /// Format a single change as BBCode
    fn format_change_bbcode(&self, change: &Change) -> String {
        match change {
            Change::Palette { name, colors_changed, description } => {
                format!(
                    "[b]Palette[/b] ([code]{}[/code]): {} ({} color{} changed)",
                    name,
                    description,
                    colors_changed,
                    if *colors_changed == 1 { "" } else { "s" }
                )
            }
            Change::Sprite { bin_name, tiles_modified, description } => {
                format!(
                    "[b]Sprite[/b] ([code]{}[/code]): {} ({} tile{} modified)",
                    bin_name,
                    description,
                    tiles_modified,
                    if *tiles_modified == 1 { "" } else { "s" }
                )
            }
            Change::Stats { field, before, after, significant } => {
                let sig_marker = if *significant { " [!]" } else { "" };
                format!("[b]Stats[/b] ([code]{}[/code]): {} -> {}{}", field, before, after, sig_marker)
            }
            Change::Animation { name, frames_changed, description } => {
                format!(
                    "[b]Animation[/b] ([code]{}[/code]): {} ({} frame{} changed)",
                    name,
                    description,
                    frames_changed,
                    if *frames_changed == 1 { "" } else { "s" }
                )
            }
            Change::Other { description } => {
                format!("[b]Other[/b]: {}", description)
            }
        }
    }

    /// Format a boxer key into a readable name
    pub(crate) fn format_boxer_name(key: &str) -> String {
        key.split('_')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => {
                        first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                    }
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}
