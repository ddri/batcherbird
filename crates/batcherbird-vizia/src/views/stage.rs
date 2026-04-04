use super::{keyboard, meters, note_display, progress_bar, WaveformView};
use crate::app_data::{AppData, AppState};
use crate::app_event::AppEvent;
use vizia::prelude::*;

pub fn stage(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // --- Error banner (shown when error_message is Some) ---
        Binding::new(cx, AppData::error_message, |cx, msg| {
            let msg = msg.get(cx);
            if let Some(text) = msg {
                HStack::new(cx, |cx| {
                    Label::new(cx, &text).color(Color::from("#e53935"));
                    Button::new(cx, |cx| Label::new(cx, "×"))
                        .on_press(|cx| cx.emit(AppEvent::DismissError));
                })
                .class("error-banner");
            }
        });

        // --- Meters (always visible) ---
        meters(cx);

        // --- REC indicator (only when Recording) ---
        HStack::new(cx, |cx| {
            Element::new(cx)
                .size(Pixels(10.0))
                .corner_radius(Percentage(50.0))
                .background_color(Color::from("#e53935"));
            Label::new(cx, "REC").class("rec-text");
        })
        .class("rec-indicator")
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Recording {
                Display::Flex
            } else {
                Display::None
            }
        }));

        // --- Note display (only when Recording) ---
        note_display(cx);

        // --- Keyboard (always visible) ---
        keyboard(cx);

        // --- Waveform (always visible, empty when idle) ---
        WaveformView::new(cx)
            .width(Stretch(1.0))
            .height(Pixels(80.0));

        // --- Progress bar (only when Recording) ---
        progress_bar(cx);

        // --- Cancel button (only when Recording) ---
        Button::new(cx, |cx| Label::new(cx, "Cancel"))
            .class("btn-cancel")
            .on_press(|cx| cx.emit(AppEvent::CancelRecording))
            .display(AppData::app_state.map(|s| {
                if *s == AppState::Recording {
                    Display::Flex
                } else {
                    Display::None
                }
            }));

        // --- Idle content (only when Idle) ---
        VStack::new(cx, |cx| {
            Label::new(cx, "Ready to record").class("idle-heading");

            // Reactive summary: "X notes · Y layers · Z total"
            Binding::new(cx, AppData::start_note, |cx, _| {
                Binding::new(cx, AppData::end_note, |cx, _| {
                    Binding::new(cx, AppData::velocity_layers, |cx, _| {
                        let start = AppData::start_note.get(cx);
                        let end = AppData::end_note.get(cx);
                        let layers = AppData::velocity_layers.get(cx);
                        let num_notes = (end as u32).saturating_sub(start as u32) + 1;
                        let total = num_notes * layers as u32;
                        let text =
                            format!("{} notes · {} layers · {} total", num_notes, layers, total);
                        Label::new(cx, &text).class("idle-summary");
                    });
                });
            });

            Button::new(cx, |cx| Label::new(cx, "ARM"))
                .class("btn-arm")
                .on_press(|cx| cx.emit(AppEvent::Arm));
        })
        .class("idle-content")
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Idle {
                Display::Flex
            } else {
                Display::None
            }
        }));

        // --- Armed content (only when Armed) ---
        VStack::new(cx, |cx| {
            Label::new(cx, "Monitoring — press Record").class("armed-heading");

            Button::new(cx, |cx| Label::new(cx, "RECORD"))
                .class("btn-record")
                .on_press(|cx| cx.emit(AppEvent::StartRecording));

            Button::new(cx, |cx| Label::new(cx, "Cancel"))
                .class("btn-cancel")
                .on_press(|cx| cx.emit(AppEvent::Disarm));
        })
        .class("armed-content")
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Armed {
                Display::Flex
            } else {
                Display::None
            }
        }));

        // --- Review content (only when Review) ---
        VStack::new(cx, |cx| {
            Label::new(cx, "Recording Complete").class("note-name");
            Label::new(cx, "Samples ready for export").class("idle-subtext");
            Button::new(cx, |cx| Label::new(cx, "New Session"))
                .class("btn-arm")
                .on_press(|cx| cx.emit(AppEvent::Disarm));
        })
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Review {
                Display::Flex
            } else {
                Display::None
            }
        }));
    })
    .class("stage");
}
