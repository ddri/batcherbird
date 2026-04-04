use crate::app_data::AppData;
use crate::app_event::AppEvent;
use vizia::prelude::*;

/// Small +/- button helper
fn increment_button(cx: &mut Context, label: &str, event: AppEvent) {
    Button::new(cx, |cx| Label::new(cx, label))
        .class("btn-sm")
        .on_press(move |cx| cx.emit(event.clone()));
}

pub fn sidebar(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // DEVICES section
        VStack::new(cx, |cx| {
            Label::new(cx, "DEVICES").class("sidebar-label");

            // MIDI device row
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
                            .class("signal-dot")
                            .size(Pixels(7.0))
                            .corner_radius(Percentage(50.0));
                        if !connected {
                            dot.class("disconnected");
                        }
                    });
                });
            })
            .class("device-row");

            // Audio input row
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
                            .class("signal-dot")
                            .size(Pixels(7.0))
                            .corner_radius(Percentage(50.0));
                        if !connected {
                            dot.class("disconnected");
                        }
                    });
                });
            })
            .class("device-row");
        })
        .class("sidebar-section");

        Element::new(cx).class("divider");

        // SAMPLING section
        VStack::new(cx, |cx| {
            Label::new(cx, "SAMPLING").class("sidebar-label");

            HStack::new(cx, |cx| {
                // START note
                VStack::new(cx, |cx| {
                    Label::new(cx, "START").class("field-label");
                    HStack::new(cx, |cx| {
                        increment_button(cx, "-", AppEvent::DecrementStartNote);
                        Label::new(cx, AppData::start_note.map(|n: &u8| AppData::note_name(*n)))
                            .class("field-value");
                        increment_button(cx, "+", AppEvent::IncrementStartNote);
                    })
                    .class("field-controls");
                })
                .class("field-box field-box-interactive");

                // END note
                VStack::new(cx, |cx| {
                    Label::new(cx, "END").class("field-label");
                    HStack::new(cx, |cx| {
                        increment_button(cx, "-", AppEvent::DecrementEndNote);
                        Label::new(cx, AppData::end_note.map(|n: &u8| AppData::note_name(*n)))
                            .class("field-value");
                        increment_button(cx, "+", AppEvent::IncrementEndNote);
                    })
                    .class("field-controls");
                })
                .class("field-box field-box-interactive");
            })
            .gap(Pixels(8.0));

            HStack::new(cx, |cx| {
                // LAYERS
                VStack::new(cx, |cx| {
                    Label::new(cx, "LAYERS").class("field-label");
                    HStack::new(cx, |cx| {
                        increment_button(cx, "-", AppEvent::DecrementVelocityLayers);
                        Label::new(cx, AppData::velocity_layers.map(|n: &u8| n.to_string()))
                            .class("field-value");
                        increment_button(cx, "+", AppEvent::IncrementVelocityLayers);
                    })
                    .class("field-controls");
                })
                .class("field-box field-box-interactive");

                // DURATION
                VStack::new(cx, |cx| {
                    Label::new(cx, "DURATION").class("field-label");
                    HStack::new(cx, |cx| {
                        increment_button(cx, "-", AppEvent::DecrementDuration);
                        Label::new(
                            cx,
                            AppData::note_duration_ms
                                .map(|ms: &u32| format!("{:.1}s", *ms as f32 / 1000.0)),
                        )
                        .class("field-value");
                        increment_button(cx, "+", AppEvent::IncrementDuration);
                    })
                    .class("field-controls");
                })
                .class("field-box field-box-interactive");
            })
            .gap(Pixels(8.0));
        })
        .class("sidebar-section");

        Element::new(cx).class("divider");

        // EXPORT section
        VStack::new(cx, |cx| {
            Label::new(cx, "EXPORT").class("sidebar-label");

            // FORMAT — click to cycle
            VStack::new(cx, |cx| {
                Label::new(cx, "FORMAT").class("field-label");
                Label::new(cx, AppData::export_format_display)
                    .class("field-value")
                    .cursor(CursorIcon::Hand)
                    .on_press(|cx| cx.emit(AppEvent::CycleExportFormat));
            })
            .class("field-box field-box-interactive")
            .cursor(CursorIcon::Hand)
            .on_press(|cx| cx.emit(AppEvent::CycleExportFormat));

            // OUTPUT directory
            VStack::new(cx, |cx| {
                Label::new(cx, "OUTPUT").class("field-label");
                Label::new(
                    cx,
                    AppData::output_directory.map(|p: &std::path::PathBuf| {
                        p.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| p.to_string_lossy().to_string())
                    }),
                )
                .class("field-value");
            })
            .class("field-box")
            .cursor(CursorIcon::Hand)
            .on_press(|cx| cx.emit(AppEvent::SelectOutputDirectory));
        })
        .class("sidebar-section");

        Button::new(cx, |cx| Label::new(cx, "Export All"))
            .class("btn-export")
            .on_press(|cx| cx.emit(AppEvent::ExportAll));
    })
    .class("sidebar");
}
