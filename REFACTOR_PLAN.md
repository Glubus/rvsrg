## Objectif

Mettre en place une architecture MVC/State Pattern claire pour le jeu de rythme afin d’éliminer l’objet dieu `App`, séparer la logique pure (models) des vues WGPU, et isoler les contrôleurs (states). Les règles suivantes guident tout le refactoring : modèles sans dépendances WGPU/Winit, vues purement déclaratives (data-in → draw), contrôleurs responsables des transitions et de l’input, atomic design côté UI.

---

## Photo de l’existant (problèmes)

- `src/app.rs` combine gestion Winit, navigation menu/gameplay, accès DB et configuration du renderer. Le gros `match` sur `KeyCode` mélange logique menu/gameplay (`KeyCode::ArrowUp` vs `KeyCode::F3/F4`) et manipule directement `Renderer`.
- `src/renderer/core.rs` instancie l’intégralité du moteur (`GameEngine`), du `MenuState` et de chaque composant visuel. Les composants (ex. `JudgementsComponent`) possèdent des buffers de strings qui dupliquent l’état du moteur et créent directement les `Section` glyph_brush.
- `src/components` contient un mélange de widgets (score, combo), de layouts (map_list) et d’items moteur. Ils reçoivent `GameEngine` complet, donc impossible de tester/réutiliser sans WGPU.

---

## Cible – structure de fichiers

```
src/
├── app.rs                # Router + stack de GameState
├── main.rs               # Bootstrap -> App
├── renderer.rs           # Façade WGPU (surface/device/pipelines, draw primitives)
├── models/
│   ├── engine.rs         # GameEngine, NoteData, HitWindow, PlayfieldConfig, Skin config-free helpers
│   ├── settings.rs       # Menu/jeu: Rate, scroll speed prefs, input mapping, pixel metrics config
│   └── stats.rs          # HitStats, Judgement, Accuracy/Score structs, DTOs pour les vues
├── states/
│   ├── mod.rs            # Trait GameState { fn update, handle_input, render, on_enter, on_exit }
│   ├── menu.rs           # MenuController : interagit avec DB + Router transitions
│   └── play.rs           # GameplayController : orchestre GameEngine + transitions
├── views/
│   ├── gameplay.rs       # Layout principal du gameplay (compose components)
│   ├── menu.rs           # Layout principal du menu
│   └── components/
│       ├── judgement.rs      # Présentation des jugements (config visuelle + render(data))
│       ├── hit_bar.rs
│       ├── stats_panel.rs
│       ├── map_list.rs
│       └── ...
└── ...
```

### Nouveaux fichiers détaillés

| Fichier | Contenu principal | Sources d’origine |
| --- | --- | --- |
| `models/engine.rs` | `GameEngine`, `NoteData`, `HitWindow`, `PixelSystem`, logique audio/scroll. Expose DTOs pour vues (ex `VisibleNote { column, offset, size }`). | `src/engine.rs`, parties de `renderer/core.rs` qui touchent le moteur. |
| `models/stats.rs` | `HitStats`, `Judgement` enum, `JudgementColors` (reste pure data), structures de scoreboard (`ComboState`, `ScoreBreakdown`). | `src/engine.rs` + `components/judgements.rs` (données). |
| `models/settings.rs` | `MenuState` renommé `MenuModel`, préférences utilisateur (rate, keybinds). Sans WGPU. | `src/menu.rs`, champs de `Renderer::new` liés à layout statique. |
| `states/mod.rs` | Trait `GameState`, enum `StateTransition` (Push/Pop/Replace/None). | Nouveau. |
| `states/menu.rs` | Contrôleur menu. Consomme `MenuModel`, demande au renderer de dessiner `views::menu::MenuView`, déclenche transitions (ex `Enter` -> `PlayState`). | Logique `App::window_event` (partie menu), `renderer::menu::render_menu`. |
| `states/play.rs` | Contrôleur gameplay. Contient `GameEngine`, gère inputs gameplay, appelle `views::gameplay::render`. | `App` gameplay branch + `renderer::gameplay`. |
| `views/gameplay.rs` | Compose `PlayfieldView`, `JudgementsView`, `HitBarView`, etc. Reçoit un `GameplayViewModel` (struct) issu du contrôleur. | `renderer::gameplay::render_gameplay`, `components/*`. |
| `views/menu.rs` | Compose `MapList`, `SongDetails`, background. Reçoit `MenuViewModel`. | `renderer::menu::render_menu`, `components/map_list.rs`. |
| `views/components/*.rs` | Widgets testables (struct config + `fn render(&self, data, renderer_ctx)`). | `src/components/*`. |
| `renderer.rs` | Rassemble ce qui est dans `renderer/core.rs` (surface, pipelines, text brush). Expose: `fn render(&mut self, view_commands: &[DrawCommand])`, `fn submit_text(&mut self, TextBatch)` etc. | `src/renderer/core.rs` + sous-mods. |

---

## Mapping des entités actuelles

| Entité actuelle | Destination | Commentaire |
| --- | --- | --- |
| `App` (gestion input/menu/game + renderer) | `app.rs` (Router + stack) + `states/*` | `App` ne manipule plus `KeyCode` directement. Il délègue `handle_input(WindowEvent, &mut GameState)`. |
| `MenuState` (`src/menu.rs`) | `models/settings.rs::MenuModel` | Ajout de méthodes pures (ex `fn visible_slice(&self) -> MenuSlice`). Pas de WGPU. |
| `GameEngine`, `HitStats`, `Judgement`, `PixelSystem`, `PlayfieldConfig` | `models/engine.rs` + `models/stats.rs` | `PixelSystem` reste model math (aucune dépendance WGPU). |
| `Renderer` + pipelines | `renderer.rs` + sous-modules conservés (`renderer/texture.rs`, etc.) | Interface minimaliste: `renderer.render(&RenderGraph)` + utilitaires de chargement. |
| `components/*` | Se scindent en `views/components/*` (config + draw) + `models/stats.rs` (données) | Exemple: `JudgementsComponent` -> `HitStats` (model) + `views/components/judgement.rs` (présentation). |
| `renderer/menu.rs` & `renderer/gameplay.rs` | `views/menu.rs` & `views/gameplay.rs` | Les fonctions actuelles seront transposées en vues data-driven, sans accès direct au `Renderer`. Elles produiront des `DrawCommand`s. |
| `renderer/text.rs`, `renderer/pipeline.rs`, etc. | Reste dans `/renderer/` mais renommés si nécessaire | Ils deviennent des détails internes au nouveau `renderer.rs`. |

---

## Flow attendu (après refactor)

1. `main.rs` crée `App`.
2. `App` instancie un `Renderer` + stack vide, pousse `MenuState`.
3. Chaque frame:
   - `App` route les évènements Winit vers `active_state.handle_input(...) -> StateTransition`.
   - `App` appelle `active_state.update(dt) -> Option<StateTransition>`.
   - `App` appelle `active_state.render(&mut renderer)` : le state construit un `ViewModel`, appelle sa vue (`views::...::render(model, &mut renderer)`), qui pousse des `DrawCommand`s au renderer.
4. Si un `StateTransition` est renvoyé, `App` manipule la stack (Push/Pop/Replace).

---

## Ordre des opérations conseillé

1. **Préparer les dossiers**  
   - Créer `src/models`, `src/states`, `src/views`, `src/views/components`.  
   - Ajouter modules vides + re-export dans `src/lib.rs`/`main.rs` si nécessaire.

2. **Extraire les Models**  
   - Déplacer `GameEngine`, `HitWindow`, `NoteData`, `PixelSystem`, `PlayfieldConfig`, `Judgement`, `HitStats`, `JudgementColors` dans `models/*`.  
   - Adapter les `use` dans le code existant pour pointer vers `crate::models::engine` etc. (aucun changement fonctionnel).

3. **Scinder `MenuState`**  
   - Renommer en `MenuModel` (ou `MenuData`) placé dans `models/settings.rs`.  
   - Fournir méthodes pures produisant des structs légers pour les vues (`MenuViewModel`, `BeatmapSummary`).

4. **Introduire le trait `GameState`**  
   - `states/mod.rs`: définir Trait + enum `Transition`.  
   - Ajouter `MenuStateController` et `PlayStateController` qui wrappe respectivement `MenuModel` et `GameEngine`.  
   - `App` devient un routeur: contient `Vec<Box<dyn GameState>>`, délègue input/update/render.

5. **Adapter le Renderer**  
   - Extraire `renderer/core.rs` → `renderer.rs` façade.  
   - Déplacer les fonctions `render_menu` / `render_gameplay` vers `views/menu.rs` & `views/gameplay.rs`, en conservant les helpers WGPU existants dans `renderer/`.  
   - Introduire un intermédiaire `RenderContext` passé par les vues (abstraction pour dessiner textes/quads/instances sans exposer tout `Renderer`).

6. **Refactorer les composants en vues Atomic**  
   - Pour chaque fichier de `src/components`, créer une contrepartie `views/components/*.rs` avec:  
     ```rust
     pub struct JudgementDisplay { position: Vec2, colors: JudgementColors }
     impl JudgementDisplay {
         pub fn render(&self, stats: &HitStats, ctx: &mut RenderContext);
     }
     ```  
   - Les données (strings, compteurs) viennent du `GameState` (models/stats), plus de mutation locale.

7. **Migrer les states vers les nouvelles vues**  
   - `states::play::render` construit `GameplayViewModel` (combo, accuracy, visible notes, etc.) à partir du moteur, appelle `views::gameplay::render(&model, ctx)`.  
   - `states::menu::render` construit `MenuViewModel` à partir du `MenuModel`.

8. **Nettoyage final**  
   - Supprimer `src/components`, `src/menu.rs`, `src/engine.rs`, `src/renderer/core.rs` une fois les versions MVC intégrées.  
   - Mettre à jour `mod.rs`/`lib.rs`, tests éventuels, ajuster import paths.  
   - Vérifier que `models` n’importent ni `wgpu` ni `winit`, et que `views` n’accèdent qu’à des ViewModels.

---

## Points de vigilance

- **Synchronisation audio** : `GameEngine` reste propriétaire des ressources audio (rodio). Lors du déplacement dans `models/engine.rs`, veiller à laisser les dépendances `rodio` côté model (acceptable car audio != wgpu/winit).  
- **Thread safety** : actuellement `Renderer::menu_state` est un `Arc<Mutex<_>>`. Avec le State Pattern, préférer faire vivre le `MenuModel` dans `MenuStateController` et ne plus l’embarquer dans `Renderer`. Seul un `ViewModel` immutable est passé au renderer par frame.  
- **Transition Menu → Play** : `MenuStateController` construit `PlayStateController` en transférant la sélection (map path, rate). Prévoir un canal (ex enum `StateTransition::Replace(Box<dyn GameState>)`).  
- **Dépendances circulaires** : `Renderer` ne doit plus connaître `MenuState`. Les states importent `renderer::RenderContext`.  
- **Build intermédiaire** : pendant les étapes 2-6, garder des réexports temporaires (`pub use crate::models::engine::*`) pour éviter de casser les imports. Reporter la suppression des anciens modules à l’étape 8.

---

## Livrables attendus ultérieurement

- Implémentation concrète des fichiers listés ci-dessus.  
- Tests manuels: navigation menu/gameplay, rescan DB, réglages vitesse, reload map, redimensionnement fenêtre.  
- Documentation rapide (README section Architecture + diagramme des states).

---

Ce plan reste ouvert aux ajustements après revue, mais fournit un ordre logique pour isoler d’abord les models, introduire le State Pattern, puis reconstruire les vues et composants de manière data-driven sans bloquer le build trop longtemps.

