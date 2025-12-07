use crate::models::stats::Judgement;
use wgpu_text::glyph_brush::{Section, Text};

#[derive(Clone)]
struct HitMarker {
    timing: f64,
    judgement: Judgement,
}

pub struct HitBarDisplay {
    position: (f32, f32),
    size: (f32, f32),
    last_hits: Vec<HitMarker>,
    max_history: usize,
}

impl HitBarDisplay {
    pub fn new(x_pixels: f32, y_pixels: f32, width_pixels: f32, height_pixels: f32) -> Self {
        Self {
            position: (x_pixels, y_pixels),
            size: (width_pixels, height_pixels),
            last_hits: Vec::with_capacity(10),
            max_history: 10,
        }
    }

    pub fn set_geometry(
        &mut self,
        x_pixels: f32,
        y_pixels: f32,
        width_pixels: f32,
        height_pixels: f32,
    ) {
        self.position = (x_pixels, y_pixels);
        self.size = (width_pixels, height_pixels);
    }

    fn push_hit(&mut self, timing: f64, judgement: Judgement) {
        let is_new = self
            .last_hits
            .last()
            .map(|hit| hit.timing != timing || hit.judgement != judgement)
            .unwrap_or(true);

        if is_new {
            self.last_hits.push(HitMarker { timing, judgement });
            if self.last_hits.len() > self.max_history {
                self.last_hits.remove(0);
            }
        }
    }

    fn timing_to_x(&self, timing_ms: f64) -> f32 {
        let (width, _) = self.size;
        let center_x = self.position.0 + (width / 2.0);
        let max_timing = 200.0;
        let ratio = (timing_ms / max_timing).clamp(-1.0, 1.0) as f32;
        center_x - (ratio * (width / 2.0))
    }

    #[inline]
    fn judgement_color(judgement: Judgement) -> [f32; 4] {
        match judgement {
            Judgement::Marv => [0.0, 1.0, 1.0, 1.0],
            Judgement::Perfect => [1.0, 1.0, 0.0, 1.0],
            Judgement::Great => [0.0, 1.0, 0.0, 1.0],
            Judgement::Good => [0.0, 0.0, 1.0, 1.0],
            Judgement::Bad => [1.0, 0.0, 1.0, 1.0],
            Judgement::Miss => [1.0, 0.0, 0.0, 1.0],
            Judgement::GhostTap => [0.5, 0.5, 0.5, 1.0],
        }
    }

    pub fn render(
        &mut self,
        latest_hit: Option<(f64, Judgement)>,
        screen_width: f32,
        screen_height: f32,
    ) -> Vec<Section<'_>> {
        if let Some((timing, judgement)) = latest_hit {
            self.push_hit(timing, judgement);
        }

        let mut sections = Vec::new();
        let (width, height) = self.size;
        let center_x = self.position.0 + (width / 2.0);

        sections.push(Section {
            screen_position: (center_x, self.position.1),
            bounds: (screen_width, screen_height),
            text: vec![
                Text::new("|")
                    .with_scale(height)
                    .with_color([1.0, 1.0, 1.0, 1.0]),
            ],
            ..Default::default()
        });

        for hit in &self.last_hits {
            sections.push(Section {
                screen_position: (self.timing_to_x(hit.timing), self.position.1),
                bounds: (screen_width, screen_height),
                text: vec![
                    Text::new("|")
                        .with_scale(height * 0.9)
                        .with_color(Self::judgement_color(hit.judgement)),
                ],
                ..Default::default()
            });
        }

        sections
    }
}
