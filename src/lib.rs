//! # WebVTT Library
//!
//! This library provides functionality for parsing and serializing WebVTT (Web Video Text Tracks) files.
//! It allows creating, modifying, and managing WebVTT cues, timestamps, and settings.
//!
//! ## Usage
//!
//! To conveniently use the main types, include the prelude:
//!
//! ```rust
//! use std::time::Duration;
//! use vtt::prelude::*;
//!
//! // Example usage
//! let mut vtt = WebVtt::new();
//! vtt.add_metadata("Language", "en-US");
//!
//! let cue = VttCue {
//!     identifier: Some("1".to_string()),
//!     start: VttTimestamp::new(Duration::from_secs(1)),
//!     end: VttTimestamp::new(Duration::from_secs(5)),
//!     settings: None,
//!     payload: "Hello, world!".to_string(),
//! };
//! vtt.add_cue(cue);
//!
//! println!("{}", vtt);
//! ```

use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::str::FromStr;
use std::time::Duration;

/// An error type representing possible parsing failures in WebVTT data.
#[derive(Debug)]
pub enum VttParseError {
    /// The provided data does not conform to the expected format.
    InvalidFormat,
    /// The hours component of a timestamp is invalid.
    InvalidHours,
    /// The minutes component of a timestamp is invalid.
    InvalidMinutes,
    /// The seconds component of a timestamp is invalid.
    InvalidSeconds,
    /// The milliseconds component of a timestamp is invalid.
    InvalidMilliseconds,
    /// A setting within a cue is invalid.
    InvalidSetting(String),
    /// The WebVTT header is missing.
    MissingHeader,
    /// A metadata line is invalid.
    InvalidMetadataLine(String),
}

impl fmt::Display for VttParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VttParseError::InvalidFormat => write!(f, "Invalid format"),
            VttParseError::InvalidHours => write!(f, "Invalid hours format"),
            VttParseError::InvalidMinutes => write!(f, "Invalid minutes format"),
            VttParseError::InvalidSeconds => write!(f, "Invalid seconds format"),
            VttParseError::InvalidMilliseconds => write!(f, "Invalid milliseconds format"),
            VttParseError::InvalidSetting(s) => write!(f, "Invalid setting: {}", s),
            VttParseError::MissingHeader => write!(f, "Missing WEBVTT header"),
            VttParseError::InvalidMetadataLine(line) => {
                write!(f, "Invalid metadata line: {}", line)
            }
        }
    }
}

impl Error for VttParseError {}

/// Represents a timestamp in WebVTT format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VttTimestamp(Duration);

impl VttTimestamp {
    /// Creates a new `VttTimestamp` from a `Duration`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use vtt::VttTimestamp;
    ///
    /// let timestamp = VttTimestamp::new(Duration::from_secs(120));
    /// ```
    pub fn new(duration: Duration) -> Self {
        VttTimestamp(duration)
    }

    /// Returns the inner `Duration` of the timestamp.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use vtt::VttTimestamp;
    ///
    /// let timestamp = VttTimestamp::new(Duration::from_secs(60));
    /// assert_eq!(timestamp.as_duration(), Duration::from_secs(60));
    /// ```
    pub fn as_duration(&self) -> Duration {
        self.0
    }
}

impl FromStr for VttTimestamp {
    type Err = VttParseError;

    /// Parses a `VttTimestamp` from a string.
    ///
    /// The string should be in the format `HH:MM:SS.mmm` or `MM:SS.mmm`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vtt::VttTimestamp;
    /// use std::str::FromStr;
    ///
    /// let timestamp = VttTimestamp::from_str("01:23:45.678").unwrap();
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(':');

        let first = parts.next().ok_or(VttParseError::InvalidFormat)?;
        let second = parts.next().ok_or(VttParseError::InvalidFormat)?;
        let third = parts.next();

        match third {
            Some(third_part) => {
                // HH:MM:SS.mmm format
                let hours = first
                    .parse::<u64>()
                    .map_err(|_| VttParseError::InvalidHours)?;
                let minutes = second
                    .parse::<u64>()
                    .map_err(|_| VttParseError::InvalidMinutes)?;
                let (seconds, milliseconds) = parse_seconds_ms(third_part)?;

                let total_millis =
                    hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + milliseconds;
                Ok(VttTimestamp(Duration::from_millis(total_millis)))
            }
            None => {
                // MM:SS.mmm format
                let minutes = first
                    .parse::<u64>()
                    .map_err(|_| VttParseError::InvalidMinutes)?;
                let sec_str = second;
                let (seconds, milliseconds) = parse_seconds_ms(sec_str)?;
                let total_millis = minutes * 60_000 + seconds * 1_000 + milliseconds;
                Ok(VttTimestamp(Duration::from_millis(total_millis)))
            }
        }
    }
}

fn parse_seconds_ms(seconds_str: &str) -> Result<(u64, u64), VttParseError> {
    if let Some(dot_pos) = seconds_str.find('.') {
        let seconds = seconds_str[..dot_pos]
            .parse::<u64>()
            .map_err(|_| VttParseError::InvalidSeconds)?;
        let millis_str = &seconds_str[dot_pos + 1..];
        let millis = millis_str
            .parse::<u64>()
            .map_err(|_| VttParseError::InvalidMilliseconds)?;
        Ok((seconds, millis))
    } else {
        let seconds = seconds_str
            .parse::<u64>()
            .map_err(|_| VttParseError::InvalidSeconds)?;
        Ok((seconds, 0))
    }
}

impl fmt::Display for VttTimestamp {
    /// Formats the `VttTimestamp` as a string in `HH:MM:SS.mmm` format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use std::time::Duration;
    /// use vtt::VttTimestamp;
    ///
    /// let timestamp = VttTimestamp::new(Duration::from_millis(5025678));
    /// assert_eq!(timestamp.to_string(), "01:23:45.678");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total_millis = self.0.as_millis();
        let hours = total_millis / 3_600_000;
        let minutes = (total_millis % 3_600_000) / 60_000;
        let seconds = (total_millis % 60_000) / 1_000;
        let millis = total_millis % 1_000;

        write!(
            f,
            "{:02}:{:02}:{:02}.{:03}",
            hours, minutes, seconds, millis
        )
    }
}

impl Serialize for VttTimestamp {
    /// Serializes the `VttTimestamp` as a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde_json;
    /// use std::time::Duration;
    /// use vtt::VttTimestamp;
    ///
    /// let timestamp = VttTimestamp::new(Duration::from_secs(5));
    /// let serialized = serde_json::to_string(&timestamp).unwrap();
    /// assert_eq!(serialized, "\"00:00:05.000\"");
    /// ```
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for VttTimestamp {
    /// Deserializes a `VttTimestamp` from a string.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use serde_json;
    /// use vtt::VttTimestamp;
    ///
    /// let json = "\"00:01:02.000\"";
    /// let timestamp: VttTimestamp = serde_json::from_str(json).unwrap();
    /// assert_eq!(timestamp.to_string(), "00:01:02.000");
    /// ```
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        VttTimestamp::from_str(&s).map_err(serde::de::Error::custom)
    }
}

/// Represents a single cue in a WebVTT file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VttCue {
    /// An optional identifier for the cue.
    pub identifier: Option<String>,
    /// The start timestamp of the cue.
    pub start: VttTimestamp,
    /// The end timestamp of the cue.
    pub end: VttTimestamp,
    /// Optional settings for the cue.
    pub settings: Option<VttSettings>,
    /// The textual content of the cue.
    pub payload: String,
}

impl FromStr for VttCue {
    type Err = VttParseError;

    /// Parses a `VttCue` from a string.
    ///
    /// The string should follow the WebVTT cue format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vtt::{VttCue, VttTimestamp};
    /// use std::str::FromStr;
    /// use std::time::Duration;
    ///
    /// let cue_str = "00:01:02.000 --> 00:03:04.000\nHello, world!";
    /// let cue = VttCue::from_str(cue_str).unwrap();
    ///
    /// assert_eq!(cue.start.as_duration(), Duration::from_secs(62));
    /// assert_eq!(cue.end.as_duration(), Duration::from_secs(184));
    /// assert_eq!(cue.payload, "Hello, world!");
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let first_line = lines.next().ok_or(VttParseError::InvalidFormat)?;

        let identifier = if !first_line.contains("-->") {
            Some(first_line.to_string())
        } else {
            None
        };

        let timing_line = if identifier.is_some() {
            lines.next().ok_or(VttParseError::InvalidFormat)?
        } else {
            first_line
        };

        let timing_parts: Vec<&str> = timing_line.split("-->").collect();
        if timing_parts.len() != 2 {
            return Err(VttParseError::InvalidFormat);
        }

        let start = VttTimestamp::from_str(timing_parts[0].trim())?;
        let end_and_settings = timing_parts[1].trim();

        let mut end_part_and_settings = end_and_settings.split_whitespace();
        let end_part = end_part_and_settings
            .next()
            .ok_or(VttParseError::InvalidFormat)?;
        let end = VttTimestamp::from_str(end_part)?;

        // Build settings string
        let settings_str = end_part_and_settings.collect::<Vec<&str>>().join(" ");
        let settings = if !settings_str.is_empty() {
            Some(parse_settings(&settings_str)?)
        } else {
            None
        };

        // Collect remaining lines as payload
        let payload = lines.collect::<Vec<&str>>().join("\n");

        Ok(VttCue {
            identifier,
            start,
            end,
            settings,
            payload,
        })
    }
}

fn parse_settings(settings_str: &str) -> Result<VttSettings, VttParseError> {
    let mut settings = VttSettings::default();

    for setting in settings_str.split_whitespace() {
        if let Some(idx) = setting.find(':') {
            let key = &setting[..idx];
            let value = &setting[idx + 1..];

            match key {
                "vertical" => {
                    settings.vertical = match value {
                        "rl" => Some(VerticalSetting::RightToLeft),
                        "lr" => Some(VerticalSetting::LeftToRight),
                        _ => None,
                    };
                }
                "line" => {
                    settings.line = match value {
                        "auto" => Some(LineSetting::Auto),
                        val => {
                            if let Some(stripped) = val.strip_suffix('%') {
                                let percent: u32 = stripped.parse().map_err(|_| {
                                    VttParseError::InvalidSetting("line percentage".into())
                                })?;
                                Some(LineSetting::Percentage(percent))
                            } else {
                                let number: i32 = val.parse().map_err(|_| {
                                    VttParseError::InvalidSetting("line number".into())
                                })?;
                                Some(LineSetting::Number(number))
                            }
                        }
                    };
                }
                "position" => {
                    if let Some(stripped) = value.strip_suffix('%') {
                        let pos: u32 = stripped
                            .parse()
                            .map_err(|_| VttParseError::InvalidSetting("position".into()))?;
                        settings.position = Some(pos);
                    }
                }
                "size" => {
                    if let Some(stripped) = value.strip_suffix('%') {
                        let size: u32 = stripped
                            .parse()
                            .map_err(|_| VttParseError::InvalidSetting("size".into()))?;
                        settings.size = Some(size);
                    }
                }
                "align" => {
                    settings.align = match value {
                        "start" => Some(AlignSetting::Start),
                        "middle" => Some(AlignSetting::Middle),
                        "end" => Some(AlignSetting::End),
                        "left" => Some(AlignSetting::Left),
                        "right" => Some(AlignSetting::Right),
                        _ => None,
                    };
                }
                _ => {}
            }
        }
    }

    Ok(settings)
}

impl fmt::Display for VttCue {
    /// Formats the `VttCue` as a string following the WebVTT cue format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vtt::{VttCue, VttTimestamp};
    /// use std::time::Duration;
    ///
    /// let cue = VttCue {
    ///     identifier: None,
    ///     start: VttTimestamp::new(Duration::from_secs(1)),
    ///     end: VttTimestamp::new(Duration::from_secs(5)),
    ///     settings: None,
    ///     payload: "Test".to_string(),
    /// };
    ///
    /// assert_eq!(cue.to_string(), "00:00:01.000 --> 00:00:05.000\nTest");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write identifier if present
        if let Some(ref id) = self.identifier {
            writeln!(f, "{}", id)?;
        }

        // Write timing line with proper spacing
        write!(f, "{} --> {}", self.start, self.end)?;

        // Write settings if present
        if let Some(ref settings) = self.settings {
            let settings_str = settings.to_string();
            if !settings_str.is_empty() {
                write!(f, " {}", settings_str)?;
            }
        }

        // Write newline and payload
        write!(f, "\n{}", self.payload.trim())
    }
}

/// Represents the settings associated with a WebVTT cue.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VttSettings {
    /// The vertical setting of the cue.
    pub vertical: Option<VerticalSetting>,
    /// The line position setting of the cue.
    pub line: Option<LineSetting>,
    /// The position percentage of the cue.
    pub position: Option<u32>,
    /// The size percentage of the cue.
    pub size: Option<u32>,
    /// The alignment setting of the cue.
    pub align: Option<AlignSetting>,
}

impl fmt::Display for VttSettings {
    /// Formats the `VttSettings` as a string suitable for WebVTT cues.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vtt::{VttSettings, VerticalSetting, LineSetting, AlignSetting};
    ///
    /// let settings = VttSettings {
    ///     vertical: Some(VerticalSetting::LeftToRight),
    ///     line: Some(LineSetting::Percentage(90)),
    ///     position: Some(50),
    ///     size: Some(40),
    ///     align: Some(AlignSetting::Middle),
    /// };
    ///
    /// assert_eq!(settings.to_string(), "vertical:lr line:90% position:50% size:40% align:middle");
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut settings = Vec::with_capacity(5);

        if let Some(ref vertical) = self.vertical {
            settings.push(match vertical {
                VerticalSetting::RightToLeft => "vertical:rl".to_string(),
                VerticalSetting::LeftToRight => "vertical:lr".to_string(),
            });
        }

        if let Some(ref line) = self.line {
            settings.push(match line {
                LineSetting::Percentage(n) => format!("line:{}%", n),
                LineSetting::Number(n) => n.to_string(),
                LineSetting::Auto => "auto".to_string(),
            });
        }

        if let Some(position) = self.position {
            settings.push(format!("position:{}%", position));
        }

        if let Some(size) = self.size {
            settings.push(format!("size:{}%", size));
        }

        if let Some(ref align) = self.align {
            settings.push(match align {
                AlignSetting::Start => "align:start".to_string(),
                AlignSetting::Middle => "align:middle".to_string(),
                AlignSetting::End => "align:end".to_string(),
                AlignSetting::Left => "align:left".to_string(),
                AlignSetting::Right => "align:right".to_string(),
            });
        }

        write!(f, "{}", settings.join(" "))
    }
}

/// Specifies the vertical orientation of a cue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VerticalSetting {
    /// Right-to-left vertical orientation.
    #[serde(rename = "rl")]
    RightToLeft,
    /// Left-to-right vertical orientation.
    #[serde(rename = "lr")]
    LeftToRight,
}

/// Specifies the line position of a cue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LineSetting {
    /// Line position as a percentage.
    Percentage(u32),
    /// Line position as a number.
    Number(i32),
    /// Automatic line positioning.
    Auto,
}

/// Specifies the alignment of a cue.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlignSetting {
    /// Start alignment.
    Start,
    /// Middle alignment.
    Middle,
    /// End alignment.
    End,
    /// Left alignment.
    Left,
    /// Right alignment.
    Right,
}

/// Represents a complete WebVTT file, including its header and cues.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WebVtt {
    /// The header of the WebVTT file.
    pub header: VttHeader,
    /// The collection of cues within the WebVTT file.
    pub cues: Vec<VttCue>,
}

impl WebVtt {
    /// Creates a new, empty `WebVtt` instance.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vtt::WebVtt;
    ///
    /// let vtt = WebVtt::new();
    /// ```
    pub fn new() -> Self {
        Self {
            header: VttHeader::default(),
            cues: Vec::new(),
        }
    }

    /// Adds a cue to the WebVTT file.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vtt::{WebVtt, VttCue, VttTimestamp};
    /// use std::time::Duration;
    ///
    /// let mut vtt = WebVtt::new();
    /// let cue = VttCue {
    ///     identifier: None,
    ///     start: VttTimestamp::new(Duration::from_secs(10)),
    ///     end: VttTimestamp::new(Duration::from_secs(20)),
    ///     settings: None,
    ///     payload: "Sample subtitle".to_string(),
    /// };
    /// vtt.add_cue(cue);
    /// ```
    pub fn add_cue(&mut self, cue: VttCue) {
        self.cues.push(cue);
    }

    /// Adds a metadata entry to the WebVTT header.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vtt::WebVtt;
    ///
    /// let mut vtt = WebVtt::new();
    /// vtt.add_metadata("Language", "en-US");
    /// ```
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.header
            .metadata
            .insert(key.to_string(), value.to_string());
    }
}

/// Represents the header section of a WebVTT file.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VttHeader {
    /// An optional description of the WebVTT content.
    pub description: Option<String>,
    /// A collection of metadata key-value pairs.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl FromStr for WebVtt {
    type Err = VttParseError;

    /// Parses a `WebVtt` instance from a string.
    ///
    /// The string should follow the WebVTT file format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vtt::WebVtt;
    /// use std::str::FromStr;
    ///
    /// let content = "WEBVTT\n\n00:01:02.000 --> 00:03:04.000\nHello, world!";
    /// let vtt = WebVtt::from_str(content).unwrap();
    /// assert_eq!(vtt.cues.len(), 1);
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let first_line = lines.next().ok_or(VttParseError::InvalidFormat)?.trim();

        // Check for WEBVTT header
        if !first_line.starts_with("WEBVTT") {
            return Err(VttParseError::MissingHeader);
        }

        let mut header = VttHeader::default();

        // Parse description if present (everything after WEBVTT on the first line)
        if first_line.len() > 6 {
            header.description = Some(first_line[6..].trim().to_string());
        }

        // Parse metadata (key: value pairs before the first empty line)
        for line in &mut lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }

            if let Some((key, value)) = trimmed.split_once(':') {
                header
                    .metadata
                    .insert(key.trim().to_string(), value.trim().to_string());
            } else {
                return Err(VttParseError::InvalidMetadataLine(trimmed.to_string()));
            }
        }

        // Parse cues
        let mut cues = Vec::new();
        let mut cue_lines = Vec::new();

        for line in lines {
            if line.trim().is_empty() {
                if !cue_lines.is_empty() {
                    cues.push(VttCue::from_str(&cue_lines.join("\n"))?);
                    cue_lines.clear();
                }
            } else {
                cue_lines.push(line);
            }
        }

        // Don't forget the last cue if file doesn't end with empty line
        if !cue_lines.is_empty() {
            cues.push(VttCue::from_str(&cue_lines.join("\n"))?);
        }

        Ok(WebVtt { header, cues })
    }
}

impl fmt::Display for WebVtt {
    /// Formats the `WebVtt` instance as a string following the WebVTT file format.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use vtt::{WebVtt, VttCue, VttTimestamp};
    /// use std::time::Duration;
    ///
    /// let mut vtt = WebVtt::new();
    /// vtt.header.description = Some("Sample File".to_string());
    /// vtt.add_metadata("Language", "en-US");
    ///
    /// let cue = VttCue {
    ///     identifier: Some("1".to_string()),
    ///     start: VttTimestamp::new(Duration::from_secs(1)),
    ///     end: VttTimestamp::new(Duration::from_secs(5)),
    ///     settings: None,
    ///     payload: "Hello, world!".to_string(),
    /// };
    /// vtt.add_cue(cue);
    ///
    /// let expected = "WEBVTT Sample File\nLanguage: en-US\n\n1\n00:00:01.000 --> 00:00:05.000\nHello, world!";
    /// assert_eq!(vtt.to_string(), expected);
    /// ```
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Write WEBVTT header
        if let Some(ref description) = self.header.description {
            writeln!(f, "WEBVTT {}", description)?;
        } else {
            writeln!(f, "WEBVTT")?;
        }

        // Write metadata
        for (key, value) in &self.header.metadata {
            writeln!(f, "{}: {}", key, value)?;
        }

        // Empty line after header section
        writeln!(f)?;

        // Write cues
        for (i, cue) in self.cues.iter().enumerate() {
            if i > 0 {
                writeln!(f)?; // Empty line between cues
            }
            write!(f, "{}", cue)?;
        }

        Ok(())
    }
}

/// A module that provides a prelude for the WebVTT library.
///
/// The prelude includes commonly used types, allowing for easier imports.
pub mod prelude {
    pub use super::{
        AlignSetting, LineSetting, VerticalSetting, VttCue, VttHeader, VttParseError, VttSettings,
        VttTimestamp, WebVtt,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timestamp() {
        let timestamp = VttTimestamp::from_str("01:23:45.678").unwrap();
        assert_eq!(timestamp.as_duration(), Duration::from_millis(5025678));

        let timestamp = VttTimestamp::from_str("23:45.678").unwrap();
        assert_eq!(timestamp.as_duration(), Duration::from_millis(1425678));
    }

    #[test]
    fn test_timestamp_display() {
        let timestamp = VttTimestamp::new(Duration::from_millis(5025678));
        assert_eq!(timestamp.to_string(), "01:23:45.678");
    }

    #[test]
    fn test_parse_simple_cue() {
        let cue_str = "00:01:02.000 --> 00:03:04.000\nHello, world!";
        let cue = VttCue::from_str(cue_str).unwrap();

        assert_eq!(cue.start.as_duration(), Duration::from_secs(62));
        assert_eq!(cue.end.as_duration(), Duration::from_secs(184));
        assert_eq!(cue.payload, "Hello, world!");
    }

    #[test]
    fn test_parse_cue_with_settings() {
        let cue_str =
            "00:00:00.000 --> 00:00:05.000 line:90% position:50% align:middle\nSubtitle text";
        let cue = VttCue::from_str(cue_str).unwrap();

        assert!(cue.settings.is_some());
        let settings = cue.settings.unwrap();
        assert_eq!(settings.line, Some(LineSetting::Percentage(90)));
        assert_eq!(settings.position, Some(50));
        assert_eq!(settings.align, Some(AlignSetting::Middle));
    }

    #[test]
    fn test_parse_cue_with_identifier() {
        let cue_str = "id1\n00:00:00.000 --> 00:00:05.000\nSubtitle text";
        let cue = VttCue::from_str(cue_str).unwrap();

        assert_eq!(cue.identifier, Some("id1".to_string()));
        assert_eq!(cue.payload, "Subtitle text");
    }

    #[test]
    fn test_display_format() {
        let cue = VttCue {
            identifier: None,
            start: VttTimestamp::new(Duration::from_secs(1)),
            end: VttTimestamp::new(Duration::from_secs(5)),
            settings: None,
            payload: "Test".to_string(),
        };

        let expected = "00:00:01.000 --> 00:00:05.000\nTest";
        assert_eq!(cue.to_string(), expected);
    }
    #[test]
    fn test_parse_basic_vtt() {
        let content = r#"WEBVTT

00:01:02.000 --> 00:03:04.000
Hello, world!

00:03:05.000 --> 00:03:08.000
Second subtitle"#;

        let vtt = WebVtt::from_str(content).unwrap();
        assert_eq!(vtt.cues.len(), 2);
        assert_eq!(vtt.cues[0].payload, "Hello, world!");
        assert_eq!(vtt.cues[1].payload, "Second subtitle");
    }

    #[test]
    fn test_parse_vtt_with_metadata() {
        let content = r#"WEBVTT Sample File
Region: id=region1 width=40%
Style: color:red

00:01:02.000 --> 00:03:04.000
First subtitle"#;

        let vtt = WebVtt::from_str(content).unwrap();
        assert_eq!(vtt.header.description, Some("Sample File".to_string()));
        assert_eq!(
            vtt.header.metadata.get("Region").unwrap(),
            "id=region1 width=40%"
        );
        assert_eq!(vtt.header.metadata.get("Style").unwrap(), "color:red");
        assert_eq!(vtt.cues.len(), 1);
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut vtt = WebVtt::new();
        vtt.header.description = Some("Test File".to_string());
        vtt.add_metadata("Language", "en-US");

        let cue = VttCue {
            identifier: Some("1".to_string()),
            start: VttTimestamp::new(Duration::from_secs(1)),
            end: VttTimestamp::new(Duration::from_secs(5)),
            settings: None,
            payload: "Test subtitle".to_string(),
        };
        vtt.add_cue(cue);

        let serialized = vtt.to_string();
        let deserialized = WebVtt::from_str(&serialized).unwrap();

        assert_eq!(deserialized.header.description, vtt.header.description);
        assert_eq!(deserialized.header.metadata, vtt.header.metadata);
        assert_eq!(deserialized.cues.len(), vtt.cues.len());
        assert_eq!(deserialized.cues[0].payload, "Test subtitle");
    }

    #[test]
    fn test_write_to_string() {
        let mut vtt = WebVtt::new();
        vtt.header.description = Some("Test".to_string());
        vtt.add_metadata("Language", "en");

        let cue = VttCue {
            identifier: None,
            start: VttTimestamp::new(Duration::from_secs(1)),
            end: VttTimestamp::new(Duration::from_secs(5)),
            settings: None,
            payload: "Test".to_string(),
        };
        vtt.add_cue(cue);

        let expected = r#"WEBVTT Test
Language: en

00:00:01.000 --> 00:00:05.000
Test"#;

        assert_eq!(vtt.to_string(), expected);
    }
}
