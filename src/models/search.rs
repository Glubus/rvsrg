#[derive(Debug, Clone, PartialEq, Default)]
pub enum RatingSource {
    #[default]
    Etterna,
    Osu,
}

impl RatingSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            RatingSource::Etterna => "etterna",
            RatingSource::Osu => "osu",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum RatingMetric {
    #[default]
    Overall,
    Stream,
    Jumpstream,
    Handstream,
    Stamina,
    Jackspeed,
    Chordjack,
    Technical,
}

impl RatingMetric {
    pub fn column_name(&self) -> &'static str {
        match self {
            RatingMetric::Overall => "overall",
            RatingMetric::Stream => "stream",
            RatingMetric::Jumpstream => "jumpstream",
            RatingMetric::Handstream => "handstream",
            RatingMetric::Stamina => "stamina",
            RatingMetric::Jackspeed => "jackspeed",
            RatingMetric::Chordjack => "chordjack",
            RatingMetric::Technical => "technical",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            RatingMetric::Overall => "Overall",
            RatingMetric::Stream => "Stream",
            RatingMetric::Jumpstream => "Jumpstream",
            RatingMetric::Handstream => "Handstream",
            RatingMetric::Stamina => "Stamina",
            RatingMetric::Jackspeed => "Jackspeed",
            RatingMetric::Chordjack => "Chordjack",
            RatingMetric::Technical => "Technical",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MenuSearchFilters {
    pub query: String,
    pub min_rating: Option<f64>,
    pub max_rating: Option<f64>,
    pub rating_source: RatingSource,
    pub rating_metric: RatingMetric,
    pub min_duration_seconds: Option<f64>,
    pub max_duration_seconds: Option<f64>,
}

impl MenuSearchFilters {
    pub fn is_active(&self) -> bool {
        !self.query.trim().is_empty()
            || self.min_rating.is_some()
            || self.max_rating.is_some()
            || self.min_duration_seconds.is_some()
            || self.max_duration_seconds.is_some()
    }
}

