use crate::app_data::{AppData, AppState};
use vizia::prelude::*;

pub fn note_display(cx: &mut Context) {
    Binding::new(cx, AppData::app_state, |cx, state| {
        if state.get(cx) == AppState::Recording {
            HStack::new(cx, |cx| {
                // Current note name (large)
                Label::new(
                    cx,
                    AppData::current_note.map(|n: &u8| AppData::note_name(*n)),
                )
                .class("note-name");

                // Velocity
                Label::new(
                    cx,
                    AppData::current_velocity.map(|v: &u8| format!("vel {}", v)),
                )
                .class("note-detail");

                // Layer info – displayed as two separate labels composed together
                Label::new(
                    cx,
                    AppData::current_layer.map(|layer: &u8| format!("layer {}", layer)),
                )
                .class("note-detail");

                Label::new(
                    cx,
                    AppData::total_layers.map(|total: &u8| format!("/ {}", total)),
                )
                .class("note-detail");
            })
            .class("note-display");
        }
    });
}
