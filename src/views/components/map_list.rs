use crate::database::models::{Beatmap, Beatmapset};
use crate::views::components::card::CardDisplay;
use bytemuck::{Pod, Zeroable};
use wgpu_text::glyph_brush::{Section, Text};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct QuadInstance {
    pub center: [f32; 2],
    pub size: [f32; 2],
    pub color: [f32; 4],
}

fn screen_to_normalized(x: f32, y: f32, width: f32, height: f32) -> [f32; 2] {
    [(x / width) * 2.0 - 1.0, -((y / height) * 2.0 - 1.0)]
}

fn create_quad(
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: [f32; 4],
    screen_width: f32,
    screen_height: f32,
) -> QuadInstance {
    let center = screen_to_normalized(
        x + width / 2.0,
        y + height / 2.0,
        screen_width,
        screen_height,
    );
    let size = [(width / screen_width) * 2.0, (height / screen_height) * 2.0];
    QuadInstance {
        center,
        size,
        color,
    }
}

pub struct MapListDisplay {
    pub cards: Vec<CardDisplay>,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub card_width: f32,
    pub card_height: f32,
    pub card_spacing: f32,
    pub difficulty_row_height: f32,
    pub difficulty_row_spacing: f32,
    pub detail_padding: f32,
    pub screen_width: f32,
    pub screen_height: f32,
    text_strings: Vec<String>,
}

impl MapListDisplay {
    pub fn new(screen_width: f32, screen_height: f32) -> Self {
        let card_width = 400.0;
        let card_height = 80.0;
        let card_spacing = 8.0;
        let difficulty_row_height = 40.0;
        let difficulty_row_spacing = 6.0;
        let detail_padding = 12.0;
        let width = card_width + 40.0;
        let x = screen_width - width;
        let y = 100.0;

        Self {
            cards: Vec::new(),
            x,
            y,
            width,
            card_width,
            card_height,
            card_spacing,
            difficulty_row_height,
            difficulty_row_spacing,
            detail_padding,
            screen_width,
            screen_height,
            text_strings: Vec::new(),
        }
    }

    pub fn update_cards(
        &mut self,
        visible_items: &[(Beatmapset, Vec<Beatmap>)],
        selected_index: usize,
        selected_difficulty_index: usize,
    ) {
        self.cards.clear();
        let mut current_y = self.y;

        for (idx, (beatmapset, beatmaps)) in visible_items.iter().enumerate() {
            let is_selected = idx == selected_index;
            let current_diff_index = if is_selected {
                selected_difficulty_index.min(beatmaps.len().saturating_sub(1))
            } else {
                0
            };

            let detail_height = if is_selected && !beatmaps.is_empty() {
                let rows = beatmaps.len() as f32;
                let spacing = beatmaps.len().saturating_sub(1) as f32;
                rows * self.difficulty_row_height
                    + spacing * self.difficulty_row_spacing
                    + self.detail_padding * 2.0
            } else {
                0.0
            };

            let total_height = self.card_height + detail_height;

            let card = CardDisplay::new(
                beatmapset.clone(),
                beatmaps.clone(),
                self.x + 20.0,
                current_y,
                self.card_width,
                total_height,
                is_selected,
                current_diff_index,
            );

            self.cards.push(card);
            current_y += total_height + self.card_spacing;
        }
    }

    pub fn create_quads(&self) -> Vec<QuadInstance> {
        let mut quads = Vec::new();

        if self.cards.is_empty() {
            return quads;
        }

        let cards_height: f32 = self.cards.iter().map(|card| card.height).sum();
        let spacing_total = if self.cards.is_empty() {
            0.0
        } else {
            (self.cards.len() as f32 - 1.0).max(0.0) * self.card_spacing
        };
        let total_height = cards_height + spacing_total + 40.0;

        quads.push(create_quad(
            self.x,
            self.y - 20.0,
            self.width,
            total_height,
            [0.0, 0.0, 0.0, 0.8],
            self.screen_width,
            self.screen_height,
        ));

        for card in &self.cards {
            quads.push(create_quad(
                card.x,
                card.y,
                card.width,
                card.height,
                card.background_color(),
                self.screen_width,
                self.screen_height,
            ));

            if card.is_selected && !card.beatmaps.is_empty() {
                let mut row_y = card.y + self.card_height + self.detail_padding;
                let row_width = card.width - 20.0;
                for (idx, _) in card.beatmaps.iter().enumerate() {
                    let color = if idx == card.current_difficulty_index {
                        [0.2, 0.2, 0.2, 0.95]
                    } else {
                        [0.1, 0.1, 0.1, 0.9]
                    };
                    quads.push(create_quad(
                        card.x + 10.0,
                        row_y,
                        row_width,
                        self.difficulty_row_height,
                        color,
                        self.screen_width,
                        self.screen_height,
                    ));
                    row_y += self.difficulty_row_height + self.difficulty_row_spacing;
                }
            }
        }

        quads
    }

    pub fn create_text_sections(&mut self) -> Vec<Section<'_>> {
        self.text_strings.clear();
        let mut card_data = Vec::new();

        for card in &self.cards {
            let title = card.title_text();
            let artist_difficulty = card.artist_difficulty_text();
            let text_color = card.text_color();

            let title_idx = self.text_strings.len();
            self.text_strings.push(title);
            let diff_idx = self.text_strings.len();
            self.text_strings.push(artist_difficulty);

            let difficulty_info = if card.is_selected && !card.beatmaps.is_empty() {
                let start_idx = self.text_strings.len();
                for beatmap in &card.beatmaps {
                    let diff_text = beatmap
                        .difficulty_name
                        .clone()
                        .unwrap_or_else(|| "Unknown".to_string());
                    self.text_strings.push(diff_text);
                }
                Some((start_idx, card.beatmaps.len()))
            } else {
                None
            };

            card_data.push((
                card.x,
                card.y,
                card.width,
                card.height,
                text_color,
                title_idx,
                diff_idx,
                difficulty_info,
            ));
        }

        let mut sections = Vec::new();
        for (card_idx, (x, y, width, height, text_color, title_idx, diff_idx, difficulty_info)) in
            card_data.into_iter().enumerate()
        {
            sections.push(Section {
                screen_position: (x + 15.0, y + 15.0),
                bounds: (width - 30.0, height),
                text: vec![
                    Text::new(&self.text_strings[title_idx])
                        .with_scale(28.0)
                        .with_color(text_color),
                ],
                ..Default::default()
            });

            sections.push(Section {
                screen_position: (x + 15.0, y + 50.0),
                bounds: (width - 30.0, height),
                text: vec![
                    Text::new(&self.text_strings[diff_idx])
                        .with_scale(18.0)
                        .with_color([0.7, 0.7, 0.7, 1.0]),
                ],
                ..Default::default()
            });

            if let (Some(card), Some((start_idx, count))) =
                (self.cards.get(card_idx), difficulty_info)
            {
                let mut row_y = y + self.card_height + self.detail_padding + 10.0;
                for idx in 0..count {
                    let string_idx = start_idx + idx;
                    let color = if idx == card.current_difficulty_index {
                        [1.0, 0.85, 0.4, 1.0]
                    } else {
                        [0.8, 0.8, 0.8, 1.0]
                    };
                    sections.push(Section {
                        screen_position: (x + 25.0, row_y),
                        bounds: (width - 50.0, self.difficulty_row_height),
                        text: vec![
                            Text::new(&self.text_strings[string_idx])
                                .with_scale(22.0)
                                .with_color(color),
                        ],
                        ..Default::default()
                    });
                    row_y += self.difficulty_row_height + self.difficulty_row_spacing;
                }
            }
        }

        sections
    }

    pub fn update_size(&mut self, screen_width: f32, screen_height: f32) {
        self.screen_width = screen_width;
        self.screen_height = screen_height;
        self.width = self.card_width + 40.0;
        self.x = screen_width - self.width;
    }
}
