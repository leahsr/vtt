# VTT

VTT is a Rust library for parsing and writing WebVTT (Web Video Text Tracks)
files. It helps you create, edit, and manage WebVTT cues, timestamps, and
settings. VTT works smoothly with [Serde](https://serde.rs/) for easy data
handling.

## Features

- **Parse WebVTT Files:** Turn WebVTT files into Rust data structures.
- **Write WebVTT Files:** Convert Rust data back to WebVTT format.
- **Manage Cues:** Add, change, and arrange WebVTT cues.
- **Handle Timestamps:** Work with precise timestamps for video tracks.
- **Use with Serde:** Easily serialize and deserialize data using Serde.

## Installation

Add VTT to your project's `Cargo.toml`:

```toml
[dependencies]
vtt = "0.1.0"
serde = { version = "1.0", features = ["derive"] }
```

## Usage

Import the prelude to access common types:

```rust
use vtt::prelude::*;
use std::time::Duration;

fn main() {
    let mut vtt = WebVtt::new();
    vtt.add_metadata("Language", "en-US");

    let cue = VttCue {
        identifier: Some("1".to_string()),
        start: VttTimestamp::new(Duration::from_secs(1)),
        end: VttTimestamp::new(Duration::from_secs(5)),
        settings: None,
        payload: "Hello, world!".to_string(),
    };
    vtt.add_cue(cue);

    println!("{}", vtt);
}
```

### Parsing a WebVTT File

Convert a WebVTT string into a `WebVtt` instance:

```rust
use vtt::prelude::*;
use std::str::FromStr;

fn parse_vtt(content: &str) -> Result<WebVtt, VttParseError> {
    WebVtt::from_str(content)
}

fn main() {
    let content = "WEBVTT\n\n00:01:02.000 --> 00:03:04.000\nHello, world!";
    let vtt = parse_vtt(content).unwrap();
    println!("Number of cues: {}", vtt.cues.len());
}
```

### Writing to WebVTT

Convert a `WebVtt` instance to a WebVTT string:

```rust
use vtt::prelude::*;
use std::time::Duration;

fn main() {
    let mut vtt = WebVtt::new();
    vtt.add_metadata("Language", "en-US");

    let cue = VttCue {
        identifier: Some("1".to_string()),
        start: VttTimestamp::new(Duration::from_secs(1)),
        end: VttTimestamp::new(Duration::from_secs(5)),
        settings: None,
        payload: "Hello, world!".to_string(),
    };
    vtt.add_cue(cue);

    let serialized = vtt.to_string();
    println!("{}", serialized);
}
```

## Documentation

Read the full documentation [here](https://docs.rs/vtt).

## Contributing

You can help make VTT better. Follow these steps:

1. Fork the repository.
2. Create a new branch.
3. Make your changes.
4. Submit a pull request.

Make sure your code follows the project's style and passes all tests.

## License

This project is licensed under the MIT or Apache-2.0 license.

## Contact

For questions or support, visit the
[repository](https://github.com/Govcraft/vtt).
