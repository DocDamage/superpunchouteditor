use crate::audio::Spc700Data;
use crate::spc::{Id666Tag, SpcError, SpcFile};
use std::path::Path;

/// Builder for creating SPC files from scratch.
///
/// Provides a fluent API for constructing SPC files with metadata.
///
/// # Example
/// ```
/// use asset_core::spc::SpcBuilder;
///
/// let builder = SpcBuilder::new()
///     .with_song_title("Victory Theme")
///     .with_game_title("Super Punch-Out!!")
///     .with_artist("Nintendo Sound Team");
/// ```
pub struct SpcBuilder {
    pub(crate) data: Spc700Data,
    pub(crate) tag: Option<Id666Tag>,
}

impl SpcBuilder {
    /// Creates a new SPC builder.
    pub fn new() -> Self {
        Self {
            data: Spc700Data::default(),
            tag: None,
        }
    }

    /// Sets SPC700 data.
    pub fn with_data(mut self, data: Spc700Data) -> Self {
        self.data = data;
        self
    }

    /// Sets ID666 tag.
    pub fn with_tag(mut self, tag: Id666Tag) -> Self {
        self.tag = Some(tag);
        self
    }

    /// Sets song title.
    pub fn with_song_title(mut self, title: &str) -> Self {
        let tag = self.tag.get_or_insert_with(Id666Tag::default);
        tag.song_title = title.to_string();
        self
    }

    /// Sets game title.
    pub fn with_game_title(mut self, title: &str) -> Self {
        let tag = self.tag.get_or_insert_with(Id666Tag::default);
        tag.game_title = title.to_string();
        self
    }

    /// Sets artist.
    pub fn with_artist(mut self, artist: &str) -> Self {
        let tag = self.tag.get_or_insert_with(Id666Tag::default);
        tag.artist = artist.to_string();
        self
    }

    /// Sets dumper name.
    pub fn with_dumper(mut self, dumper: &str) -> Self {
        let tag = self.tag.get_or_insert_with(Id666Tag::default);
        tag.dumper = dumper.to_string();
        self
    }

    /// Sets playback duration.
    pub fn with_duration(mut self, seconds: u32) -> Self {
        let tag = self.tag.get_or_insert_with(Id666Tag::default);
        tag.seconds_to_play = seconds;
        self
    }

    /// Builds and saves to file.
    ///
    /// # Arguments
    /// - `path`: Output file path
    ///
    /// # Returns
    /// `Ok(())` on success, `Err(SpcError)` on failure
    pub fn save<P: AsRef<Path>>(self, path: P) -> Result<(), SpcError> {
        SpcFile::save(&self.data, path, self.tag.as_ref())
    }
}

impl Default for SpcBuilder {
    fn default() -> Self {
        Self::new()
    }
}
