use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use tempfile::tempdir;

const MEDIA_EXTENSIONS: &[&str] = &[
    "mp4", "mov", "m4v", "mkv", "webm", "avi", "mp3", "wav", "m4a", "aac", "flac", "ogg",
];
const TRANSCRIPT_EXTENSIONS: &[&str] = &["txt"];
/// Upper bound for each transcript read by `merge_transcripts`.
const MAX_TRANSCRIPT_BYTES: u64 = 50 * 1024 * 1024;
const MAX_NAME_CHARS: usize = 120;

// ---------------------------------------------------------------------------
// Input validation. The paths below arrive from the web view, so they are
// checked here as well instead of trusting the frontend.
// ---------------------------------------------------------------------------

/// Resolves `path` to an existing regular file whose extension is in `allowed`.
fn validate_input_file(path: &str, allowed: &[&str]) -> Result<PathBuf, String> {
    let resolved = Path::new(path)
        .canonicalize()
        .map_err(|_| "The selected file does not exist.".to_string())?;

    if !resolved.is_file() {
        return Err("The selected path is not a regular file.".to_string());
    }

    let extension = resolved
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| "The selected file has no valid extension.".to_string())?;

    if !allowed.contains(&extension.as_str()) {
        return Err(format!(
            "Unsupported file type (.{extension}). Allowed: {}.",
            allowed.join(", ")
        ));
    }

    Ok(resolved)
}

/// Returns the output directory: the one chosen in the UI (absolute, created if
/// missing, must be a directory) or `fallback` when none was chosen.
fn resolve_output_dir(chosen: Option<&str>, fallback: &Path) -> Result<PathBuf, String> {
    let dir = match chosen {
        Some(dir) if !dir.trim().is_empty() => PathBuf::from(dir),
        _ => fallback.to_path_buf(),
    };

    if !dir.is_absolute() {
        return Err("The output folder must be an absolute path.".to_string());
    }
    if !dir.exists() {
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("Could not create output directory: {e}"))?;
    }

    let resolved = dir
        .canonicalize()
        .map_err(|e| format!("Could not open output directory: {e}"))?;
    if !resolved.is_dir() {
        return Err("The output path is not a folder.".to_string());
    }
    Ok(resolved)
}

/// Accepts a bare file name (no folders, no `..`, no control characters).
fn validate_file_name(name: &str) -> Result<String, String> {
    let name = name.trim();
    let invalid = name.is_empty()
        || name == "."
        || name == ".."
        || name.chars().count() > MAX_NAME_CHARS
        || name
            .chars()
            .any(|c| c == '/' || c == '\\' || c == ':' || c.is_control());

    if invalid {
        return Err("Invalid name for the combined file.".to_string());
    }
    Ok(name.to_string())
}

#[tauri::command(rename_all = "camelCase")]
async fn transcribe(video_path: String, output_dir: Option<String>) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        transcribe_file(&video_path, output_dir.as_deref())
    })
    .await
    .map_err(|error| error.to_string())?
}

fn transcribe_file(video_path: &str, output_dir: Option<&str>) -> Result<String, String> {
    let video = validate_input_file(video_path, MEDIA_EXTENSIONS)?;

    let source_dir = video
        .parent()
        .ok_or_else(|| "Could not determine the file directory.".to_string())?;
    let target_dir = resolve_output_dir(output_dir, source_dir)?;

    let stem = video
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Could not determine the file name.".to_string())?;

    let ffmpeg = find_ffmpeg()?;
    let whisper = find_whisper()?;
    let model = find_model()?;

    let temp_dir = tempdir().map_err(|_| "Could not create a temporary directory.".to_string())?;

    let temporary_audio = temp_dir.path().join("audio.wav");

    let output_txt = available_output_path(&target_dir, stem);
    let output_base = output_txt.with_extension("");

    let ffmpeg_output = Command::new(&ffmpeg)
        .arg("-y")
        .arg("-i")
        .arg(&video)
        .arg("-ar")
        .arg("16000")
        .arg("-ac")
        .arg("1")
        .arg(&temporary_audio)
        .output()
        .map_err(|error| format!("Could not start FFmpeg: {error}"))?;

    if !ffmpeg_output.status.success() {
        return Err(format_process_error("FFmpeg failed", &ffmpeg_output.stderr));
    }

    let whisper_output = Command::new(&whisper)
        .arg("-m")
        .arg(&model)
        .arg("-f")
        .arg(&temporary_audio)
        .arg("-l")
        .arg("auto")
        .arg("-otxt")
        .arg("-of")
        .arg(&output_base)
        .output()
        .map_err(|error| format!("Could not start whisper.cpp: {error}"))?;

    if !whisper_output.status.success() {
        return Err(format_process_error(
            "whisper.cpp failed",
            &whisper_output.stderr,
        ));
    }

    if !output_txt.exists() {
        return Err("Whisper finished, but the transcript file was not created.".to_string());
    }

    Ok(output_txt.to_string_lossy().into_owned())
}

#[tauri::command(rename_all = "camelCase")]
fn merge_transcripts(
    files: Vec<String>,
    output_dir: Option<String>,
    combined_name: Option<String>,
) -> Result<String, String> {
    if files.is_empty() {
        return Err("No transcript files to merge.".to_string());
    }

    let transcripts = files
        .iter()
        .map(|file| validate_input_file(file, TRANSCRIPT_EXTENSIONS))
        .collect::<Result<Vec<_>, _>>()?;

    for file in &transcripts {
        let size = std::fs::metadata(file)
            .map_err(|e| format!("Could not read transcript {}: {e}", file.display()))?
            .len();
        if size > MAX_TRANSCRIPT_BYTES {
            return Err(format!(
                "Transcript {} is too large to merge.",
                file.display()
            ));
        }
    }

    let source_dir = transcripts[0]
        .parent()
        .ok_or_else(|| "Could not determine the destination directory.".to_string())?;
    let target_dir = resolve_output_dir(output_dir.as_deref(), source_dir)?;

    let base_name = validate_file_name(
        combined_name
            .as_deref()
            .unwrap_or("Transcripciones_Combinadas"),
    )?;
    let output_txt = available_output_path(&target_dir, &base_name);

    let mut combined_content = String::new();

    for (index, file_path) in transcripts.iter().enumerate() {
        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Archivo");

        let content = std::fs::read_to_string(file_path)
            .map_err(|e| format!("Could not read transcript {}: {e}", file_path.display()))?;

        if index > 0 {
            combined_content
                .push_str("\n\n════════════════════════════════════════════════════════════\n\n");
        }
        combined_content.push_str(&format!("=== {} ===\n\n", file_stem));
        combined_content.push_str(content.trim());
        combined_content.push('\n');
    }

    std::fs::write(&output_txt, combined_content)
        .map_err(|e| format!("Could not save combined file: {e}"))?;

    Ok(output_txt.to_string_lossy().into_owned())
}

#[tauri::command]
fn show_in_folder(path: String) -> Result<(), String> {
    // Only transcripts produced by the app (.txt) can be revealed.
    let file = validate_input_file(&path, TRANSCRIPT_EXTENSIONS)
        .map_err(|_| "The transcript file does not exist.".to_string())?;

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg("-R")
            .arg(&file)
            .spawn()
            .map_err(|error| error.to_string())?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("explorer")
            .arg(format!("/select,{}", file.display()))
            .spawn()
            .map_err(|error| error.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        let parent = file
            .parent()
            .ok_or_else(|| "Could not locate the output folder.".to_string())?;

        Command::new("xdg-open")
            .arg(parent)
            .spawn()
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn available_output_path(parent: &std::path::Path, stem: &str) -> PathBuf {
    let first = parent.join(format!("{stem}_Transcription.txt"));

    if !first.exists() {
        return first;
    }

    for number in 2.. {
        let candidate = parent.join(format!("{stem}_Transcription_{number}.txt"));

        if !candidate.exists() {
            return candidate;
        }
    }

    unreachable!()
}

fn find_ffmpeg() -> Result<PathBuf, String> {
    find_executable(
        "ffmpeg",
        &["/opt/homebrew/bin/ffmpeg", "/usr/local/bin/ffmpeg"],
    )
    .ok_or_else(|| {
        "FFmpeg was not found. This development build currently requires FFmpeg to be installed."
            .to_string()
    })
}

fn find_whisper() -> Result<PathBuf, String> {
    find_executable(
        "whisper-cli",
        &[
            "/opt/homebrew/bin/whisper-cli",
            "/usr/local/bin/whisper-cli",
        ],
    )
    .ok_or_else(|| {
        "whisper-cli was not found. This development build currently requires whisper.cpp to be installed."
            .to_string()
    })
}

fn find_executable(command: &str, known_paths: &[&str]) -> Option<PathBuf> {
    for path in known_paths {
        let candidate = PathBuf::from(path);

        if candidate.exists() {
            return Some(candidate);
        }
    }

    let path_variable = env::var_os("PATH")?;

    for directory in env::split_paths(&path_variable) {
        let candidate = directory.join(command);

        if candidate.exists() {
            return Some(candidate);
        }

        #[cfg(target_os = "windows")]
        {
            let candidate = directory.join(format!("{command}.exe"));

            if candidate.exists() {
                return Some(candidate);
            }
        }
    }

    None
}

fn find_model() -> Result<PathBuf, String> {
    let home =
        home_directory().ok_or_else(|| "Could not determine the home directory.".to_string())?;

    let models_dir = home.join("whisper-models");

    // 1. Try ggml-medium.bin first
    let preferred = models_dir.join("ggml-medium.bin");
    if preferred.exists() {
        return Ok(preferred);
    }

    // 2. Try other standard model names
    let common_models = [
        "ggml-large-v3-turbo.bin",
        "ggml-large-v3.bin",
        "ggml-large.bin",
        "ggml-small.bin",
        "ggml-base.bin",
        "ggml-tiny.bin",
    ];
    for name in common_models {
        let candidate = models_dir.join(name);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    // 3. Search models_dir for any .bin file
    if let Ok(entries) = std::fs::read_dir(&models_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("bin") {
                return Ok(path);
            }
        }
    }

    // 4. Also check whisper-cpp Homebrew models location
    let homebrew_models = PathBuf::from("/opt/homebrew/share/whisper-cpp/models");
    if let Ok(entries) = std::fs::read_dir(&homebrew_models) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("bin") {
                return Ok(path);
            }
        }
    }

    Err(format!(
        "Whisper model was not found in {} or /opt/homebrew/share/whisper-cpp/models. Make sure to download a GGML model (e.g. ggml-medium.bin) into ~/whisper-models/.",
        models_dir.display()
    ))
}

fn home_directory() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        env::var_os("USERPROFILE").map(PathBuf::from)
    }

    #[cfg(not(target_os = "windows"))]
    {
        env::var_os("HOME").map(PathBuf::from)
    }
}

fn format_process_error(title: &str, stderr: &[u8]) -> String {
    let message = String::from_utf8_lossy(stderr);
    let message = message.trim();

    if message.is_empty() {
        title.to_string()
    } else {
        format!("{title}: {message}")
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            transcribe,
            show_in_folder,
            merge_transcripts
        ])
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(dir: &Path, name: &str, content: &str) -> String {
        let path = dir.join(name);
        std::fs::write(&path, content).unwrap();
        path.to_string_lossy().into_owned()
    }

    #[test]
    fn input_file_accepts_allowed_extension_case_insensitive() {
        let dir = tempdir().unwrap();
        let file = touch(dir.path(), "Clase.MP3", "");
        assert!(validate_input_file(&file, MEDIA_EXTENSIONS).is_ok());
    }

    #[test]
    fn input_file_rejects_missing_directory_and_wrong_extension() {
        let dir = tempdir().unwrap();
        let missing = dir.path().join("nada.mp3");
        assert!(validate_input_file(missing.to_str().unwrap(), MEDIA_EXTENSIONS).is_err());

        let folder = dir.path().join("carpeta.mp3");
        std::fs::create_dir(&folder).unwrap();
        assert!(validate_input_file(folder.to_str().unwrap(), MEDIA_EXTENSIONS).is_err());

        let script = touch(dir.path(), "script.sh", "echo hola");
        assert!(validate_input_file(&script, MEDIA_EXTENSIONS).is_err());
        assert!(validate_input_file(&script, TRANSCRIPT_EXTENSIONS).is_err());
    }

    #[test]
    fn input_file_resolves_dot_dot_segments() {
        let dir = tempdir().unwrap();
        std::fs::create_dir(dir.path().join("sub")).unwrap();
        touch(dir.path(), "a.txt", "x");
        let tricky = dir.path().join("sub/../a.txt");
        let resolved =
            validate_input_file(tricky.to_str().unwrap(), TRANSCRIPT_EXTENSIONS).unwrap();
        assert_eq!(resolved, dir.path().join("a.txt").canonicalize().unwrap());
    }

    #[test]
    fn output_dir_rules() {
        let dir = tempdir().unwrap();
        let fallback = dir.path().canonicalize().unwrap();

        assert_eq!(resolve_output_dir(None, &fallback).unwrap(), fallback);
        assert_eq!(resolve_output_dir(Some("  "), &fallback).unwrap(), fallback);
        assert!(resolve_output_dir(Some("relativa/carpeta"), &fallback).is_err());

        let file = touch(dir.path(), "no-soy-carpeta.txt", "");
        assert!(resolve_output_dir(Some(&file), &fallback).is_err());

        let new_dir = dir.path().join("nueva/sub");
        let created = resolve_output_dir(new_dir.to_str(), &fallback).unwrap();
        assert!(created.is_dir());
    }

    #[test]
    fn combined_name_must_be_a_bare_file_name() {
        assert_eq!(validate_file_name(" Mis apuntes ").unwrap(), "Mis apuntes");
        for bad in [
            "",
            "..",
            "../fuera",
            "/etc/passwd",
            "a/b",
            "a\\b",
            "C:x",
            "a\nb",
        ] {
            assert!(validate_file_name(bad).is_err(), "should reject {bad:?}");
        }
        assert!(validate_file_name(&"x".repeat(MAX_NAME_CHARS + 1)).is_err());
    }

    #[test]
    fn merge_combines_txt_files_and_rejects_others() {
        let dir = tempdir().unwrap();
        let a = touch(dir.path(), "uno_Transcription.txt", "hola");
        let b = touch(dir.path(), "dos_Transcription.txt", "mundo");

        let out = merge_transcripts(vec![a.clone(), b], None, Some("Juntas".into())).unwrap();
        let text = std::fs::read_to_string(&out).unwrap();
        assert!(text.contains("=== uno_Transcription ===") && text.contains("hola"));
        assert!(text.contains("=== dos_Transcription ===") && text.contains("mundo"));
        assert!(out.ends_with("Juntas_Transcription.txt"));

        let not_txt = touch(dir.path(), "secreto.key", "clave");
        assert!(merge_transcripts(vec![a.clone(), not_txt], None, None).is_err());
        assert!(merge_transcripts(vec![a.clone()], None, Some("../escape".into())).is_err());
        assert!(merge_transcripts(vec![], None, None).is_err());
    }

    #[test]
    fn merge_rejects_oversized_transcripts() {
        let dir = tempdir().unwrap();
        let big = dir.path().join("grande.txt");
        let file = std::fs::File::create(&big).unwrap();
        file.set_len(MAX_TRANSCRIPT_BYTES + 1).unwrap();
        let err =
            merge_transcripts(vec![big.to_string_lossy().into_owned()], None, None).unwrap_err();
        assert!(err.contains("too large"));
    }
}
