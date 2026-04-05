use crate::app_data::AppData;
use crate::app_event::AppEvent;
use vizia::prelude::*;

fn increment_button(cx: &mut Context, label: &str, event: AppEvent) {
    Button::new(cx, |cx| Label::new(cx, label))
        .width(Pixels(22.0))
        .height(Pixels(22.0))
        .class("btn-sm")
        .on_press(move |cx| cx.emit(event.clone()));
}

fn field_box(cx: &mut Context, label_text: &str, content: impl FnOnce(&mut Context)) {
    VStack::new(cx, |cx| {
        Label::new(cx, label_text).class("field-label");
        content(cx);
    })
    .width(Stretch(1.0))
    .height(Auto)
    .background_color(Color::from("#161620"))
    .border_width(Pixels(1.0))
    .border_color(Color::from("#1e1e2a"))
    .corner_radius(Pixels(4.0))
    .padding(Pixels(8.0))
    .vertical_gap(Pixels(3.0));
}

pub fn sidebar(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // ---- DEVICES ----
        VStack::new(cx, |cx| {
            Label::new(cx, "DEVICES").class("sidebar-label");

            // MIDI Out
            VStack::new(cx, |cx| {
                Label::new(cx, "MIDI Out").class("device-type");
                HStack::new(cx, |cx| {
                    Binding::new(cx, AppData::selected_midi_device, |cx, sel| {
                        let sel = sel.get(cx);
                        Binding::new(cx, AppData::midi_devices, move |cx, devs| {
                            let devs = devs.get(cx);
                            let name = match sel {
                                Some(i) if i < devs.len() => devs[i].clone(),
                                _ if !devs.is_empty() => devs[0].clone(),
                                _ => "No MIDI devices".to_string(),
                            };
                            Label::new(cx, &name)
                                .class("device-name")
                                .cursor(CursorIcon::Hand)
                                .on_press(|cx| cx.emit(AppEvent::CycleNextMidiDevice));
                        });
                    });
                    Binding::new(cx, AppData::midi_connected, |cx, connected| {
                        let connected = connected.get(cx);
                        let dot = Element::new(cx)
                            .width(Pixels(7.0))
                            .height(Pixels(7.0))
                            .corner_radius(Percentage(50.0))
                            .background_color(if connected {
                                Color::from("#28c840")
                            } else {
                                Color::from("#555555")
                            });
                    });
                })
                .height(Auto)
                .horizontal_gap(Pixels(6.0));
            })
            .height(Auto)
            .vertical_gap(Pixels(2.0));

            // Audio In
            VStack::new(cx, |cx| {
                Label::new(cx, "Audio In").class("device-type");
                HStack::new(cx, |cx| {
                    Binding::new(cx, AppData::selected_audio_input, |cx, sel| {
                        let sel = sel.get(cx);
                        Binding::new(cx, AppData::audio_input_devices, move |cx, devs| {
                            let devs = devs.get(cx);
                            let name = match sel {
                                Some(i) if i < devs.len() => devs[i].clone(),
                                _ if !devs.is_empty() => devs[0].clone(),
                                _ => "No audio devices".to_string(),
                            };
                            Label::new(cx, &name)
                                .class("device-name")
                                .cursor(CursorIcon::Hand)
                                .on_press(|cx| cx.emit(AppEvent::CycleNextAudioInput));
                        });
                    });
                    Binding::new(cx, AppData::audio_connected, |cx, connected| {
                        let connected = connected.get(cx);
                        let dot = Element::new(cx)
                            .width(Pixels(7.0))
                            .height(Pixels(7.0))
                            .corner_radius(Percentage(50.0))
                            .background_color(if connected {
                                Color::from("#28c840")
                            } else {
                                Color::from("#555555")
                            });
                    });
                })
                .height(Auto)
                .horizontal_gap(Pixels(6.0));
            })
            .height(Auto)
            .vertical_gap(Pixels(2.0));
        })
        .width(Stretch(1.0))
        .height(Auto)
        .padding(Pixels(14.0))
        .vertical_gap(Pixels(10.0));

        Element::new(cx)
            .width(Stretch(1.0))
            .height(Pixels(1.0))
            .background_color(Color::from("#222230"));

        // ---- SAMPLING ----
        VStack::new(cx, |cx| {
            Label::new(cx, "SAMPLING").class("sidebar-label");

            // START / END row
            HStack::new(cx, |cx| {
                field_box(cx, "START", |cx| {
                    HStack::new(cx, |cx| {
                        increment_button(cx, "-", AppEvent::DecrementStartNote);
                        Label::new(cx, AppData::start_note.map(|n: &u8| AppData::note_name(*n)))
                            .class("field-value")
                            .width(Stretch(1.0))
                            .alignment(Alignment::Center);
                        increment_button(cx, "+", AppEvent::IncrementStartNote);
                    })
                    .height(Auto)
                    .horizontal_gap(Pixels(4.0))
                    .alignment(Alignment::Center);
                });

                field_box(cx, "END", |cx| {
                    HStack::new(cx, |cx| {
                        increment_button(cx, "-", AppEvent::DecrementEndNote);
                        Label::new(cx, AppData::end_note.map(|n: &u8| AppData::note_name(*n)))
                            .class("field-value")
                            .width(Stretch(1.0))
                            .alignment(Alignment::Center);
                        increment_button(cx, "+", AppEvent::IncrementEndNote);
                    })
                    .height(Auto)
                    .horizontal_gap(Pixels(4.0))
                    .alignment(Alignment::Center);
                });
            })
            .width(Stretch(1.0))
            .height(Auto)
            .horizontal_gap(Pixels(8.0));

            // LAYERS / DURATION row
            HStack::new(cx, |cx| {
                field_box(cx, "LAYERS", |cx| {
                    HStack::new(cx, |cx| {
                        increment_button(cx, "-", AppEvent::DecrementVelocityLayers);
                        Label::new(cx, AppData::velocity_layers.map(|n: &u8| n.to_string()))
                            .class("field-value")
                            .width(Stretch(1.0))
                            .alignment(Alignment::Center);
                        increment_button(cx, "+", AppEvent::IncrementVelocityLayers);
                    })
                    .height(Auto)
                    .horizontal_gap(Pixels(4.0))
                    .alignment(Alignment::Center);
                });

                field_box(cx, "DURATION", |cx| {
                    HStack::new(cx, |cx| {
                        increment_button(cx, "-", AppEvent::DecrementDuration);
                        Label::new(
                            cx,
                            AppData::note_duration_ms
                                .map(|ms: &u32| format!("{:.1}s", *ms as f32 / 1000.0)),
                        )
                        .class("field-value")
                        .width(Stretch(1.0))
                        .alignment(Alignment::Center);
                        increment_button(cx, "+", AppEvent::IncrementDuration);
                    })
                    .height(Auto)
                    .horizontal_gap(Pixels(4.0))
                    .alignment(Alignment::Center);
                });
            })
            .width(Stretch(1.0))
            .height(Auto)
            .horizontal_gap(Pixels(8.0));
        })
        .width(Stretch(1.0))
        .height(Auto)
        .padding(Pixels(14.0))
        .vertical_gap(Pixels(10.0));

        Element::new(cx)
            .width(Stretch(1.0))
            .height(Pixels(1.0))
            .background_color(Color::from("#222230"));

        // ---- EXPORT ----
        VStack::new(cx, |cx| {
            Label::new(cx, "EXPORT").class("sidebar-label");

            field_box(cx, "FORMAT", |cx| {
                Label::new(cx, AppData::export_format_display)
                    .class("field-value")
                    .cursor(CursorIcon::Hand)
                    .on_press(|cx| cx.emit(AppEvent::CycleExportFormat));
            });

            field_box(cx, "OUTPUT", |cx| {
                Label::new(
                    cx,
                    AppData::output_directory.map(|p: &std::path::PathBuf| {
                        p.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| p.to_string_lossy().to_string())
                    }),
                )
                .class("field-value")
                .cursor(CursorIcon::Hand)
                .on_press(|cx| cx.emit(AppEvent::SelectOutputDirectory));
            });
        })
        .width(Stretch(1.0))
        .height(Auto)
        .padding(Pixels(14.0))
        .vertical_gap(Pixels(10.0));

        // Spacer to push export button to bottom
        Element::new(cx).height(Stretch(1.0));

        // Export button
        Button::new(cx, |cx| Label::new(cx, "Export All"))
            .class("btn-export")
            .height(Pixels(32.0))
            .left(Pixels(14.0))
            .right(Pixels(14.0))
            .bottom(Pixels(14.0))
            .on_press(|cx| cx.emit(AppEvent::ExportAll));
    })
    .width(Pixels(220.0))
    .height(Stretch(1.0))
    .background_color(Color::from("#0e0e15"))
    .border_width(Pixels(1.0))
    .border_color(Color::from("#1e1e28"));
}
