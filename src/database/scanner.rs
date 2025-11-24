use crate::database::connection::Database;
use crate::database::query::{insert_beatmap, insert_beatmapset};
use std::fs;
use std::path::{Path, PathBuf};

/// Scanne le dossier songs/ et remplit la base de données
pub async fn scan_songs_directory(
    db: &Database,
    songs_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if !songs_path.exists() {
        eprintln!("The songs/ directory does not exist");
        return Ok(());
    }

    println!("Scanning maps in {:?}...", songs_path);

    // Parcourir tous les dossiers dans songs/
    let entries = fs::read_dir(songs_path)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        // Chercher tous les fichiers .osu dans ce dossier
        let osu_files: Vec<PathBuf> = match fs::read_dir(&path) {
            Ok(dir) => dir
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().and_then(|s| s.to_str()) == Some("osu"))
                .collect(),
            Err(_) => continue,
        };

        if osu_files.is_empty() {
            continue;
        }

        // Utiliser le premier fichier .osu pour les métadonnées du beatmapset
        let first_osu = &osu_files[0];

        // Charger la map avec rosu-map
        match rosu_map::Beatmap::from_path(first_osu) {
            Ok(map) => {
                // Extraire les métadonnées depuis la map
                let title = map.title.clone();
                let artist = map.artist.clone();

                // Chercher l'image de fond dans les events
                let background_filename = map.background_file.clone();
                let image_path = if !background_filename.is_empty() {
                    find_background_image(&path, Some(background_filename.as_str()))
                } else {
                    None
                };

                // Insérer le beatmapset
                let path_str = match path.to_str() {
                    Some(s) => s,
                    None => continue,
                };
                let beatmapset_id = match insert_beatmapset(
                    db.pool(),
                    path_str,
                    image_path.as_deref(),
                    Some(artist.as_str()),
                    Some(title.as_str()),
                )
                .await
                {
                    Ok(id) => id,
                    Err(e) => {
                        eprintln!("Error inserting beatmapset: {}", e);
                        continue;
                    }
                };

                // Insérer toutes les beatmaps
                for osu_file in &osu_files {
                    match rosu_map::Beatmap::from_path(osu_file) {
                        Ok(bm) => {
                            // Compter les notes (seulement les circles pour l'instant)
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

                            if let Some(osu_str) = osu_file.to_str() {
                                if let Err(e) = insert_beatmap(
                                    db.pool(),
                                    beatmapset_id,
                                    osu_str,
                                    Some(&difficulty_name),
                                    note_count,
                                )
                                .await
                                {
                                    eprintln!("Error inserting beatmap: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error loading {:?}: {}", osu_file, e);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Error loading {:?}: {}", first_osu, e);
            }
        }
    }

    println!("Scan completed!");
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
