// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Jareer and Concat contributors

//! User media that belongs to the app and can be reused by any project.

use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::AppDirs;

const INDEX: &str = "library.json";
static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// One reusable file and the words that tell an editor when it belongs.
#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub struct Asset {
    /// Stable id, also used as the stored file's stem.
    pub id: String,
    /// The name shown in the app.
    pub name: String,
    /// The user's guidance for when the file should be used.
    pub description: String,
    /// The name of the selected source file, for display and provenance.
    pub source_name: String,
    /// File name relative to the app's `assets` directory.
    pub file: String,
}

/// The app-managed directory containing the reusable media and its index.
pub fn folder(dirs: &AppDirs) -> PathBuf {
    dirs.data.join("assets")
}

/// The machine's saved assets, in the order they were added.
pub fn list(dirs: &AppDirs) -> Result<Vec<Asset>, String> {
    let index = folder(dirs).join(INDEX);
    let contents = match std::fs::read(index) {
        Ok(contents) => contents,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("could not read the asset library: {error}")),
    };
    let library: AssetIndex = serde_json::from_slice(&contents)
        .map_err(|error| format!("could not read the asset library: {error}"))?;
    if library.version != 1
        || library
            .assets
            .iter()
            .any(|asset| !safe_file_name(&asset.file))
    {
        return Err("the asset library index is invalid".to_owned());
    }
    Ok(library.assets)
}

/// Copy one file into the app's library and save its name and description.
pub fn add(dirs: &AppDirs, source: &Path, name: &str, description: &str) -> Result<Asset, String> {
    let name = name.trim();
    let description = description.trim();
    if name.is_empty() || description.is_empty() {
        return Err("Enter both a name and a description for this asset.".to_owned());
    }
    if !source.is_file() {
        return Err("The selected asset file could not be found.".to_owned());
    }

    let folder = folder(dirs);
    std::fs::create_dir_all(&folder)
        .map_err(|error| format!("could not create the asset library: {error}"))?;
    let mut assets = list(dirs)?;
    if assets
        .iter()
        .any(|asset| asset.name.eq_ignore_ascii_case(name))
    {
        return Err("An asset with that name already exists.".to_owned());
    }
    let id = format!(
        "{:x}-{:x}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        NEXT_ID.fetch_add(1, Ordering::Relaxed)
    );
    let extension = source
        .extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| {
            !extension.is_empty() && extension.chars().all(|char| char.is_ascii_alphanumeric())
        });
    let file = match extension {
        Some(extension) => format!("{id}.{extension}"),
        None => id.clone(),
    };
    let destination = folder.join(&file);
    std::fs::copy(source, &destination)
        .map_err(|error| format!("could not copy the asset into Concat: {error}"))?;

    let asset = Asset {
        id,
        name: name.to_owned(),
        description: description.to_owned(),
        source_name: source
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned(),
        file,
    };
    assets.push(asset.clone());
    let index = folder.join(INDEX);
    let temporary = folder.join("library.json.tmp");
    let write_result = (|| {
        let mut output = std::fs::File::create(&temporary)
            .map_err(|error| format!("could not save the asset library: {error}"))?;
        serde_json::to_writer_pretty(&mut output, &AssetIndex { version: 1, assets })
            .map_err(|error| format!("could not save the asset library: {error}"))?;
        output
            .sync_all()
            .map_err(|error| format!("could not save the asset library: {error}"))?;
        std::fs::rename(&temporary, &index)
            .map_err(|error| format!("could not save the asset library: {error}"))
    })();
    if let Err(error) = write_result {
        let _ = std::fs::remove_file(&destination);
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    Ok(asset)
}

/// Resolve the stored file without allowing an edited index to escape its folder.
pub fn path(dirs: &AppDirs, asset: &Asset) -> Option<PathBuf> {
    safe_file_name(&asset.file).then(|| folder(dirs).join(&asset.file))
}

fn safe_file_name(file: &str) -> bool {
    let mut components = Path::new(file).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

#[derive(Deserialize, Serialize)]
struct AssetIndex {
    version: u32,
    assets: Vec<Asset>,
}
