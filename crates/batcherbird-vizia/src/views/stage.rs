use super::{keyboard, meters, note_display, progress_bar, WaveformView};
use crate::app_data::{AppData, AppState};
use crate::app_event::AppEvent;
use vizia::prelude::*;

pub fn stage(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // --- Error banner ---
        Binding::new(cx, AppData::error_message, |cx, msg| {
            let msg = msg.get(cx);
            if let Some(text) = msg {
                HStack::new(cx, |cx| {
                    Label::new(cx, &text)
                        .color(Color::from("#e53935"))
                        .width(Stretch(1.0));
                    Button::new(cx, |cx| Label::new(cx, "×"))
                        .on_press(|cx| cx.emit(AppEvent::DismissError))
                        .width(Pixels(24.0))
                        .height(Pixels(24.0));
                })
                .width(Stretch(1.0))
                .height(Auto)
                .background_color(Color::from("#2a1015"))
                .border_width(Pixels(1.0))
                .border_color(Color::from("#e53935"))
                .corner_radius(Pixels(6.0))
                .padding(Pixels(8.0))
                .horizontal_gap(Pixels(8.0));
            }
        });

        // --- Meters (always visible) ---
        meters(cx);

        // --- REC indicator (only when Recording) ---
        HStack::new(cx, |cx| {
            Element::new(cx)
                .width(Pixels(10.0))
                .height(Pixels(10.0))
                .corner_radius(Percentage(50.0))
                .background_color(Color::from("#e53935"));
            Label::new(cx, "REC")
                .color(Color::from("#e53935"))
                .font_size(12.0);
        })
        .height(Auto)
        .horizontal_gap(Pixels(6.0))
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Recording { Display::Flex } else { Display::None }
        }));

        // --- Note display (only when Recording) ---
        note_display(cx);

        // --- Keyboard (always visible) ---
        keyboard(cx);

        // --- Waveform (always visible) ---
        WaveformView::new(cx)
            .width(Stretch(1.0))
            .height(Pixels(100.0));

        // --- Progress bar (only when Recording) ---
        progress_bar(cx);

        // --- Cancel button (only when Recording) ---
        Button::new(cx, |cx| Label::new(cx, "Cancel"))
            .class("btn-cancel")
            .height(Pixels(32.0))
            .on_press(|cx| cx.emit(AppEvent::CancelRecording))
            .display(AppData::app_state.map(|s| {
                if *s == AppState::Recording { Display::Flex } else { Display::None }
            }));

        // Spacer
        Element::new(cx).height(Stretch(1.0));

        // --- Idle content ---
        VStack::new(cx, |cx| {
            Label::new(cx, "Ready to record")
                .color(Color::from("#444444"))
                .font_size(14.0)
                .width(Stretch(1.0))
                .alignment(Alignment::Center);

            Binding::new(cx, AppData::start_note, |cx, _| {
                Binding::new(cx, AppData::end_note, |cx, _| {
                    Binding::new(cx, AppData::velocity_layers, |cx, _| {
                        let start = AppData::start_note.get(cx);
                        let end = AppData::end_note.get(cx);
                        let layers = AppData::velocity_layers.get(cx);
                        let num_notes = (end as u32).saturating_sub(start as u32) + 1;
                        let total = num_notes * layers as u32;
                        Label::new(cx, &format!("{} notes · {} layers · {} total", num_notes, layers, total))
                            .color(Color::from("#333333"))
                            .font_size(12.0)
                            .width(Stretch(1.0))
                            .alignment(Alignment::Center);
                    });
                });
            });

            Button::new(cx, |cx| Label::new(cx, "ARM"))
                .class("btn-arm")
                .height(Pixels(44.0))
                .on_press(|cx| cx.emit(AppEvent::Arm));
        })
        .width(Stretch(1.0))
        .height(Auto)
        .vertical_gap(Pixels(8.0))
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Idle { Display::Flex } else { Display::None }
        }));

        // --- Armed content ---
        VStack::new(cx, |cx| {
            Label::new(cx, "Monitoring — press Record")
                .color(Color::from("#888888"))
                .font_size(14.0);

            Button::new(cx, |cx| Label::new(cx, "RECORD"))
                .class("btn-record")
                .height(Pixels(44.0))
                .on_press(|cx| cx.emit(AppEvent::StartRecording));

            Button::new(cx, |cx| Label::new(cx, "Cancel"))
                .class("btn-cancel")
                .height(Pixels(32.0))
                .on_press(|cx| cx.emit(AppEvent::Disarm));
        })
        .width(Stretch(1.0))
        .height(Auto)
        .vertical_gap(Pixels(12.0))
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Armed { Display::Flex } else { Display::None }
        }));

        // --- Review content ---
        VStack::new(cx, |cx| {
            Label::new(cx, "Recording Complete")
                .color(Color::white())
                .font_size(20.0);

            Binding::new(cx, AppData::recorded_count, |cx, count| {
                let count = count.get(cx);
                Label::new(cx, &format!("{} samples recorded", count))
                    .color(Color::from("#888888"))
                    .font_size(13.0);
            });

            HStack::new(cx, |cx| {
                Binding::new(cx, AppData::is_playing, |cx, playing| {
                    if playing.get(cx) {
                        Button::new(cx, |cx| Label::new(cx, "Pause"))
                            .class("btn-play")
                            .on_press(|cx| cx.emit(AppEvent::PausePreview));
                    } else {
                        Button::new(cx, |cx| Label::new(cx, "Play"))
                            .class("btn-play")
                            .on_press(|cx| cx.emit(AppEvent::PlayPreview));
                    }
                });
                Button::new(cx, |cx| Label::new(cx, "Stop"))
                    .class("btn-stop")
                    .on_press(|cx| cx.emit(AppEvent::StopPreview));
            })
            .height(Auto)
            .horizontal_gap(Pixels(8.0));

            HStack::new(cx, |cx| {
                Button::new(cx, |cx| Label::new(cx, "Export All"))
                    .class("btn-export")
                    .width(Stretch(1.0))
                    .on_press(|cx| cx.emit(AppEvent::ExportAll));
                Button::new(cx, |cx| Label::new(cx, "New Session"))
                    .class("btn-cancel")
                    .width(Stretch(1.0))
                    .on_press(|cx| cx.emit(AppEvent::Disarm));
            })
            .height(Auto)
            .horizontal_gap(Pixels(8.0));
        })
        .width(Stretch(1.0))
        .height(Auto)
        .vertical_gap(Pixels(12.0))
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Review { Display::Flex } else { Display::None }
        }));
    })
    .width(Stretch(1.0))
    .height(Stretch(1.0))
    .background_color(Color::from("#111118"))
    .padding(Pixels(20.0))
    .vertical_gap(Pixels(12.0));
}
