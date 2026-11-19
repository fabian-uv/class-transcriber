use std::{env, path::PathBuf, process::Command};

use tempfile::tempdir;

#[tauri::command(rename_all = "camelCase")]
async fn transcribe(video_path: String, output_dir: Option<String>) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || transcribe_file(&video_path, output_dir.as_deref()))
        .await
        .map_err(|error| error.to_string())?
}

fn transcribe_file(video_path: &str, output_dir: Option<&str>) -> Result<String, String> {
    let video = PathBuf::from(video_path);

    if !video.exists() {
        return Err("The selected file does not exist.".to_string());
    }
    let extension = video
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| value.to_ascii_lowercase())
        .ok_or_else(|| "The selected file has no valid extension.".to_string())?;

    const ALLOWED_EXTENSIONS: &[&str] = &[
        "mp4", "mov", "m4v", "mkv", "webm", "avi", "mp3", "wav", "m4a", "aac", "flac", "ogg",
    ];

    if !ALLOWED_EXTENSIONS.contains(&extension.as_str()) {
        return Err("Unsupported audio or video format.".to_string());
    }

    let target_dir = match output_dir {
        Some(dir) if !dir.trim().is_empty() => PathBuf::from(dir),
        _ => video
            .parent()
            .ok_or_else(|| "Could not determine the file directory.".to_string())?
            .to_path_buf(),
    };

    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Could not create output directory: {e}"))?;
    }

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

    let first_file = PathBuf::from(&files[0]);
    let target_dir = match output_dir {
        Some(dir) if !dir.trim().is_empty() => PathBuf::from(dir),
        _ => first_file
            .parent()
            .ok_or_else(|| "Could not determine the destination directory.".to_string())?
            .to_path_buf(),
    };

    if !target_dir.exists() {
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Could not create output directory: {e}"))?;
    }

    let base_name = combined_name.unwrap_or_else(|| "Transcripciones_Combinadas".to_string());
    let output_txt = available_output_path(&target_dir, &base_name);

    let mut combined_content = String::new();

    for (index, file_path_str) in files.iter().enumerate() {
        let file_path = PathBuf::from(file_path_str);
        let file_stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Archivo");

        let content = std::fs::read_to_string(&file_path)
            .map_err(|e| format!("Could not read transcript {}: {e}", file_path.display()))?;

        if index > 0 {
            combined_content.push_str("\n\n════════════════════════════════════════════════════════════\n\n");
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
    let file = PathBuf::from(path);

    if !file.exists() {
        return Err("The transcript file does not exist.".to_string());
    }

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
