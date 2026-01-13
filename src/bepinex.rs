use crate::types::ModIndex;
use anyhow::Result;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::PathBuf;
use zip::read::ZipArchive;

pub fn index_path(game_dir: &PathBuf) -> PathBuf {
    game_dir.join("BepInEx").join("mod-manager.index.json")
}

pub fn plugins_dir(game_dir: &PathBuf) -> PathBuf {
    game_dir.join("BepInEx").join("plugins")
}

pub fn bep_config_path(game_dir: &PathBuf) -> PathBuf {
    game_dir.join("BepInEx").join("config").join("BepInEx.cfg")
}

pub fn load_index(game_dir: &PathBuf) -> ModIndex {
    let path = index_path(game_dir);
    if let Ok(mut f) = File::open(&path) {
        let mut buf = String::new();
        if f.read_to_string(&mut buf).is_ok() {
            if let Ok(idx) = serde_json::from_str::<ModIndex>(&buf) {
                return idx;
            }
        }
    }
    ModIndex::default()
}

pub fn save_index(game_dir: &PathBuf, mods: &ModIndex) -> Result<()> {
    let path = index_path(game_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut f = File::create(path)?;
    let data = serde_json::to_string_pretty(&mods)?;
    f.write_all(data.as_bytes())?;
    Ok(())
}

pub fn is_bep_installed(game_dir: &PathBuf) -> bool {
    let bep_core_dll = game_dir.join("BepInEx").join("core").join("BepInEx.dll");
    bep_core_dll.exists()
}

pub fn detect_bep_status(game_dir: &PathBuf) -> String {
    let bep_core_dll = game_dir.join("BepInEx").join("core").join("BepInEx.dll");
    if bep_core_dll.exists() {
        "Installed".into()
    } else {
        "Not installed".into()
    }
}

pub fn validate_bepinex_installation(game_dir: &PathBuf) -> Result<()> {
    let bep_core_dll = game_dir.join("BepInEx").join("core").join("BepInEx.dll");
    let bep_core_xml = game_dir
        .join("BepInEx")
        .join("core")
        .join("BepInEx.Core.dll");
    let winhttp_dll = game_dir.join("winhttp.dll");

    if !bep_core_dll.exists() && !bep_core_xml.exists() {
        return Err(anyhow::anyhow!(
            "BepInEx/core/BepInEx.dll or BepInEx.Core.dll not found after extraction"
        ));
    }

    if !winhttp_dll.exists() {
        return Err(anyhow::anyhow!(
            "winhttp.dll not found in game directory after extraction"
        ));
    }

    Ok(())
}

pub fn ensure_dirs(game_dir: &PathBuf) -> Result<()> {
    fs::create_dir_all(plugins_dir(game_dir))?;
    Ok(())
}

pub fn set_unity_log_listening_false(game_dir: &PathBuf) -> Result<()> {
    let cfg_path = bep_config_path(game_dir);
    if !cfg_path.exists() {
        if let Some(parent) = cfg_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut f = File::create(&cfg_path)?;
        f.write_all(b"[Logging]\nUnityLogListening = false\n")?;
        return Ok(());
    }
    let mut content = String::new();
    File::open(&cfg_path)?.read_to_string(&mut content)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut in_logging = false;
    let mut updated = false;
    for i in 0..lines.len() {
        let l = lines[i].trim();
        if l.starts_with('[') && l.ends_with(']') {
            in_logging = l.eq_ignore_ascii_case("[logging]");
        } else if in_logging && l.to_lowercase().starts_with("unityloglistening") {
            lines[i] = "UnityLogListening = false".to_string();
            updated = true;
            break;
        }
    }
    if !updated {
        if !content.to_lowercase().contains("[logging]") {
            lines.push("".into());
            lines.push("[Logging]".into());
        }
        lines.push("UnityLogListening = false".into());
    }
    let mut f = File::create(&cfg_path)?;
    f.write_all(lines.join("\n").as_bytes())?;
    Ok(())
}

pub fn install_bepinex_from_zip_bytes(game_dir: &PathBuf, bytes: &[u8]) -> Result<()> {
    let reader = io::Cursor::new(bytes);
    let mut zip = ZipArchive::new(reader)?;

    for i in 0..zip.len() {
        let mut f = zip.by_index(i)?;
        let file_path = f.name();

        // Skip empty paths
        if file_path.is_empty() {
            continue;
        }

        let outpath = game_dir.join(file_path);

        if file_path.ends_with('/') {
            // Directory entry
            fs::create_dir_all(&outpath)?;
        } else {
            // File entry - create parent directories if needed
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut f, &mut outfile)?;
        }
    }

    let _ = set_unity_log_listening_false(game_dir);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ModEntry;
    use std::io::Cursor;
    use zip::ZipWriter;

    fn create_test_zip() -> Vec<u8> {
        let cursor = Cursor::new(Vec::new());
        let mut zw = ZipWriter::new(cursor);
        let options: zip::write::FileOptions<()> = zip::write::FileOptions::default();

        // Add key BepInEx files
        zw.start_file(".doorstop_version", options).unwrap();
        zw.write_all(b"1.0.0").unwrap();

        zw.start_file("doorstop_config.ini", options).unwrap();
        zw.write_all(b"[General]\nenabled=true\n").unwrap();

        zw.start_file("changelog.txt", options).unwrap();
        zw.write_all(b"v6.0.0-pre.2 Changelog\n").unwrap();

        zw.start_file("winhttp.dll", options).unwrap();
        zw.write_all(b"fake winhttp content").unwrap();

        // BepInEx directory structure
        zw.start_file("BepInEx/", options).unwrap();

        zw.start_file("BepInEx/core/", options).unwrap();

        zw.start_file("BepInEx/core/BepInEx.dll", options).unwrap();
        zw.write_all(b"fake dll content").unwrap();

        zw.start_file("BepInEx/core/BepInEx.Core.xml", options)
            .unwrap();
        zw.write_all(b"<xml></xml>").unwrap();

        zw.start_file("BepInEx/patchers/", options).unwrap();

        zw.start_file("BepInEx/plugins/", options).unwrap();

        zw.start_file("BepInEx/config/", options).unwrap();

        // dotnet directory
        zw.start_file("dotnet/", options).unwrap();

        zw.start_file("dotnet/.version", options).unwrap();
        zw.write_all(b"6.0.0").unwrap();

        let cursor = zw.finish().unwrap();
        cursor.into_inner()
    }

    #[test]
    fn test_bepinex_extraction() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let game_dir = PathBuf::from(temp_dir.path());

        let zip_bytes = create_test_zip();
        install_bepinex_from_zip_bytes(&game_dir, &zip_bytes).expect("Failed to extract BepInEx");

        // Check all expected files exist
        assert!(game_dir.join(".doorstop_version").exists());
        assert!(game_dir.join("doorstop_config.ini").exists());
        assert!(game_dir.join("changelog.txt").exists());
        assert!(game_dir.join("winhttp.dll").exists());

        // Check BepInEx structure
        assert!(game_dir.join("BepInEx").is_dir());
        assert!(game_dir.join("BepInEx/core").is_dir());
        assert!(game_dir.join("BepInEx/core/BepInEx.dll").exists());
        assert!(game_dir.join("BepInEx/core/BepInEx.Core.xml").exists());
        assert!(game_dir.join("BepInEx/patchers").is_dir());
        assert!(game_dir.join("BepInEx/plugins").is_dir());
        assert!(game_dir.join("BepInEx/config").is_dir());

        // Check dotnet directory
        assert!(game_dir.join("dotnet").is_dir());
        assert!(game_dir.join("dotnet/.version").exists());

        // Verify file contents
        let mut content = String::new();
        File::open(game_dir.join(".doorstop_version"))
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert_eq!(content, "1.0.0");

        // Check BepInEx installation detection
        assert!(is_bep_installed(&game_dir));
        assert_eq!(detect_bep_status(&game_dir), "Installed");
    }

    #[test]
    fn test_is_bep_not_installed() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let game_dir = PathBuf::from(temp_dir.path());

        assert!(!is_bep_installed(&game_dir));
        assert_eq!(detect_bep_status(&game_dir), "Not installed");
    }

    #[test]
    fn test_mod_index_persistence() {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
        let game_dir = PathBuf::from(temp_dir.path());

        let mut index = ModIndex::default();
        index.mods.push(ModEntry {
            id: "test_mod".to_string(),
            name: "Test Mod".to_string(),
            version: Some("1.0.0".to_string()),
            source_zip: None,
            installed_files: vec!["BepInEx/plugins/test.dll".to_string()],
        });

        save_index(&game_dir, &index).expect("Failed to save index");
        let loaded = load_index(&game_dir);

        assert_eq!(loaded.mods.len(), 1);
        assert_eq!(loaded.mods[0].id, "test_mod");
        assert_eq!(loaded.mods[0].name, "Test Mod");
    }
}
