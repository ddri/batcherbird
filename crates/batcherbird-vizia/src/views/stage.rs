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
                        .font_size(12.0)
                        .width(Stretch(1.0));
                    Label::new(cx, "×")
                        .color(Color::from("#888888"))
                        .font_size(14.0)
                        .width(Pixels(20.0))
                        .alignment(Alignment::Center)
                        .cursor(CursorIcon::Hand)
                        .on_press(|cx| cx.emit(AppEvent::DismissError));
                })
                .width(Stretch(1.0))
                .height(Auto)
                .background_color(Color::from("#1a0a0a"))
                .border_width(Pixels(1.0))
                .border_color(Color::from("#3a1520"))
                .corner_radius(Pixels(4.0))
                .padding(Pixels(8.0))
                .horizontal_gap(Pixels(8.0));
            }
        });

        // --- Info / success banner ---
        Binding::new(cx, AppData::info_message, |cx, msg| {
            let msg = msg.get(cx);
            if let Some(text) = msg {
                HStack::new(cx, |cx| {
                    Label::new(cx, &text)
                        .color(Color::from("#28c840"))
                        .font_size(12.0)
                        .width(Stretch(1.0));
                    Label::new(cx, "×")
                        .color(Color::from("#888888"))
                        .font_size(14.0)
                        .width(Pixels(20.0))
                        .alignment(Alignment::Center)
                        .cursor(CursorIcon::Hand)
                        .on_press(|cx| cx.emit(AppEvent::DismissError));
                })
                .width(Stretch(1.0))
                .height(Auto)
                .background_color(Color::from("#0a1a0a"))
                .border_width(Pixels(1.0))
                .border_color(Color::from("#15301a"))
                .corner_radius(Pixels(4.0))
                .padding(Pixels(8.0))
                .horizontal_gap(Pixels(8.0));
            }
        });

        // --- Meters ---
        meters(cx);

        // --- REC indicator (Recording only) ---
        HStack::new(cx, |cx| {
            Element::new(cx)
                .width(Pixels(8.0))
                .height(Pixels(8.0))
                .corner_radius(Percentage(50.0))
                .background_color(Color::from("#e53935"));
            Label::new(cx, "REC")
                .color(Color::from("#e53935"))
                .font_size(11.0);
        })
        .height(Auto)
        .horizontal_gap(Pixels(6.0))
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Recording { Display::Flex } else { Display::None }
        }));

        // --- Note display (Recording only) ---
        note_display(cx);

        // --- Keyboard ---
        keyboard(cx);

        // --- Waveform ---
        WaveformView::new(cx)
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .min_height(Pixels(100.0))
            .corner_radius(Pixels(3.0));

        // --- Progress bar (Recording only) ---
        progress_bar(cx);

        // --- Cancel (Recording only) ---
        Label::new(cx, "Cancel")
            .font_size(12.0)
            .color(Color::from("#888888"))
            .width(Stretch(1.0))
            .height(Pixels(28.0))
            .alignment(Alignment::Center)
            .background_color(Color::from("#131318"))
            .border_width(Pixels(1.0))
            .border_color(Color::from("#1e1e28"))
            .corner_radius(Pixels(4.0))
            .cursor(CursorIcon::Hand)
            .on_press(|cx| cx.emit(AppEvent::CancelRecording))
            .display(AppData::app_state.map(|s| {
                if *s == AppState::Recording { Display::Flex } else { Display::None }
            }));

        // --- Idle content ---
        VStack::new(cx, |cx| {
            Label::new(cx, "Ready to record")
                .color(Color::from("#555555"))
                .font_size(13.0)
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
                            .font_size(11.0)
                            .width(Stretch(1.0))
                            .alignment(Alignment::Center);
                    });
                });
            });

            // ARM button — dark bg, subtle red border
            Label::new(cx, "ARM")
                .font_size(16.0)
                .color(Color::from("#e53935"))
                .width(Stretch(1.0))
                .height(Pixels(42.0))
                .alignment(Alignment::Center)
                .background_color(Color::from("#141418"))
                .border_width(Pixels(2.0))
                .border_color(Color::from("#e5393544"))
                .corner_radius(Pixels(6.0))
                .cursor(CursorIcon::Hand)
                .on_press(|cx| cx.emit(AppEvent::Arm));
        })
        .width(Stretch(1.0))
        .height(Auto)
        .vertical_gap(Pixels(6.0))
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Idle { Display::Flex } else { Display::None }
        }));

        // --- Armed content ---
        VStack::new(cx, |cx| {
            Label::new(cx, "Monitoring — press Record")
                .color(Color::from("#888888"))
                .font_size(13.0)
                .width(Stretch(1.0))
                .alignment(Alignment::Center);

            // RECORD button — red bg
            Label::new(cx, "RECORD")
                .font_size(16.0)
                .color(Color::white())
                .width(Stretch(1.0))
                .height(Pixels(42.0))
                .alignment(Alignment::Center)
                .background_color(Color::from("#e53935"))
                .corner_radius(Pixels(6.0))
                .cursor(CursorIcon::Hand)
                .on_press(|cx| cx.emit(AppEvent::StartRecording));

            Label::new(cx, "Cancel")
                .font_size(12.0)
                .color(Color::from("#666666"))
                .width(Stretch(1.0))
                .height(Pixels(28.0))
                .alignment(Alignment::Center)
                .background_color(Color::from("#131318"))
                .border_width(Pixels(1.0))
                .border_color(Color::from("#1e1e28"))
                .corner_radius(Pixels(4.0))
                .cursor(CursorIcon::Hand)
                .on_press(|cx| cx.emit(AppEvent::Disarm));
        })
        .width(Stretch(1.0))
        .height(Auto)
        .vertical_gap(Pixels(8.0))
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Armed { Display::Flex } else { Display::None }
        }));

        // --- Review content ---
        VStack::new(cx, |cx| {
            Label::new(cx, "Recording Complete")
                .color(Color::white())
                .font_size(18.0);

            Binding::new(cx, AppData::recorded_count, |cx, count| {
                let count = count.get(cx);
                Label::new(cx, &format!("{} samples recorded", count))
                    .color(Color::from("#888888"))
                    .font_size(12.0);
            });

            HStack::new(cx, |cx| {
                Binding::new(cx, AppData::is_playing, |cx, playing| {
                    let label = if playing.get(cx) { "Pause" } else { "Play" };
                    let event = if playing.get(cx) { AppEvent::PausePreview } else { AppEvent::PlayPreview };
                    Label::new(cx, label)
                        .font_size(12.0)
                        .color(Color::white())
                        .width(Pixels(60.0))
                        .height(Pixels(28.0))
                        .alignment(Alignment::Center)
                        .background_color(Color::from("#28c840"))
                        .corner_radius(Pixels(4.0))
                        .cursor(CursorIcon::Hand)
                        .on_press(move |cx| cx.emit(event.clone()));
                });
                Label::new(cx, "Stop")
                    .font_size(12.0)
                    .color(Color::from("#cccccc"))
                    .width(Pixels(60.0))
                    .height(Pixels(28.0))
                    .alignment(Alignment::Center)
                    .background_color(Color::from("#2a2a2a"))
                    .corner_radius(Pixels(4.0))
                    .cursor(CursorIcon::Hand)
                    .on_press(|cx| cx.emit(AppEvent::StopPreview));
            })
            .height(Auto)
            .horizontal_gap(Pixels(6.0));

            HStack::new(cx, |cx| {
                Label::new(cx, "Export All")
                    .font_size(12.0)
                    .color(Color::from("#4a9eff"))
                    .width(Stretch(1.0))
                    .height(Pixels(30.0))
                    .alignment(Alignment::Center)
                    .background_color(Color::from("#131318"))
                    .border_width(Pixels(1.0))
                    .border_color(Color::from("#4a9eff"))
                    .corner_radius(Pixels(4.0))
                    .cursor(CursorIcon::Hand)
                    .on_press(|cx| cx.emit(AppEvent::ExportAll));
                Label::new(cx, "New Session")
                    .font_size(12.0)
                    .color(Color::from("#888888"))
                    .width(Stretch(1.0))
                    .height(Pixels(30.0))
                    .alignment(Alignment::Center)
                    .background_color(Color::from("#131318"))
                    .border_width(Pixels(1.0))
                    .border_color(Color::from("#1e1e28"))
                    .corner_radius(Pixels(4.0))
                    .cursor(CursorIcon::Hand)
                    .on_press(|cx| cx.emit(AppEvent::Disarm));
            })
            .height(Auto)
            .horizontal_gap(Pixels(6.0));
        })
        .width(Stretch(1.0))
        .height(Auto)
        .vertical_gap(Pixels(10.0))
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Review { Display::Flex } else { Display::None }
        }));
    })
    .width(Stretch(1.0))
    .height(Stretch(1.0))
    .background_color(Color::from("#111118"))
    .padding(Pixels(20.0))
    .vertical_gap(Pixels(10.0));
}
