use crate::app_data::AppData;
use crate::app_event::{AppEvent, InstrumentPreset};
use vizia::prelude::*;

fn card_header(cx: &mut Context, title: &str) {
    HStack::new(cx, move |cx| {
        Element::new(cx).class("card-indicator");
        Label::new(cx, title).class("card-title");
    })
    .class("card-header");
}

fn compact_stepper(
    cx: &mut Context,
    label: &str,
    value: impl FnOnce(&mut Context),
    dec: AppEvent,
    inc: AppEvent,
) {
    VStack::new(cx, |cx| {
        Label::new(cx, label).class("field-label");
        HStack::new(cx, |cx| {
            Label::new(cx, "-")
                .class("stepper-btn")
                .on_press(move |cx| cx.emit(dec.clone()));
            value(cx);
            Label::new(cx, "+")
                .class("stepper-btn")
                .on_press(move |cx| cx.emit(inc.clone()));
        })
        .height(Auto)
        .horizontal_gap(Pixels(4.0))
        .alignment(Alignment::Center);
    })
    .class("field-box");
}

#[allow(clippy::too_many_arguments)]
fn stepper_pair(
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
        compact_stepper(cx, label_a, value_a, dec_a, inc_a);
        compact_stepper(cx, label_b, value_b, dec_b, inc_b);
    })
    .width(Stretch(1.0))
    .height(Auto)
    .horizontal_gap(Pixels(6.0));
}

pub fn sidebar(cx: &mut Context) {
    VStack::new(cx, |cx| {
            // ==========================================
            // CARD 1: I/O & ROUTING
            // ==========================================
            VStack::new(cx, |cx| {
                card_header(cx, "I/O & ROUTING");

                // MIDI Output
                VStack::new(cx, |cx| {
                    Label::new(cx, "MIDI DESTINATION").class("field-label");
                    PickList::new(cx, AppData::midi_devices, AppData::selected_midi_device, true)
                        .on_select(|cx, idx| cx.emit(AppEvent::SelectMidiDevice(idx)))
                        .width(Stretch(1.0));
                })
                .height(Auto)
                .vertical_gap(Pixels(3.0));

                // Audio Input
                VStack::new(cx, |cx| {
                    Label::new(cx, "AUDIO SOURCE").class("field-label");
                    PickList::new(cx, AppData::audio_input_devices, AppData::selected_audio_input, true)
                        .on_select(|cx, idx| cx.emit(AppEvent::SelectAudioInput(idx)))
                        .width(Stretch(1.0));
                })
                .height(Auto)
                .vertical_gap(Pixels(3.0));

                // Channel Routing
                VStack::new(cx, |cx| {
                    Label::new(cx, "CHANNEL TOPOLOGY").class("field-label");
                    PickList::new(cx, AppData::channel_routing_options, AppData::selected_channel_routing, true)
                        .on_select(|cx, idx| cx.emit(AppEvent::SelectChannelRouting(idx)))
                        .width(Stretch(1.0));
                })
                .height(Auto)
                .vertical_gap(Pixels(3.0));

                // Playthrough Monitor Toggle
                HStack::new(cx, |cx| {
                    Label::new(cx, "MONITOR PLAYTHROUGH").class("field-label").width(Stretch(1.0));

                    Binding::new(cx, AppData::playthrough_enabled, |cx, enabled| {
                        let is_on = enabled.get(cx);
                        HStack::new(cx, move |cx| {
                            Label::new(cx, if is_on { "ACTIVE" } else { "MUTED" })
                                .font_size(9.0)
                                .font_weight(FontWeightKeyword::Bold)
                                .color(if is_on {
                                    Color::from("#10b981")
                                } else {
                                    Color::from("#64748b")
                                });
                        })
                        .padding_left(Pixels(8.0))
                        .padding_right(Pixels(8.0))
                        .padding_top(Pixels(3.0))
                        .padding_bottom(Pixels(3.0))
                        .background_color(if is_on {
                            Color::from("#062817")
                        } else {
                            Color::from("#161a27")
                        })
                        .border_width(Pixels(1.0))
                        .border_color(if is_on {
                            Color::from("#10b98166")
                        } else {
                            Color::from("#242c3f")
                        })
                        .corner_radius(Pixels(4.0))
                        .cursor(CursorIcon::Hand)
                        .on_press(|cx| cx.emit(AppEvent::TogglePlaythrough));
                    });
                })
                .height(Auto)
                .alignment(Alignment::Center);

                // Input Gain Trim
                VStack::new(cx, |cx| {
                    HStack::new(cx, |cx| {
                        Label::new(cx, "PREAMP GAIN TRIM").class("field-label").width(Stretch(1.0));

                        Binding::new(cx, AppData::input_gain_display, |cx, display| {
                            let text = display.get(cx);
                            Label::new(cx, &text)
                                .font_size(10.0)
                                .font_weight(FontWeightKeyword::Bold)
                                .color(Color::from("#f59e0b"))
                                .cursor(CursorIcon::Hand)
                                .on_press(|cx| cx.emit(AppEvent::ResetInputGain));
                        });
                    })
                    .height(Auto)
                    .alignment(Alignment::Center);

                    HStack::new(cx, |cx| {
                        Label::new(cx, "-1 dB")
                            .class("chip-btn")
                            .width(Stretch(1.0))
                            .on_press(|cx| cx.emit(AppEvent::AdjustInputGain(-1.0)));

                        Label::new(cx, "0 dB")
                            .class("chip-btn")
                            .width(Stretch(1.0))
                            .on_press(|cx| cx.emit(AppEvent::ResetInputGain));

                        Label::new(cx, "+1 dB")
                            .class("chip-btn")
                            .width(Stretch(1.0))
                            .on_press(|cx| cx.emit(AppEvent::AdjustInputGain(1.0)));
                    })
                    .class("chip-group");
                })
                .height(Auto)
                .vertical_gap(Pixels(4.0));
            })
            .class("card");

            // ==========================================
            // CARD 2: SAMPLER ENGINE
            // ==========================================
            VStack::new(cx, |cx| {
                card_header(cx, "SAMPLER ENGINE");

                // Octave Range Presets
                VStack::new(cx, |cx| {
                    Label::new(cx, "OCTAVE SPAN").class("field-label");
                    HStack::new(cx, |cx| {
                        for (label, octaves) in [("1 OCT", 1), ("2 OCT", 2), ("4 OCT", 4)] {
                            Label::new(cx, label)
                                .class("chip-btn")
                                .width(Stretch(1.0))
                                .on_press(move |cx| cx.emit(AppEvent::SetOctavePreset(octaves)));
                        }
                    })
                    .class("chip-group");
                })
                .height(Auto)
                .vertical_gap(Pixels(3.0));

                // Instrument Style Presets
                VStack::new(cx, |cx| {
                    Label::new(cx, "INSTRUMENT PROFILE").class("field-label");
                    HStack::new(cx, |cx| {
                        for (label, preset) in [
                            ("LEAD", InstrumentPreset::Lead),
                            ("PAD", InstrumentPreset::Pad),
                            ("BASS", InstrumentPreset::Bass),
                            ("PLUCK", InstrumentPreset::Pluck),
                        ] {
                            Label::new(cx, label)
                                .class("chip-btn")
                                .width(Stretch(1.0))
                                .on_press(move |cx| cx.emit(AppEvent::ApplyInstrumentPreset(preset)));
                        }
                    })
                    .class("chip-group");
                })
                .height(Auto)
                .vertical_gap(Pixels(3.0));

                // Note Range Steppers
                stepper_pair(
                    cx,
                    "START NOTE",
                    |cx| {
                        Label::new(cx, AppData::start_note.map(|n: &u8| AppData::note_name(*n)))
                            .class("field-value")
                            .width(Stretch(1.0))
                            .alignment(Alignment::Center);
                    },
                    AppEvent::DecrementStartNote,
                    AppEvent::IncrementStartNote,
                    "END NOTE",
                    |cx| {
                        Label::new(cx, AppData::end_note.map(|n: &u8| AppData::note_name(*n)))
                            .class("field-value")
                            .width(Stretch(1.0))
                            .alignment(Alignment::Center);
                    },
                    AppEvent::DecrementEndNote,
                    AppEvent::IncrementEndNote,
                );

                // Step Interval Picklist
                VStack::new(cx, |cx| {
                    Label::new(cx, "STEPPING INTERVAL").class("field-label");
                    PickList::new(
                        cx,
                        AppData::note_step_options,
                        AppData::selected_step_index,
                        true,
                    )
                    .on_select(|cx, idx| cx.emit(AppEvent::SelectNoteStepByIndex(idx)))
                    .width(Stretch(1.0));
                })
                .height(Auto)
                .vertical_gap(Pixels(3.0));

                // Layers & Duration Steppers
                stepper_pair(
                    cx,
                    "VEL LAYERS",
                    |cx| {
                        Label::new(cx, AppData::velocity_layers.map(|n: &u8| n.to_string()))
                            .class("field-value")
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
                        .class("field-value")
                        .width(Stretch(1.0))
                        .alignment(Alignment::Center);
                    },
                    AppEvent::DecrementDuration,
                    AppEvent::IncrementDuration,
                );

                // Live session telemetry badge
                HStack::new(cx, |cx| {
                    Label::new(cx, AppData::session_summary_display)
                        .font_size(10.0)
                        .font_weight(FontWeightKeyword::Bold)
                        .color(Color::from("#38bdf8"))
                        .alignment(Alignment::Center);
                })
                .width(Stretch(1.0))
                .height(Pixels(26.0))
                .background_color(Color::from("#0c2033"))
                .corner_radius(Pixels(4.0))
                .border_width(Pixels(1.0))
                .border_color(Color::from("#0284c755"))
                .alignment(Alignment::Center);
            })
            .class("card");

            // ==========================================
            // CARD 3: EXPORT PIPELINE
            // ==========================================
            VStack::new(cx, |cx| {
                card_header(cx, "EXPORT PIPELINE");

                // Target Format
                VStack::new(cx, |cx| {
                    Label::new(cx, "EXPORT FORMAT").class("field-label");
                    PickList::new(cx, AppData::format_options, AppData::selected_format_index, true)
                        .on_select(|cx, idx| cx.emit(AppEvent::SelectFormatByIndex(idx)))
                        .width(Stretch(1.0));
                })
                .height(Auto)
                .vertical_gap(Pixels(3.0));

                // Destination Folder
                VStack::new(cx, |cx| {
                    Label::new(cx, "DESTINATION DIRECTORY").class("field-label");
                    HStack::new(cx, |cx| {
                        Label::new(
                            cx,
                            AppData::output_directory.map(|p: &std::path::PathBuf| {
                                p.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| p.to_string_lossy().to_string())
                            }),
                        )
                        .font_size(11.0)
                        .color(Color::from("#cbd5e1"))
                        .width(Stretch(1.0));

                        Label::new(cx, "Browse")
                            .font_size(10.0)
                            .color(Color::from("#60a5fa"));
                    })
                    .height(Auto)
                    .alignment(Alignment::Center);
                })
                .class("field-box")
                .cursor(CursorIcon::Hand)
                .on_press(|cx| cx.emit(AppEvent::SelectOutputDirectory));

                // Export All Button
                Label::new(cx, "EXPORT ALL FORMATS")
                    .class("btn-secondary")
                    .width(Stretch(1.0))
                    .on_press(|cx| cx.emit(AppEvent::ExportAll));
            })
            .class("card");
    })
    .class("sidebar");
}
