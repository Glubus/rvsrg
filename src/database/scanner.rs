//! Filesystem scanner that imports beatmapsets into the database.

use crate::database::connection::Database;
use crate::database::query::{insert_beatmap, insert_beatmapset};
use crate::difficulty;
use md5::Context;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Scans the `songs/` directory and fills the database.
pub async fn scan_songs_directory(
    db: &Database,
    songs_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !songs_path.exists() {
        eprintln!("The songs/ directory does not exist");
        return Ok(());
    }

    difficulty::init_global_calc()?;

    // Walk every sub-folder under songs/.
    let entries = fs::read_dir(songs_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let osu_files = match collect_osu_files(&path) {
            Some(files) if !files.is_empty() => files,
            _ => continue,
        };

        if let Err(e) = process_beatmapset(db, &path, &osu_files).await {
            eprintln!("Error processing beatmapset {:?}: {}", path, e);
        }
    }

    Ok(())
}

fn collect_osu_files(path: &Path) -> Option<Vec<PathBuf>> {
    let entries = fs::read_dir(path).ok()?;
    let files = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("osu"))
        .collect::<Vec<_>>();
    Some(files)
}

async fn process_beatmapset(
    db: &Database,
    folder: &Path,
    osu_files: &[PathBuf],
) -> Result<(), Box<dyn std::error::Error>> {
    let Some(first_osu) = osu_files.first() else {
        return Ok(());
    };

    let map = rosu_map::Beatmap::from_path(first_osu)?;
    let title = map.title.clone();
    let artist = map.artist.clone();
    let image_path = if map.background_file.is_empty() {
        None
    } else {
        find_background_image(folder, Some(map.background_file.as_str()))
    };

    let Some(path_str) = folder.to_str() else {
        return Ok(());
    };

    let beatmapset_id = insert_beatmapset(
        db.pool(),
        path_str,
        image_path.as_deref(),
        Some(artist.as_str()),
        Some(title.as_str()),
    )
    .await?;

    for osu_file in osu_files {
        if let Err(e) = process_osu_file(db, beatmapset_id, osu_file).await {
            eprintln!("Error processing {:?}: {}", osu_file, e);
        }
    }

    Ok(())
}

async fn process_osu_file(
    db: &Database,
    beatmapset_id: i64,
    osu_file: &PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    let hash = calculate_file_hash(osu_file)?;
    let bm = rosu_map::Beatmap::from_path(osu_file)?;

    let note_count = bm
        .hit_objects
        .iter()
        .filter(|ho| {
            matches!(
                ho.kind,
                rosu_map::section::hit_objects::HitObjectKind::Circle(_)
            )
        })
        .count() as i32;

    let difficulty_name = bm.version.clone();
    let difficulty_info = difficulty::analyze(&bm)?;

    if let Some(osu_str) = osu_file.to_str() {
        insert_beatmap(
            db.pool(),
            beatmapset_id,
            &hash,
            osu_str,
            Some(&difficulty_name),
            note_count,
            difficulty_info.duration_ms,
            difficulty_info.nps,
        )
        .await?;

        for rating in &difficulty_info.ratings {
            db.upsert_beatmap_rating(
                &hash,
                &rating.name,
                rating.ssr.overall,
                rating.ssr.stream,
                rating.ssr.jumpstream,
                rating.ssr.handstream,
                rating.ssr.stamina,
                rating.ssr.jackspeed,
                rating.ssr.chordjack,
                rating.ssr.technical,
            )
            .await?;
        }
    }

    Ok(())
}

fn find_background_image(beatmapset_path: &Path, filename: Option<&str>) -> Option<String> {
    filename.and_then(|fname| {
        let image_path = beatmapset_path.join(fname);
        if image_path.exists() {
            image_path.to_str().map(|s| s.to_string())
        } else {
            None
        }
    })
}

/// Computes the MD5 hash for an `.osu` chart file.
fn calculate_file_hash(file_path: &Path) -> Result<String, std::io::Error> {
    let mut file = fs::File::open(file_path)?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    let mut context = Context::new();
    context.consume(buffer.as_bytes());
    let result = context.finalize();
    let hash_string = format!("{:x}", result);

    Ok(hash_string)
}
