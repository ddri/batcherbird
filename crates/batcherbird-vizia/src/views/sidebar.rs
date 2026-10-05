use super::disclosure::disclosure;
use crate::app_data::{AppData, AppState};
use crate::app_event::{AppEvent, InstrumentPreset};
use vizia::prelude::*;

pub(super) fn action<'a>(cx: &'a mut Context, label: &str, event: AppEvent) -> Handle<'a, Button> {
    Button::new(cx, |cx| Label::new(cx, label))
        .name(label.to_owned())
        .text_value(label.to_owned())
        .on_press(move |cx| cx.emit(event.clone()))
}

fn stepper(
    cx: &mut Context,
    title: &str,
    value: impl FnOnce(&mut Context),
    dec: AppEvent,
    inc: AppEvent,
) {
    VStack::new(cx, |cx| {
        Label::new(cx, title).class("field-label");
        HStack::new(cx, |cx| {
            action(cx, "−", dec)
                .name(format!("Decrease {}", title))
                .text_value(format!("Decrease {}", title))
                .class("stepper-btn");
            value(cx);
            action(cx, "+", inc)
                .name(format!("Increase {}", title))
                .text_value(format!("Increase {}", title))
                .class("stepper-btn");
        })
        .class("stepper");
    })
    .class("field")
    .width(Stretch(1.0));
}

pub fn sidebar(cx: &mut Context) {
    ScrollView::new(cx, |cx| {
        VStack::new(cx, |cx| {
            disclosure(cx, "01  Connections", AppData::app_state.map(|s| matches!(s, AppState::Idle | AppState::Armed)), |cx| {
                Label::new(cx, "MIDI output").class("field-label");
                PickList::new(
                    cx,
                    AppData::midi_devices,
                    AppData::selected_midi_device,
                    true,
                )
                .name("MIDI output device").text_value("MIDI output device")
                .on_select(|cx, idx| cx.emit(AppEvent::SelectMidiDevice(idx)))
                .width(Stretch(1.0));
                Label::new(cx, "Audio input").class("field-label");
                PickList::new(
                    cx,
                    AppData::audio_input_devices,
                    AppData::selected_audio_input,
                    true,
                )
                .name("Audio input device").text_value("Audio input device")
                .on_select(|cx, idx| cx.emit(AppEvent::SelectAudioInput(idx)))
                .width(Stretch(1.0));
                action(cx, "Refresh devices", AppEvent::RefreshDevices).width(Stretch(1.0));
                Button::new(cx, |cx| {
                    Label::new(
                        cx,
                        AppData::playthrough_enabled.map(|on| {
                            if *on {
                                "Monitoring on"
                            } else {
                                "Monitoring off"
                            }
                        }),
                    )
                })
                .name(AppData::playthrough_enabled.map(|on| {
                    if *on {
                        "Monitoring on"
                    } else {
                        "Monitoring off"
                    }
                })).text_value(AppData::playthrough_enabled.map(|on| {
                    if *on {
                        "Monitoring on"
                    } else {
                        "Monitoring off"
                    }
                }))
                .on_press(|cx| cx.emit(AppEvent::TogglePlaythrough))
                .width(Stretch(1.0));
                disclosure(cx, "Routing and gain", false, |cx| {
                Label::new(cx, "Input channels").class("field-label");
                PickList::new(
                    cx,
                    AppData::channel_routing_options,
                    AppData::selected_channel_routing,
                    true,
                )
                .name("Input channels")
                .text_value(AppData::channel_routing_display)
                .on_select(|cx, idx| cx.emit(AppEvent::SelectChannelRouting(idx)))
                .width(Stretch(1.0));
                HStack::new(cx, |cx| {
                    Label::new(cx, "Digital gain")
                        .class("field-label")
                        .width(Stretch(1.0));
                    Label::new(cx, AppData::input_gain_display)
                        .name("Digital gain")
                        .text_value(AppData::input_gain_display)
                        .class("field-value");
                })
                .height(Auto)
                .alignment(Alignment::Center);
                HStack::new(cx, |cx| {
                    action(cx, "−1 dB", AppEvent::AdjustInputGain(-1.0)).width(Stretch(1.0));
                    action(cx, "Reset", AppEvent::ResetInputGain).width(Stretch(1.0));
                    action(cx, "+1 dB", AppEvent::AdjustInputGain(1.0)).width(Stretch(1.0));
                })
                .class("control-row");
                }).class("advanced-disclosure");
            })
            .class("settings-group");

            disclosure(cx, "02  Capture plan", AppData::app_state.map(|s| *s != AppState::Review), |cx| {
                Label::new(cx, "Starting point").class("field-label");
                HStack::new(cx, |cx| {
                    for (label, preset) in [
                        ("Lead", InstrumentPreset::Lead),
                        ("Pad", InstrumentPreset::Pad),
                        ("Bass", InstrumentPreset::Bass),
                        ("Pluck", InstrumentPreset::Pluck),
                    ] {
                        action(cx, label, AppEvent::ApplyInstrumentPreset(preset))
                            .width(Stretch(1.0));
                    }
                })
                .class("control-row");
                HStack::new(cx, |cx| {
                    stepper(
                        cx,
                        "First note",
                        |cx| {
                            Label::new(cx, AppData::start_note.map(|n| AppData::note_name(*n)))
                                .class("field-value")
                                .width(Stretch(1.0));
                        },
                        AppEvent::DecrementStartNote,
                        AppEvent::IncrementStartNote,
                    );
                    stepper(
                        cx,
                        "Last note",
                        |cx| {
                            Label::new(cx, AppData::end_note.map(|n| AppData::note_name(*n)))
                                .class("field-value")
                                .width(Stretch(1.0));
                        },
                        AppEvent::DecrementEndNote,
                        AppEvent::IncrementEndNote,
                    );
                })
                .class("control-row");
                Label::new(cx, "Note spacing").class("field-label");
                PickList::new(
                    cx,
                    AppData::note_step_options,
                    AppData::selected_step_index,
                    true,
                )
                .name("Note spacing").text_value("Note spacing")
                .on_select(|cx, idx| cx.emit(AppEvent::SelectNoteStepByIndex(idx)))
                .width(Stretch(1.0));
                HStack::new(cx, |cx| {
                    stepper(
                        cx,
                        "Velocity layers",
                        |cx| {
                            Label::new(cx, AppData::velocity_layers.map(|n| n.to_string()))
                                .class("field-value")
                                .width(Stretch(1.0));
                        },
                        AppEvent::DecrementVelocityLayers,
                        AppEvent::IncrementVelocityLayers,
                    );
                    stepper(
                        cx,
                        "Hold",
                        |cx| {
                            Label::new(
                                cx,
                                AppData::note_duration_ms
                                    .map(|ms| format!("{:.1} s", *ms as f32 / 1000.0)),
                            )
                            .class("field-value")
                            .width(Stretch(1.0));
                        },
                        AppEvent::DecrementDuration,
                        AppEvent::IncrementDuration,
                    );
                })
                .class("control-row");
                Label::new(cx, "Release tail").class("field-label");
                HStack::new(cx, |cx| {
                    Button::new(cx, |cx| Label::new(cx, "−"))
                        .name("Decrease release tail").text_value("Decrease release tail")
                        .on_press(|cx| {
                            let ms = AppData::release_duration_ms.get(cx);
                            cx.emit(AppEvent::SetReleaseDuration(ms.saturating_sub(100)));
                        })
                        .class("stepper-btn");
                    Label::new(
                        cx,
                        AppData::release_duration_ms
                            .map(|ms| format!("{:.1} s", *ms as f32 / 1000.0)),
                    )
                    .class("field-value")
                    .width(Stretch(1.0));
                    Button::new(cx, |cx| Label::new(cx, "+"))
                        .name("Increase release tail").text_value("Increase release tail")
                        .on_press(|cx| {
                            let ms = AppData::release_duration_ms.get(cx);
                            cx.emit(AppEvent::SetReleaseDuration(ms.saturating_add(100)));
                        })
                        .class("stepper-btn");
                })
                .class("stepper");
                Label::new(cx, AppData::session_summary_display).class("plan-summary");
            })
            .class("settings-group");

            disclosure(cx, "03  Export instrument", AppData::app_state.map(|s| *s == AppState::Review), |cx| {
                Label::new(cx, "Instrument name").class("field-label");
                Textbox::new(cx, AppData::instrument_name)
                    .name("Instrument name")
                    .on_edit(|cx, text| cx.emit(AppEvent::SetInstrumentName(text)))
                    .width(Stretch(1.0));
                Label::new(cx, "Format").class("field-label");
                PickList::new(
                    cx,
                    AppData::format_options,
                    AppData::selected_format_index,
                    true,
                )
                .name("Export format")
                .text_value(AppData::export_format_display)
                .on_select(|cx, idx| cx.emit(AppEvent::SelectFormatByIndex(idx)))
                .width(Stretch(1.0));
                Button::new(cx, |cx| Label::new(cx, "Choose export folder…"))
                    .name("Choose export folder").text_value("Choose export folder")
                    .on_press(|cx| cx.emit(AppEvent::SelectOutputDirectory))
                    .width(Stretch(1.0));
                Label::new(
                    cx,
                    AppData::output_directory.map(|p| p.to_string_lossy().to_string()),
                )
                .class("muted")
                .width(Stretch(1.0));
                disclosure(cx, "Silence trimming", false, |cx| {
                    Button::new(cx, |cx| Label::new(cx, AppData::trim_silence.map(|on| if *on { "Trim silence on" } else { "Trim silence off" })))
                        .name(AppData::trim_silence.map(|on| if *on { "Trim silence enabled" } else { "Trim silence disabled" })).text_value(AppData::trim_silence.map(|on| if *on { "Trim silence enabled" } else { "Trim silence disabled" }))
                        .checked(AppData::trim_silence)
                        .on_press(|cx| { let on = AppData::trim_silence.get(cx); cx.emit(AppEvent::SetTrimSilence(!on)); })
                        .width(Stretch(1.0));
                    Label::new(cx, "Automatic trimming may remove very quiet attacks. Turn it off to keep the full take; edge fades still apply.")
                        .class("muted").width(Stretch(1.0));
                }).class("advanced-disclosure");
                disclosure(cx, "Loop options", false, |cx| {
                Button::new(cx, |cx| {
                    Label::new(
                        cx,
                        AppData::auto_loop.map(|enabled| {
                            if *enabled {
                                "Automatic loops on (experimental)"
                            } else {
                                "Automatic loops off"
                            }
                        }),
                    )
                })
                .name(AppData::auto_loop.map(|on| {
                    if *on {
                        "Automatic loops enabled, experimental"
                    } else {
                        "Automatic loops disabled"
                    }
                })).text_value(AppData::auto_loop.map(|on| {
                    if *on {
                        "Automatic loops enabled, experimental"
                    } else {
                        "Automatic loops disabled"
                    }
                }))
                .on_press(|cx| {
                    let enabled = AppData::auto_loop.get(cx);
                    cx.emit(AppEvent::SetAutoLoop(!enabled));
                })
                .width(Stretch(1.0));
                    Label::new(cx, "Automatic loop detection is experimental. Listen to the exported instrument before sharing it.").class("muted").width(Stretch(1.0));
                }).class("advanced-disclosure");
                Button::new(cx, |cx| {
                    Label::new(
                        cx,
                        AppData::export_in_progress.map(|busy| {
                            if *busy {
                                "Exporting…"
                            } else {
                                "Export selected format"
                            }
                        }),
                    )
                })
                .name(AppData::export_in_progress.map(|busy| {
                    if *busy {
                        "Exporting instrument"
                    } else {
                        "Export selected format"
                    }
                })).text_value(AppData::export_in_progress.map(|busy| {
                    if *busy {
                        "Exporting instrument"
                    } else {
                        "Export selected format"
                    }
                }))
                .on_press(|cx| cx.emit(AppEvent::ExportAll))
                .class("primary")
                .width(Stretch(1.0))
                .disabled(AppData::recorded_count.map(|n| *n == 0));
            })
            .class("settings-group");
        })
        .class("inspector-content")
        .disabled(AppData::controls_busy);
    })
    .class("sidebar")
    .show_horizontal_scrollbar(false);
}
