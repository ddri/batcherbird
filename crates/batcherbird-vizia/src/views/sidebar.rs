use crate::app_data::AppData;
use crate::app_event::AppEvent;
use vizia::prelude::*;

fn section_label(cx: &mut Context, text: &str) {
    Label::new(cx, text)
        .font_size(10.0)
        .color(Color::from("#555555"));
}

fn device_row_midi(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Label::new(cx, "MIDI Out")
            .font_size(10.0)
            .color(Color::from("#666666"));
        HStack::new(cx, |cx| {
            Label::new(cx, "<")
                .font_size(10.0)
                .color(Color::from("#555555"))
                .width(Pixels(14.0))
                .alignment(Alignment::Center)
                .cursor(CursorIcon::Hand)
                .on_press(|cx| cx.emit(AppEvent::CyclePrevMidiDevice));
            Binding::new(cx, AppData::selected_midi_device, |cx, sel| {
                let sel = sel.get(cx);
                Binding::new(cx, AppData::midi_devices, move |cx, devs| {
                    let devs = devs.get(cx);
                    let name = match sel {
                        Some(i) if i < devs.len() => devs[i].clone(),
                        _ if !devs.is_empty() => devs[0].clone(),
                        _ => "No devices".to_string(),
                    };
                    Label::new(cx, &name)
                        .font_size(12.0)
                        .color(Color::from("#cccccc"))
                        .width(Stretch(1.0));
                });
            });
            Label::new(cx, ">")
                .font_size(10.0)
                .color(Color::from("#555555"))
                .width(Pixels(14.0))
                .alignment(Alignment::Center)
                .cursor(CursorIcon::Hand)
                .on_press(|cx| cx.emit(AppEvent::CycleNextMidiDevice));
            Binding::new(cx, AppData::midi_connected, |cx, c| {
                let connected = c.get(cx);
                Element::new(cx)
                    .width(Pixels(6.0))
                    .height(Pixels(6.0))
                    .corner_radius(Percentage(50.0))
                    .background_color(if connected {
                        Color::from("#28c840")
                    } else {
                        Color::from("#444444")
                    });
            });
        })
        .height(Auto)
        .horizontal_gap(Pixels(4.0))
        .alignment(Alignment::Left);
    })
    .height(Auto)
    .vertical_gap(Pixels(1.0));
}

fn device_row_audio(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Label::new(cx, "Audio In")
            .font_size(10.0)
            .color(Color::from("#666666"));
        HStack::new(cx, |cx| {
            Label::new(cx, "<")
                .font_size(10.0)
                .color(Color::from("#555555"))
                .width(Pixels(14.0))
                .alignment(Alignment::Center)
                .cursor(CursorIcon::Hand)
                .on_press(|cx| cx.emit(AppEvent::CyclePrevAudioInput));
            Binding::new(cx, AppData::selected_audio_input, |cx, sel| {
                let sel = sel.get(cx);
                Binding::new(cx, AppData::audio_input_devices, move |cx, devs| {
                    let devs = devs.get(cx);
                    let name = match sel {
                        Some(i) if i < devs.len() => devs[i].clone(),
                        _ if !devs.is_empty() => devs[0].clone(),
                        _ => "No devices".to_string(),
                    };
                    Label::new(cx, &name)
                        .font_size(12.0)
                        .color(Color::from("#cccccc"))
                        .width(Stretch(1.0));
                });
            });
            Label::new(cx, ">")
                .font_size(10.0)
                .color(Color::from("#555555"))
                .width(Pixels(14.0))
                .alignment(Alignment::Center)
                .cursor(CursorIcon::Hand)
                .on_press(|cx| cx.emit(AppEvent::CycleNextAudioInput));
            Binding::new(cx, AppData::audio_connected, |cx, c| {
                let connected = c.get(cx);
                Element::new(cx)
                    .width(Pixels(6.0))
                    .height(Pixels(6.0))
                    .corner_radius(Percentage(50.0))
                    .background_color(if connected {
                        Color::from("#28c840")
                    } else {
                        Color::from("#444444")
                    });
            });
        })
        .height(Auto)
        .horizontal_gap(Pixels(4.0))
        .alignment(Alignment::Left);
    })
    .height(Auto)
    .vertical_gap(Pixels(1.0));
}

fn field_pair(
    cx: &mut Context,
    label_a: &str,
    value_a: impl FnOnce(&mut Context),
    dec_a: AppEvent,
    inc_a: AppEvent,
    label_b: &str,
    value_b: impl FnOnce(&mut Context),
    dec_b: AppEvent,
    inc_b: AppEvent,
) {
    HStack::new(cx, |cx| {
        compact_field(cx, label_a, value_a, dec_a, inc_a);
        compact_field(cx, label_b, value_b, dec_b, inc_b);
    })
    .width(Stretch(1.0))
    .height(Auto)
    .horizontal_gap(Pixels(6.0));
}

fn compact_field(
    cx: &mut Context,
    label: &str,
    value: impl FnOnce(&mut Context),
    dec: AppEvent,
    inc: AppEvent,
) {
    VStack::new(cx, |cx| {
        Label::new(cx, label)
            .font_size(9.0)
            .color(Color::from("#555555"));
        HStack::new(cx, |cx| {
            // Minus button
            Label::new(cx, "-")
                .font_size(12.0)
                .color(Color::from("#666666"))
                .width(Pixels(18.0))
                .height(Pixels(18.0))
                .alignment(Alignment::Center)
                .background_color(Color::from("#1a1a25"))
                .corner_radius(Pixels(2.0))
                .cursor(CursorIcon::Hand)
                .on_press(move |cx| cx.emit(dec.clone()));
            // Value
            value(cx);
            // Plus button
            Label::new(cx, "+")
                .font_size(12.0)
                .color(Color::from("#666666"))
                .width(Pixels(18.0))
                .height(Pixels(18.0))
                .alignment(Alignment::Center)
                .background_color(Color::from("#1a1a25"))
                .corner_radius(Pixels(2.0))
                .cursor(CursorIcon::Hand)
                .on_press(move |cx| cx.emit(inc.clone()));
        })
        .height(Auto)
        .horizontal_gap(Pixels(3.0))
        .alignment(Alignment::Center);
    })
    .width(Stretch(1.0))
    .height(Auto)
    .background_color(Color::from("#131318"))
    .border_width(Pixels(1.0))
    .border_color(Color::from("#1e1e28"))
    .corner_radius(Pixels(3.0))
    .padding_left(Pixels(6.0))
    .padding_right(Pixels(6.0))
    .padding_top(Pixels(5.0))
    .padding_bottom(Pixels(5.0))
    .vertical_gap(Pixels(2.0));
}

fn info_field(cx: &mut Context, label: &str, content: impl FnOnce(&mut Context)) {
    VStack::new(cx, |cx| {
        Label::new(cx, label)
            .font_size(9.0)
            .color(Color::from("#555555"));
        content(cx);
    })
    .width(Stretch(1.0))
    .height(Auto)
    .background_color(Color::from("#131318"))
    .border_width(Pixels(1.0))
    .border_color(Color::from("#1e1e28"))
    .corner_radius(Pixels(3.0))
    .padding_left(Pixels(8.0))
    .padding_right(Pixels(8.0))
    .padding_top(Pixels(5.0))
    .padding_bottom(Pixels(5.0))
    .vertical_gap(Pixels(2.0));
}

fn divider(cx: &mut Context) {
    Element::new(cx)
        .width(Stretch(1.0))
        .height(Pixels(1.0))
        .background_color(Color::from("#1a1a25"));
}

pub fn sidebar(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // ---- DEVICES ----
        VStack::new(cx, |cx| {
            section_label(cx, "DEVICES");
            device_row_midi(cx);
            device_row_audio(cx);
        })
        .width(Stretch(1.0))
        .height(Auto)
        .padding(Pixels(12.0))
        .vertical_gap(Pixels(8.0));

        divider(cx);

        // ---- SAMPLING ----
        VStack::new(cx, |cx| {
            section_label(cx, "SAMPLING");

            field_pair(
                cx,
                "START",
                |cx| {
                    Label::new(cx, AppData::start_note.map(|n: &u8| AppData::note_name(*n)))
                        .font_size(14.0)
                        .color(Color::from("#dddddd"))
                        .width(Stretch(1.0))
                        .alignment(Alignment::Center);
                },
                AppEvent::DecrementStartNote,
                AppEvent::IncrementStartNote,
                "END",
                |cx| {
                    Label::new(cx, AppData::end_note.map(|n: &u8| AppData::note_name(*n)))
                        .font_size(14.0)
                        .color(Color::from("#dddddd"))
                        .width(Stretch(1.0))
                        .alignment(Alignment::Center);
                },
                AppEvent::DecrementEndNote,
                AppEvent::IncrementEndNote,
            );

            field_pair(
                cx,
                "LAYERS",
                |cx| {
                    Label::new(cx, AppData::velocity_layers.map(|n: &u8| n.to_string()))
                        .font_size(14.0)
                        .color(Color::from("#dddddd"))
                        .width(Stretch(1.0))
                        .alignment(Alignment::Center);
                },
                AppEvent::DecrementVelocityLayers,
                AppEvent::IncrementVelocityLayers,
                "DURATION",
                |cx| {
                    Label::new(
                        cx,
                        AppData::note_duration_ms
                            .map(|ms: &u32| format!("{:.1}s", *ms as f32 / 1000.0)),
                    )
                    .font_size(14.0)
                    .color(Color::from("#dddddd"))
                    .width(Stretch(1.0))
                    .alignment(Alignment::Center);
                },
                AppEvent::DecrementDuration,
                AppEvent::IncrementDuration,
            );
        })
        .width(Stretch(1.0))
        .height(Auto)
        .padding(Pixels(12.0))
        .vertical_gap(Pixels(8.0));

        divider(cx);

        // ---- EXPORT ----
        VStack::new(cx, |cx| {
            section_label(cx, "EXPORT");

            compact_field(
                cx,
                "FORMAT",
                |cx| {
                    Label::new(cx, AppData::export_format_display)
                        .font_size(12.0)
                        .color(Color::from("#dddddd"))
                        .width(Stretch(1.0))
                        .alignment(Alignment::Center);
                },
                AppEvent::CycleExportFormatBack,
                AppEvent::CycleExportFormat,
            );

            info_field(cx, "OUTPUT", |cx| {
                Label::new(
                    cx,
                    AppData::output_directory.map(|p: &std::path::PathBuf| {
                        p.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| p.to_string_lossy().to_string())
                    }),
                )
                .font_size(12.0)
                .color(Color::from("#cccccc"))
                .cursor(CursorIcon::Hand)
                .on_press(|cx| cx.emit(AppEvent::SelectOutputDirectory));
            });
        })
        .width(Stretch(1.0))
        .height(Auto)
        .padding(Pixels(12.0))
        .vertical_gap(Pixels(8.0));

        // Push export button to bottom
        Element::new(cx).height(Stretch(1.0));

        // Export button
        VStack::new(cx, |cx| {
            Label::new(cx, "Export All")
                .font_size(12.0)
                .color(Color::from("#666666"))
                .width(Stretch(1.0))
                .alignment(Alignment::Center)
                .cursor(CursorIcon::Hand)
                .on_press(|cx| cx.emit(AppEvent::ExportAll));
        })
        .width(Stretch(1.0))
        .height(Pixels(32.0))
        .background_color(Color::from("#131318"))
        .border_width(Pixels(1.0))
        .border_color(Color::from("#1e1e28"))
        .corner_radius(Pixels(4.0))
        .alignment(Alignment::Center)
        .left(Pixels(12.0))
        .right(Pixels(12.0))
        .bottom(Pixels(12.0));
    })
    .width(Pixels(210.0))
    .height(Stretch(1.0))
    .background_color(Color::from("#0c0c12"));
}
