//! Sons du client : musiques `.ogg` (`Sound\Music\`, issues de `Resource\music`) et effets
//! `.wav` de l'interface. Volumes réglés par `WTF/Config.wtf` (`Sound_MusicVolume`,
//! `Sound_SFXVolume`), comme dans WoW.

use std::collections::HashMap;

use macroquad::audio::{load_sound_from_bytes, play_sound, stop_sound, PlaySoundParams, Sound};

use crate::assets::Assets;

/// Musique de l'écran de connexion et du lobby.
pub const LOBBY_MUSIC: &str = "rappelz_newmain.ogg";
/// Musiques jouées en alternance dans le monde.
pub const FIELD_MUSIC: [&str; 6] = [
    "11_horaizen_in_the_sunset.ogg",
    "19_morning_breaks_of_rondoh.ogg",
    "14_breeze_of_old_forest.ogg",
    "04_resonance_of_world.ogg",
    "10_gaia.ogg",
    "13_this_is_the_sun_of_today.ogg",
];
pub const CLICK: &str = "ui_button_click.wav";

pub struct Audio {
    music: Option<(String, Sound)>,
    effects: HashMap<String, Option<Sound>>,
    music_volume: f32,
    sfx_volume: f32,
}

impl Audio {
    pub fn new(music_volume: f32, sfx_volume: f32) -> Self {
        Self { music: None, effects: HashMap::new(), music_volume, sfx_volume }
    }

    /// Lance une musique en boucle (ne fait rien si elle joue déjà).
    pub async fn play_music(&mut self, assets: &mut Assets, name: &str) {
        if self.music.as_ref().is_some_and(|(n, _)| n == name) || self.music_volume <= 0.0 {
            return;
        }
        if let Some((_, old)) = self.music.take() {
            stop_sound(&old);
        }
        let Some(bytes) = assets.read(name) else {
            tracing::warn!("musique introuvable : {name}");
            return;
        };
        match load_sound_from_bytes(&bytes).await {
            Ok(sound) => {
                play_sound(&sound, PlaySoundParams { looped: true, volume: self.music_volume });
                self.music = Some((name.to_string(), sound));
            }
            Err(e) => tracing::warn!("{name} : {e:?}"),
        }
    }

    /// Effet sonore (mis en cache après le premier chargement).
    pub async fn effect(&mut self, assets: &mut Assets, name: &str) {
        if self.sfx_volume <= 0.0 {
            return;
        }
        if !self.effects.contains_key(name) {
            let sound = match assets.read(name) {
                Some(bytes) => load_sound_from_bytes(&bytes).await.ok(),
                None => None,
            };
            self.effects.insert(name.to_string(), sound);
        }
        if let Some(Some(s)) = self.effects.get(name) {
            play_sound(s, PlaySoundParams { looped: false, volume: self.sfx_volume });
        }
    }
}
