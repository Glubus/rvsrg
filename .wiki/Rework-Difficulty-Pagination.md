# Rework : Difficulty Calculators + Song Select Pagination

## 📋 Vue d'ensemble

### Problèmes actuels
1. **Scan lent** : Calcul de difficulté à l'import (MinaCalc + rosu-pp)
2. **Mémoire** : Toutes les beatmaps chargées en RAM
3. **Pas extensible** : Calculateurs hardcodés (etterna, osu)

### Solution proposée
1. **Scan léger** : Métadonnées seulement (hash, notes, durée)
2. **Calcul à la demande** : Quand on sélectionne une map
3. **Pagination** : 50 items max en mémoire, lazy-load
4. **Rhai scripts** : Calculateurs custom

---

## 🏗️ Architecture

### 1. Difficulty Calculators

```
src/difficulty/
├── mod.rs              # Gestionnaire principal
├── calculator.rs       # Trait DifficultyCalculator
├── builtin/
│   ├── etterna.rs      # Calculateur Etterna (MinaCalc)
│   ├── osu.rs          # Calculateur osu! (rosu-pp)
│   └── mod.rs
├── scripted/
│   ├── engine.rs       # Moteur Rhai
│   ├── context.rs      # Données exposées aux scripts
│   └── mod.rs
└── registry.rs         # Registre des calculateurs
```

#### Trait Calculator

```rust
pub trait DifficultyCalculator: Send + Sync {
    /// Identifiant unique (ex: "etterna_v4.0", "osu_v1.0", "custom_nps_v1")
    fn id(&self) -> &str;
    
    /// Nom affiché
    fn display_name(&self) -> &str;
    
    /// Version pour invalidation du cache
    fn version(&self) -> &str;
    
    /// Calcule la difficulté pour une map à un rate donné
    fn calculate(&self, ctx: &CalculationContext) -> Result<BeatmapSsr, CalcError>;
    
    /// Peut calculer pour n'importe quel rate? (sinon rates discrets)
    fn supports_arbitrary_rates(&self) -> bool { false }
    
    /// Rates disponibles si discrets
    fn available_rates(&self) -> Option<Vec<f64>> { None }
}

pub struct CalculationContext {
    pub notes: Vec<NoteInfo>,      // timestamp, column, is_hold, hold_duration
    pub key_count: u8,
    pub duration_ms: f64,
    pub bpm: f64,
    pub rate: f64,
    pub nps: f64,
    // Résultats d'autres calculateurs (pour hybrides)
    pub other_results: HashMap<String, BeatmapSsr>,
}
```

#### DB Schema Update

```sql
-- Modifier beatmap_rating pour inclure calculator_id
ALTER TABLE beatmap_rating ADD COLUMN calculator_id TEXT NOT NULL DEFAULT 'etterna_v4.0';
ALTER TABLE beatmap_rating ADD COLUMN rate REAL NOT NULL DEFAULT 1.0;

-- Index pour lookup rapide
CREATE INDEX idx_rating_lookup ON beatmap_rating(beatmap_hash, calculator_id, rate);
```

### 2. Song Select Pagination

#### Nouveau MenuState

```rust
pub struct MenuState {
    // Pagination
    pub total_count: usize,           // Nombre total en DB
    pub page_size: usize,             // 50
    pub current_offset: usize,        // Offset actuel
    pub loaded_beatmapsets: Vec<(Beatmapset, Vec<BeatmapLight>)>,
    
    // Sélection
    pub global_selected_index: usize, // Index global (0..total_count)
    pub selected_difficulty_index: usize,
    
    // Cache de difficulté
    pub difficulty_cache: HashMap<(String, String, OrderedFloat<f64>), BeatmapSsr>,
    // key = (beatmap_hash, calculator_id, rate)
    
    // Calculateur actif
    pub active_calculator: String,    // "etterna_v4.0", "osu_v1.0", etc.
    
    // ... autres champs existants
}

// Beatmap léger (pas de ratings chargés par défaut)
pub struct BeatmapLight {
    pub hash: String,
    pub difficulty_name: Option<String>,
    pub note_count: i32,
    pub duration_ms: i32,
    pub nps: f64,
    pub path: String,
}
```

#### Queries paginées

```rust
impl Database {
    /// Compte le total de beatmapsets (avec filtres)
    pub async fn count_beatmapsets(&self, filters: &MenuSearchFilters) -> Result<usize>;
    
    /// Récupère une page de beatmapsets (sans ratings)
    pub async fn get_beatmapsets_page(
        &self,
        offset: usize,
        limit: usize,
        filters: &MenuSearchFilters,
    ) -> Result<Vec<(Beatmapset, Vec<BeatmapLight>)>>;
    
    /// Récupère le rating caché pour une map
    pub async fn get_cached_rating(
        &self,
        beatmap_hash: &str,
        calculator_id: &str,
        rate: f64,
    ) -> Result<Option<BeatmapSsr>>;
    
    /// Sauvegarde un rating calculé
    pub async fn cache_rating(
        &self,
        beatmap_hash: &str,
        calculator_id: &str,
        rate: f64,
        ssr: &BeatmapSsr,
    ) -> Result<()>;
}
```

### 3. Flow de calcul

```
┌─────────────────────────────────────────────────────────────────┐
│                        User selects map                         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  Check DB cache: (hash, calculator_id, rate)                    │
│  - Si rate == 1.0 et calculator builtin → très probable en DB   │
│  - Si rate != 1.0 → peut-être pas en DB                         │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┴───────────────┐
              │                               │
        Cache HIT                        Cache MISS
              │                               │
              ▼                               ▼
┌─────────────────────┐        ┌─────────────────────────────────┐
│  Return cached SSR  │        │  Load .osu file                 │
└─────────────────────┘        │  Parse notes                    │
                               │  Call calculator.calculate()    │
                               │  Save to DB cache               │
                               │  Return SSR                     │
                               └─────────────────────────────────┘
```

### 4. Scroll/Pagination Flow

```
┌─────────────────────────────────────────────────────────────────┐
│  State: offset=0, loaded=[0..49], selected=25                   │
└─────────────────────────────────────────────────────────────────┘
                              │
                     User scrolls down
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  selected=60 (hors de [0..49])                                  │
│  → Déclenche chargement                                         │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│  new_offset = selected - 25 = 35  (centré)                      │
│  DB query: OFFSET 35 LIMIT 50                                   │
│  State: offset=35, loaded=[35..84], selected=60                 │
└─────────────────────────────────────────────────────────────────┘
```

#### Règles de rechargement

```rust
const PAGE_SIZE: usize = 50;
const PRELOAD_MARGIN: usize = 10;  // Recharge si à 10 items du bord

fn should_reload(selected: usize, offset: usize) -> Option<usize> {
    let local_idx = selected.saturating_sub(offset);
    let loaded_count = PAGE_SIZE;
    
    // Trop près du début
    if local_idx < PRELOAD_MARGIN && offset > 0 {
        return Some(selected.saturating_sub(PAGE_SIZE / 2));
    }
    
    // Trop près de la fin
    if local_idx > loaded_count - PRELOAD_MARGIN {
        return Some(selected.saturating_sub(PAGE_SIZE / 2));
    }
    
    None
}
```

---

## 📁 Structure des fichiers Rhai

```
skins/
└── default/
    └── calculators/
        ├── manifest.toml       # Liste des calculateurs
        ├── simple_nps.rhai
        ├── density.rhai
        └── hybrid.rhai
```

### manifest.toml

```toml
[[calculator]]
id = "simple_nps"
name = "Simple NPS"
version = "1.0"
file = "simple_nps.rhai"

[[calculator]]
id = "density_analyzer"
name = "Density Analyzer"
version = "1.2"
file = "density.rhai"
```

### Exemple script Rhai

```rhai
// simple_nps.rhai
// Contexte disponible: ctx.notes, ctx.key_count, ctx.duration_ms, ctx.rate, ctx.nps, ctx.bpm

fn calculate(ctx) {
    let base_diff = ctx.nps * 2.0;
    
    // Ajustement selon la durée (stamina)
    let duration_factor = if ctx.duration_ms > 180000 {
        1.15
    } else if ctx.duration_ms > 120000 {
        1.08
    } else {
        1.0
    };
    
    // Ajustement selon le key count
    let key_factor = match ctx.key_count {
        4 => 1.0,
        5 => 1.05,
        6 => 1.1,
        7 => 1.15,
        _ => 1.2
    };
    
    let overall = base_diff * duration_factor * key_factor * ctx.rate;
    
    // Retourne un objet avec tous les champs requis
    #{
        overall: overall,
        stream: overall * 0.8,
        jumpstream: overall * 0.85,
        handstream: overall * 0.7,
        stamina: overall * duration_factor,
        jackspeed: overall * 0.5,
        chordjack: overall * 0.6,
        technical: overall * 0.4
    }
}
```

---

## ⏱️ Estimation

| Tâche | Temps |
|-------|-------|
| 1. Trait Calculator + builtins | 2h |
| 2. Modifier scanner (no calc) | 30min |
| 3. DB schema + queries paginées | 1h |
| 4. MenuState pagination | 2h |
| 5. UI song_list lazy-load | 1h |
| 6. Rhai engine integration | 2h |
| 7. UI settings calculateur | 1h |
| 8. Tests + debug | 2h |
| **Total** | **~12h** |

---

## 🚀 Ordre d'implémentation

1. **Phase 1 : Foundation** (4h)
   - [ ] Trait `DifficultyCalculator`
   - [ ] Adapter etterna.rs et osu.rs
   - [ ] Modifier scanner (skip calc)
   - [ ] DB schema migration

2. **Phase 2 : Pagination** (3h)
   - [ ] Queries paginées
   - [ ] `MenuState` refactor
   - [ ] `song_list.rs` lazy-load

3. **Phase 3 : On-demand calc** (2h)
   - [ ] Cache lookup/save
   - [ ] Calculate on map select
   - [ ] Handle rate changes

4. **Phase 4 : Rhai** (3h)
   - [ ] Rhai engine setup
   - [ ] Context exposition
   - [ ] Scripts exemples
   - [ ] UI selector

---

## ❓ Questions ouvertes

1. **Invalidation du cache** : Si on update un calculateur, comment invalider les anciennes valeurs?
   → Solution: `calculator_id` inclut la version

2. **Scan existant** : Que faire des ratings déjà en DB?
   → Garder, ils seront "etterna_v4.0" par défaut

3. **Performance Rhai** : Acceptable pour ~1000 maps?
   → Oui, scripts simples < 1ms, calcul on-demand seulement

4. **Rates pour scripts** : Discrets ou arbitraires?
   → Les scripts supportent arbitraires par défaut

