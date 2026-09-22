use super::{keyboard, meters, note_display, progress_bar, WaveformView};
use crate::app_data::{AppData, AppState};
use crate::app_event::AppEvent;
use vizia::prelude::*;

pub fn stage(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // --- Error Banner ---
        Binding::new(cx, AppData::error_message, |cx, msg| {
            let msg = msg.get(cx);
            if let Some(text) = msg {
                HStack::new(cx, |cx| {
                    Label::new(cx, &text)
                        .color(Color::from("#ef4444"))
                        .font_size(12.0)
                        .font_weight(FontWeightKeyword::Bold)
                        .width(Stretch(1.0));
                    Label::new(cx, "×")
                        .color(Color::from("#94a3b8"))
                        .font_size(16.0)
                        .width(Pixels(24.0))
                        .alignment(Alignment::Center)
                        .cursor(CursorIcon::Hand)
                        .on_press(|cx| cx.emit(AppEvent::DismissError));
                })
                .class("banner-error");
            }
        });

        // --- Info / Success Banner ---
        Binding::new(cx, AppData::info_message, |cx, msg| {
            let msg = msg.get(cx);
            if let Some(text) = msg {
                HStack::new(cx, |cx| {
                    Label::new(cx, &text)
                        .color(Color::from("#10b981"))
                        .font_size(12.0)
                        .font_weight(FontWeightKeyword::Bold)
                        .width(Stretch(1.0));
                    Label::new(cx, "×")
                        .color(Color::from("#94a3b8"))
                        .font_size(16.0)
                        .width(Pixels(24.0))
                        .alignment(Alignment::Center)
                        .cursor(CursorIcon::Hand)
                        .on_press(|cx| cx.emit(AppEvent::DismissError));
                })
                .class("banner-info");
            }
        });

        // --- Hardware Meters Deck ---
        meters(cx);

        // --- Recording State Indicator Bar ---
        HStack::new(cx, |cx| {
            Element::new(cx)
                .width(Pixels(10.0))
                .height(Pixels(10.0))
                .corner_radius(Percentage(50.0))
                .background_color(Color::from("#ef4444"));
            Label::new(cx, "RECORDING IN PROGRESS")
                .color(Color::from("#ef4444"))
                .font_size(11.0)
                .font_weight(FontWeightKeyword::Bold);
        })
        .height(Auto)
        .horizontal_gap(Pixels(8.0))
        .alignment(Alignment::Center)
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Recording { Display::Flex } else { Display::None }
        }));

        // --- Note Display (Recording only) ---
        note_display(cx);

        // --- Virtual Keyboard ---
        keyboard(cx);

        // --- Waveform Oscilloscope Display ---
        WaveformView::new(cx)
            .width(Stretch(1.0))
            .height(Stretch(1.0))
            .min_height(Pixels(120.0))
            .corner_radius(Pixels(6.0));

        // --- Progress Bar (Recording only) ---
        progress_bar(cx);

        // --- Cancel Action (Recording only) ---
        Label::new(cx, "ABORT RECORDING")
            .class("btn-secondary")
            .width(Stretch(1.0))
            .height(Pixels(32.0))
            .on_press(|cx| cx.emit(AppEvent::CancelRecording))
            .display(AppData::app_state.map(|s| {
                if *s == AppState::Recording { Display::Flex } else { Display::None }
            }));

        // --- Idle State Controls ---
        VStack::new(cx, |cx| {
            Binding::new(cx, AppData::start_note, |cx, _| {
                Binding::new(cx, AppData::end_note, |cx, _| {
                    Binding::new(cx, AppData::velocity_layers, |cx, _| {
                        let start = AppData::start_note.get(cx);
                        let end = AppData::end_note.get(cx);
                        let layers = AppData::velocity_layers.get(cx);
                        let num_notes = (end as u32).saturating_sub(start as u32) + 1;
                        let total = num_notes * layers as u32;
                        Label::new(cx, &format!("READY  ·  {} NOTES  ·  {} VELOCITY LAYERS  ·  {} TOTAL SAMPLES", num_notes, layers, total))
                            .color(Color::from("#64748b"))
                            .font_size(11.0)
                            .font_weight(FontWeightKeyword::Bold)
                            .width(Stretch(1.0))
                            .alignment(Alignment::Center);
                    });
                });
            });

            // Prominent Tactile ARM Button
            Label::new(cx, "ARM AUTO-SAMPLER")
                .class("btn-primary-arm")
                .on_press(|cx| cx.emit(AppEvent::Arm));
        })
        .width(Stretch(1.0))
        .height(Auto)
        .vertical_gap(Pixels(8.0))
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Idle { Display::Flex } else { Display::None }
        }));

        // --- Armed State Deck ---
        VStack::new(cx, |cx| {
            Label::new(cx, "ARMED  ·  PRE-FLIGHT SIGNAL MONITORING ACTIVE")
                .color(Color::from("#94a3b8"))
                .font_size(11.0)
                .font_weight(FontWeightKeyword::Bold)
                .width(Stretch(1.0))
                .alignment(Alignment::Center);

            // Quick toggles in Armed view: Playthrough & Channel Routing
            HStack::new(cx, |cx| {
                // Playthrough toggle pill
                Binding::new(cx, AppData::playthrough_enabled, |cx, enabled| {
                    let is_on = enabled.get(cx);
                    HStack::new(cx, move |cx| {
                        Label::new(
                            cx,
                            if is_on {
                                "🔊 Playthrough: ON"
                            } else {
                                "🔈 Playthrough: OFF"
                            },
                        )
                        .font_size(11.0)
                        .font_weight(FontWeightKeyword::Bold)
                        .color(if is_on {
                            Color::from("#10b981")
                        } else {
                            Color::from("#64748b")
                        });
                    })
                    .class("btn-pill")
                    .on_press(|cx| cx.emit(AppEvent::TogglePlaythrough));
                });

                // Channel Routing toggle pill
                Binding::new(cx, AppData::channel_routing_display, |cx, display| {
                    let text = display.get(cx);
                    HStack::new(cx, move |cx| {
                        Label::new(cx, &format!("🎛 Routing: {}", text))
                            .font_size(11.0)
                            .font_weight(FontWeightKeyword::Bold)
                            .color(Color::from("#38bdf8"));
                    })
                    .class("btn-pill")
                    .on_press(|cx| cx.emit(AppEvent::CycleChannelRouting));
                });

                // Gain Trim pill
                Binding::new(cx, AppData::input_gain_display, |cx, display| {
                    let text = display.get(cx);
                    HStack::new(cx, move |cx| {
                        Label::new(cx, &format!("🎚 Preamp: {}", text))
                            .font_size(11.0)
                            .font_weight(FontWeightKeyword::Bold)
                            .color(Color::from("#f59e0b"));
                    })
                    .class("btn-pill")
                    .on_press(|cx| cx.emit(AppEvent::ResetInputGain));
                });
            })
            .alignment(Alignment::Center)
            .horizontal_gap(Pixels(8.0))
            .height(Auto);

            // Optional gain staging feedback banner
            Binding::new(cx, AppData::gain_check_message, |cx, msg| {
                if let Some(text) = msg.get(cx) {
                    let color_hex = AppData::gain_check_status_color.get(cx);
                    let text_color = color_hex.clone();
                    HStack::new(cx, move |cx| {
                        Label::new(cx, &text)
                            .color(Color::from(text_color.as_str()))
                            .font_size(12.0)
                            .font_weight(FontWeightKeyword::Bold)
                            .width(Stretch(1.0))
                            .alignment(Alignment::Center);
                    })
                    .width(Stretch(1.0))
                    .height(Auto)
                    .background_color(Color::from("#101524"))
                    .border_width(Pixels(1.0))
                    .border_color(Color::from(color_hex.as_str()))
                    .corner_radius(Pixels(6.0))
                    .padding(Pixels(8.0));
                }
            });

            // Action row: Test Note button + Record button
            HStack::new(cx, |cx| {
                Binding::new(cx, AppData::is_testing_note, |cx, testing| {
                    let is_testing = testing.get(cx);
                    let label = if is_testing {
                        "TESTING NOTE..."
                    } else {
                        "TEST NOTE (VEL 127)"
                    };
                    Label::new(cx, label)
                        .font_size(13.0)
                        .font_weight(FontWeightKeyword::Bold)
                        .color(if is_testing { Color::from("#64748b") } else { Color::from("#38bdf8") })
                        .width(Stretch(1.0))
                        .height(Pixels(46.0))
                        .alignment(Alignment::Center)
                        .background_color(Color::from("#121927"))
                        .border_width(Pixels(1.5))
                        .border_color(if is_testing { Color::from("#1e293b") } else { Color::from("#0284c7") })
                        .corner_radius(Pixels(8.0))
                        .cursor(if is_testing { CursorIcon::Default } else { CursorIcon::Hand })
                        .on_press(move |cx| {
                            if !is_testing {
                                cx.emit(AppEvent::PlayTestNote);
                            }
                        });
                });

                // Prominent RECORD button
                Label::new(cx, "START RECORDING")
                    .class("btn-primary-record")
                    .on_press(|cx| cx.emit(AppEvent::StartRecording));
            })
            .width(Stretch(1.0))
            .height(Auto)
            .horizontal_gap(Pixels(10.0));

            Label::new(cx, "DISARM")
                .class("btn-secondary")
                .width(Stretch(1.0))
                .height(Pixels(30.0))
                .on_press(|cx| cx.emit(AppEvent::Disarm));
        })
        .width(Stretch(1.0))
        .height(Auto)
        .vertical_gap(Pixels(8.0))
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Armed { Display::Flex } else { Display::None }
        }));

        // --- Review State Deck ---
        VStack::new(cx, |cx| {
            Label::new(cx, "SESSION RECORDING COMPLETE")
                .color(Color::white())
                .font_size(16.0)
                .font_weight(FontWeightKeyword::Bold);

            Binding::new(cx, AppData::recorded_count, |cx, count| {
                let count = count.get(cx);
                Label::new(cx, &format!("{} multisamples captured & verified", count))
                    .color(Color::from("#94a3b8"))
                    .font_size(12.0);
            });

            HStack::new(cx, |cx| {
                Binding::new(cx, AppData::is_playing, |cx, playing| {
                    let label = if playing.get(cx) { "PAUSE PREVIEW" } else { "PLAY PREVIEW" };
                    let event = if playing.get(cx) { AppEvent::PausePreview } else { AppEvent::PlayPreview };
                    Label::new(cx, label)
                        .font_size(12.0)
                        .font_weight(FontWeightKeyword::Bold)
                        .color(Color::white())
                        .width(Pixels(130.0))
                        .height(Pixels(34.0))
                        .alignment(Alignment::Center)
                        .background_color(Color::from("#10b981"))
                        .corner_radius(Pixels(6.0))
                        .cursor(CursorIcon::Hand)
                        .on_press(move |cx| cx.emit(event.clone()));
                });
                Label::new(cx, "STOP")
                    .font_size(12.0)
                    .font_weight(FontWeightKeyword::Bold)
                    .color(Color::from("#cbd5e1"))
                    .width(Pixels(80.0))
                    .height(Pixels(34.0))
                    .alignment(Alignment::Center)
                    .background_color(Color::from("#242c3f"))
                    .corner_radius(Pixels(6.0))
                    .cursor(CursorIcon::Hand)
                    .on_press(|cx| cx.emit(AppEvent::StopPreview));
            })
            .height(Auto)
            .horizontal_gap(Pixels(8.0));

            HStack::new(cx, |cx| {
                Label::new(cx, "EXPORT ALL FORMATS")
                    .font_size(13.0)
                    .font_weight(FontWeightKeyword::Bold)
                    .color(Color::from("#38bdf8"))
                    .width(Stretch(1.0))
                    .height(Pixels(36.0))
                    .alignment(Alignment::Center)
                    .background_color(Color::from("#0c2033"))
                    .border_width(Pixels(1.0))
                    .border_color(Color::from("#0284c7"))
                    .corner_radius(Pixels(6.0))
                    .cursor(CursorIcon::Hand)
                    .on_press(|cx| cx.emit(AppEvent::ExportAll));

                Label::new(cx, "NEW SESSION")
                    .class("btn-secondary")
                    .width(Stretch(1.0))
                    .height(Pixels(36.0))
                    .on_press(|cx| cx.emit(AppEvent::Disarm));
            })
            .height(Auto)
            .horizontal_gap(Pixels(8.0));
        })
        .width(Stretch(1.0))
        .height(Auto)
        .vertical_gap(Pixels(10.0))
        .display(AppData::app_state.map(|s| {
            if *s == AppState::Review { Display::Flex } else { Display::None }
        }));
    })
    .class("stage");
}
