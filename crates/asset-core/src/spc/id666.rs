/// ID666 tag information (metadata).
///
/// ID666 is an optional metadata block that can be included in SPC files
/// to store information about the song, game, dumper, etc.
///
/// # Fields
/// - `song_title`: Song title (up to 32 bytes)
/// - `game_title`: Game name (up to 32 bytes)
/// - `dumper`: Name of person who dumped the SPC (up to 16 bytes)
/// - `comments`: Additional comments (up to 32 bytes)
/// - `dump_date`: Dump date in MMDDYYYY format (11 bytes)
/// - `seconds_to_play`: Suggested playback duration in seconds
/// - `fade_length_ms`: Fade out length in milliseconds
/// - `artist`: Song artist/composer (up to 32 bytes)
/// - `channel_disables`: Which channels to disable by default
/// - `emulator`: Emulator used to dump
#[derive(Debug, Clone, Default)]
pub struct Id666Tag {
    /// Song title (32 bytes max)
    pub song_title: String,
    /// Game name (32 bytes max)
    pub game_title: String,
    /// Dumper name (16 bytes max)
    pub dumper: String,
    /// Comments (32 bytes max)
    pub comments: String,
    /// Dump date (MMDDYYYY format, 11 bytes)
    pub dump_date: String,
    /// SPC duration in seconds
    pub seconds_to_play: u32,
    /// Fade length in milliseconds
    pub fade_length_ms: u32,
    /// Artist name (32 bytes max)
    pub artist: String,
    /// Default channel disables
    pub channel_disables: u8,
    /// Emulator used to dump
    pub emulator: u8,
}

impl Id666Tag {
    /// Creates a new empty ID666 tag.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the song title.
    pub fn with_song_title(mut self, title: impl Into<String>) -> Self {
        self.song_title = title.into();
        self
    }

    /// Sets the game title.
    pub fn with_game_title(mut self, title: impl Into<String>) -> Self {
        self.game_title = title.into();
        self
    }

    /// Sets the artist name.
    pub fn with_artist(mut self, artist: impl Into<String>) -> Self {
        self.artist = artist.into();
        self
    }

    /// Sets the dumper name.
    pub fn with_dumper(mut self, dumper: impl Into<String>) -> Self {
        self.dumper = dumper.into();
        self
    }

    /// Sets the dump date.
    pub fn with_dump_date(mut self, date: impl Into<String>) -> Self {
        self.dump_date = date.into();
        self
    }

    /// Sets the playback duration.
    pub fn with_duration(mut self, seconds: u32) -> Self {
        self.seconds_to_play = seconds;
        self
    }

    /// Sets the fade length.
    pub fn with_fade(mut self, ms: u32) -> Self {
        self.fade_length_ms = ms;
        self
    }
}
