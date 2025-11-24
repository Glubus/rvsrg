use wgpu::Device;
use wgpu::TextureFormat;
use wgpu_text::{BrushBuilder, TextBrush};

/// Crée un TextBrush minimal avec une police par défaut (pour éviter les timeouts)
pub fn create_minimal_text_brush(
    device: &Device,
    width: u32,
    height: u32,
    format: TextureFormat,
) -> TextBrush {
    use wgpu_text::glyph_brush::ab_glyph::FontArc;
    // Essayer de charger une police depuis assets/font.ttf si elle existe
    if let Ok(font_data) = std::fs::read("assets/font.ttf") {
        if let Ok(font) = FontArc::try_from_vec(font_data) {
            return BrushBuilder::using_font(font).build(device, width, height, format);
        }
    }

    // Si pas de police disponible, on crée quand même un TextBrush
    // Le texte ne s'affichera pas mais l'app ne plantera pas
    eprintln!("WARNING: No font available, text will not be displayed");
    // On doit créer un TextBrush valide - utiliser une police système si possible
    // Sinon, on utilisera une police par défaut de wgpu_text
    // Créer une police minimale avec des données vides (cela échouera probablement)
    // Mais au moins on essaie
    let empty_font = FontArc::try_from_vec(vec![]).ok();
    if let Some(font) = empty_font {
        BrushBuilder::using_font(font).build(device, width, height, format)
    } else {
        // Dernier recours : panic car on ne peut pas créer de TextBrush sans police
        panic!(
            "Cannot create TextBrush without font. Please place a 'font.ttf' file in 'assets/font.ttf' or in the skin."
        );
    }
}

/// Charge un TextBrush depuis un chemin de police
pub fn load_text_brush(
    device: &Device,
    width: u32,
    height: u32,
    format: TextureFormat,
    font_path: &std::path::Path,
) -> TextBrush {
    use wgpu_text::glyph_brush::ab_glyph::FontArc;

    eprintln!("Attempting to load font from: {:?}", font_path);

    match std::fs::read(font_path) {
        Ok(font_data) => match FontArc::try_from_vec(font_data) {
            Ok(font) => BrushBuilder::using_font(font).build(device, width, height, format),
            Err(e) => {
                eprintln!("Error parsing font: {}", e);
                create_minimal_text_brush(device, width, height, format)
            }
        },
        Err(e) => {
            eprintln!("Error reading file {:?}: {}", font_path, e);
            create_minimal_text_brush(device, width, height, format)
        }
    }
}
