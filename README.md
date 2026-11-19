# Class Transcriber

![License](https://img.shields.io/badge/license-MIT-00b981)
![Tauri](https://img.shields.io/badge/Tauri-2.x-00b981)
![whisper.cpp](https://img.shields.io/badge/whisper.cpp-local-00b981)
![Status](https://img.shields.io/badge/status-active%20development-00b981)

A lightweight desktop application for transcribing audio and video files locally using [whisper.cpp](https://github.com/ggml-org/whisper.cpp).

All media processing is performed offline on your device. No cloud services, accounts, or subscriptions are required.

![Class Transcriber](docs/screenshot.png)

## Overview

Class Transcriber provides an efficient, privacy-first interface to transcribe recordings, lectures, and media files without sending data to third-party APIs. It wraps `whisper.cpp` and `ffmpeg` into a native desktop workflow built with Tauri.

## Features

- **Batch Processing Queue**: Select multiple media files at once or drop an entire batch. Files are queued and processed sequentially with individual progress tracking.
- **Transcript Merging**: Combine all processed transcriptions into a single consolidated text file with standardized delimiters and section titles, available automatically on completion or on demand.
- **Custom Output Directory**: Save generated transcripts directly alongside source media or define a dedicated output destination folder.
- **Local and Private**: All audio extraction and speech recognition run entirely on local hardware. Media files never leave your system.
- **Flexible Model Discovery**: Automatically locates GGML models in `~/whisper-models/` (supporting `ggml-medium.bin`, large, small, base, tiny, or custom model files) and standard system paths.
- **Bilingual Interface**: Native support for English and Spanish.
- **Broad Media Format Support**: Handles common video and audio containers without manual pre-conversion.

## Supported Formats

### Video
- MP4, MOV, M4V, MKV, WebM, AVI

### Audio
- MP3, WAV, M4A, AAC, FLAC, OGG

## Requirements

The current macOS build relies on local system binaries:

1. **FFmpeg** (audio extraction and sampling):
   ```bash
   brew install ffmpeg
   ```
2. **whisper.cpp** CLI (`whisper-cli`):
   ```bash
   brew install whisper-cpp
   ```
3. **Whisper GGML Model**:
   Place compatible GGML model files in `~/whisper-models/` (for example, `ggml-medium.bin` or `ggml-large-v3-turbo.bin`):
   ```bash
   mkdir -p ~/whisper-models
   curl -L -o ~/whisper-models/ggml-medium.bin https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin
   ```

## Installation on macOS

### From Source

Clone the repository and run the installation script:

```bash
git clone https://github.com/fabianubv/class-transcriber.git
cd class-transcriber
./install.sh
```

This compiles the release binary and installs `Class Transcriber.app` into `/Applications`.

### Building a Disk Image (DMG)

To package a standalone `.dmg` installer:

```bash
./build-dmg.sh
```

The resulting `Class-Transcriber.dmg` will be created in the project root directory.

## Development

Prerequisites:
- Node.js (v18+)
- Rust and Cargo
- Xcode Command Line Tools (on macOS)

```bash
# Clone the repository
git clone https://github.com/fabianubv/class-transcriber.git
cd class-transcriber

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

To build a standalone release without installing:

```bash
npx tauri build
```

## Privacy Policy

Class Transcriber processes all media exclusively on your local machine.

- No telemetry or analytics collection.
- No network requests during transcription.
- No transmission of user audio, video, transcripts, or directory metadata.

## Platform Support

- macOS: Fully supported and tested (Apple Silicon & Intel).
- Windows & Linux: Planned.

## License

MIT License. See [LICENSE](LICENSE) for details.
