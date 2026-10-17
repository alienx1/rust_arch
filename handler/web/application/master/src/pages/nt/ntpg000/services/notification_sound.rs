//! The sound of a new notification: a short two-note chime made with the Web Audio API (no audio
//! file to ship), and the user's on/off choice (kept in the browser, `localStorage`).
//!
//! Browsers only let a page make sound after the user has interacted with it once, so the audio
//! context is created and resumed on the first click / key press; a notification that arrives
//! before that stays silent (the badge still shows it).

use std::cell::RefCell;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::Closure;
use web_sys::{AudioContext, AudioContextState, OscillatorType};

const STORAGE_KEY: &str = "notification_sound";

thread_local! {
    static CONTEXT: RefCell<Option<AudioContext>> = const { RefCell::new(None) };
}

/// Whether the user wants a sound (on unless switched off)
pub fn sound_enabled() -> bool {
    web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(STORAGE_KEY).ok().flatten())
        .map(|v| v != "off")
        .unwrap_or(true)
}

pub fn save_sound_enabled(on: bool) {
    if let Some(storage) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = storage.set_item(STORAGE_KEY, if on { "on" } else { "off" });
    }
}

fn context() -> Option<AudioContext> {
    CONTEXT.with(|c| {
        let mut c = c.borrow_mut();
        if c.is_none() {
            *c = AudioContext::new().ok();
        }
        c.clone()
    })
}

/// Creates / resumes the audio context at the user's first click or key press, so a later push can
/// make sound
pub fn unlock_audio_on_first_gesture() {
    let Some(document) = web_sys::window().and_then(|w| w.document()) else { return };
    for event in ["pointerdown", "keydown"] {
        let handler = Closure::once_into_js(move || {
            if let Some(ctx) = context() {
                let _ = ctx.resume();
            }
        });
        let options = web_sys::AddEventListenerOptions::new();
        options.set_once(true);
        let _ = document.add_event_listener_with_callback_and_add_event_listener_options(
            event,
            handler.unchecked_ref(),
            &options,
        );
    }
}

/// Plays the chime (does nothing when the browser still blocks audio)
pub fn play_chime() {
    let Some(ctx) = context() else { return };
    if ctx.state() != AudioContextState::Running {
        let _ = ctx.resume();
        return;
    }
    let now = ctx.current_time();
    // Two soft sine notes, E6 then A6, each fading out
    for (i, freq) in [1318.5_f32, 1760.0].into_iter().enumerate() {
        let start = now + i as f64 * 0.14;
        let (Ok(osc), Ok(gain)) = (ctx.create_oscillator(), ctx.create_gain()) else { return };
        osc.set_type(OscillatorType::Sine);
        osc.frequency().set_value(freq);
        let level = gain.gain();
        let _ = level.set_value_at_time(0.0001, start);
        let _ = level.exponential_ramp_to_value_at_time(0.18, start + 0.02);
        let _ = level.exponential_ramp_to_value_at_time(0.0001, start + 0.30);
        let _ = osc.connect_with_audio_node(&gain);
        let _ = gain.connect_with_audio_node(&ctx.destination());
        let _ = osc.start_with_when(start);
        let _ = osc.stop_with_when(start + 0.32);
    }
}
