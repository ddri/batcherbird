use super::disclosure::disclosure;
use super::sidebar::action;
use super::{keyboard, meters, note_display, progress_bar, WaveformView};
use crate::app_data::{AppData, AppState};
use crate::app_event::AppEvent;
use vizia::prelude::*;

pub fn toolbar(cx: &mut Context) {
    HStack::new(cx, |cx| {
        VStack::new(cx, |cx| {
            Label::new(cx, "batcherbird").class("brand");
            Label::new(cx, "HARDWARE TO INSTRUMENT").class("brand-caption");
        })
        .height(Auto)
        .width(Pixels(252.0));
        VStack::new(cx, |cx| {
            Label::new(cx, AppData::instrument_name)
                .class("session-name")
                .width(Stretch(1.0))
                .text_wrap(false)
                .text_overflow(TextOverflow::Ellipsis);
            Label::new(cx, AppData::session_status)
                .name("Session status")
                .text_value(AppData::session_status)
                .class("muted");
        })
        .width(Stretch(1.0))
        .height(Auto);
        Label::new(
            cx,
            AppData::has_unsaved_changes
                .map(|modified| if *modified { "Modified" } else { "Saved" }),
        )
        .class("muted");
        action(cx, "Panic", AppEvent::Panic);
        action(cx, "Open…", AppEvent::OpenSession).disabled(AppData::controls_busy);
        action(cx, "Save session", AppEvent::SaveSession).disabled(AppData::controls_busy);
    })
    .class("toolbar");
}

pub fn stage(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Binding::new(cx, AppData::error_message, |cx, msg| {
            if let Some(text) = msg.get(cx) {
                HStack::new(cx, |cx| {
                    Label::new(cx, &text).width(Stretch(1.0));
                    action(cx, "Dismiss", AppEvent::DismissError);
                })
                .class("banner-error");
            }
        });
        Binding::new(cx, AppData::info_message, |cx, msg| {
            if let Some(text) = msg.get(cx) {
                HStack::new(cx, |cx| {
                    Label::new(cx, &text).width(Stretch(1.0));
                    action(cx, "Dismiss", AppEvent::DismissError);
                })
                .class("banner-info");
            }
        });
        HStack::new(cx, |cx| {
            VStack::new(cx, |cx| {
                Label::new(
                    cx,
                    AppData::app_state.map(|state| match state {
                        AppState::Review => "Review your samples",
                        AppState::Recording | AppState::Stopping => "Capture in progress",
                        _ => "Build your instrument",
                    }),
                )
                .class("workspace-title");
                Label::new(
                    cx,
                    AppData::app_state.map(|s| match s {
                        AppState::Idle => "Connect your synth and build a capture plan.",
                        AppState::Armed => "Audition a note and check your input levels.",
                        AppState::Recording => "Capturing your instrument…",
                        AppState::Stopping => "Stopping capture and keeping completed samples…",
                        AppState::Review => "Select a sample to listen, inspect, or record again.",
                    }),
                )
                .class("muted");
            })
            .height(Auto)
            .width(Stretch(1.0));
            Label::new(
                cx,
                AppData::app_state.map(|s| match s {
                    AppState::Idle => "SETUP",
                    AppState::Armed => "SIGNAL CHECK",
                    AppState::Recording => "RECORDING",
                    AppState::Stopping => "STOPPING",
                    AppState::Review => "REVIEW",
                }),
            )
            .class("status-tag");
        })
        .height(Auto)
        .alignment(Alignment::Center);
        VStack::new(cx, meters)
            .width(Stretch(1.0))
            .height(Auto)
            .display(AppData::app_state.map(|s| {
                if *s == AppState::Review {
                    Display::None
                } else {
                    Display::Flex
                }
            }));

        Binding::new(cx, AppData::app_state, |cx, state| match state.get(cx) {
            AppState::Idle | AppState::Armed => {
                VStack::new(cx, |cx| {
                    Label::new(cx, "Capture a sound. Keep it forever.").class("empty-title");
                    Label::new(
                        cx,
                        "01   Connect MIDI out to your synth, then its audio to your interface.",
                    )
                    .class("guide-line");
                    Label::new(
                        cx,
                        "02   Choose the notes, velocity layers, hold, and release tail.",
                    )
                    .class("guide-line");
                    Label::new(
                        cx,
                        "03   Check the signal, record, and export your instrument.",
                    )
                    .class("guide-line");
                    Label::new(cx, AppData::session_summary_display).class("plan-summary");
                })
                .class("empty-workspace");
            }
            AppState::Recording | AppState::Stopping => {
                VStack::new(cx, |cx| {
                    note_display(cx);
                    WaveformView::new(cx)
                        .width(Stretch(1.0))
                        .height(Stretch(1.0))
                        .min_height(Pixels(100.0));
                    progress_bar(cx);
                })
                .width(Stretch(1.0))
                .height(Stretch(1.0))
                .vertical_gap(Pixels(16.0));
            }
            AppState::Review => {
                HStack::new(cx, |cx| {
                    VStack::new(cx, |cx| {
                        Label::new(
                            cx,
                            AppData::recorded_count.map(|n| format!("Samples  ·  {}", n)),
                        )
                        .class("section-title");
                        List::new(cx, AppData::sample_labels, |cx, index, item| {
                            Button::new(cx, |cx| {
                                Label::new(cx, item)
                                    .width(Stretch(1.0))
                                    .text_wrap(false)
                                    .text_overflow(TextOverflow::Ellipsis)
                            })
                            .name(item)
                            .on_press(move |cx| cx.emit(AppEvent::SelectSample(index)))
                            .class("sample-row")
                            .checked(
                                AppData::selected_sample.map(move |selected| *selected == index),
                            );
                        })
                        .height(Stretch(1.0))
                        .width(Stretch(1.0));
                    })
                    .class("sample-browser");
                    VStack::new(cx, |cx| {
                        Label::new(cx, AppData::selected_sample_label)
                            .name("Selected sample")
                            .text_value(AppData::selected_sample_label)
                            .class("section-title");
                        WaveformView::new(cx)
                            .width(Stretch(1.0))
                            .height(Stretch(1.0))
                            .min_height(Pixels(100.0));
                        Label::new(
                            cx,
                            "Original recording • Export processing is applied to a copy",
                        )
                        .class("muted");
                        HStack::new(cx, |cx| {
                            action(cx, "Re-record sample", AppEvent::RecordSelectedSample)
                                .disabled(AppData::controls_busy);
                            action(cx, "Edit capture plan", AppEvent::Disarm)
                                .disabled(AppData::controls_busy);
                        })
                        .class("control-row");
                    })
                    .width(Stretch(1.0))
                    .height(Stretch(1.0))
                    .vertical_gap(Pixels(12.0));
                })
                .height(Stretch(1.0))
                .width(Stretch(1.0))
                .horizontal_gap(Pixels(20.0));
            }
        });

        HStack::new(cx, |cx| {
            Binding::new(cx, AppData::app_state, |cx, state| match state.get(cx) {
                AppState::Idle => {
                    action(cx, "Check input signal", AppEvent::Arm)
                        .class("primary")
                        .disabled(AppData::controls_busy);
                }
                AppState::Armed => {
                    action(cx, "Test note", AppEvent::PlayTestNote)
                        .disabled(AppData::is_testing_note);
                    action(cx, "Start recording", AppEvent::StartRecording)
                        .class("record")
                        .disabled(AppData::is_testing_note);
                    action(cx, "Disarm", AppEvent::Disarm);
                }
                AppState::Recording => {
                    action(cx, "Stop recording", AppEvent::CancelRecording);
                }
                AppState::Stopping => {
                    Label::new(cx, "Releasing notes and saving samples…").class("muted");
                }
                AppState::Review => {
                    Button::new(cx, |cx| {
                        Label::new(
                            cx,
                            AppData::is_playing.map(|playing| {
                                if *playing {
                                    "Stop preview"
                                } else {
                                    "Play sample"
                                }
                            }),
                        )
                    })
                    .name(AppData::is_playing.map(|playing| {
                        if *playing {
                            "Stop preview"
                        } else {
                            "Play selected sample"
                        }
                    }))
                    .on_press(|cx| {
                        if AppData::is_playing.get(cx) {
                            cx.emit(AppEvent::StopPreview);
                        } else {
                            cx.emit(AppEvent::PlayPreview);
                        }
                    })
                    .disabled(AppData::session_busy);
                    Button::new(cx, |cx| {
                        Label::new(
                            cx,
                            AppData::export_format_display
                                .map(|format| format!("Export {}", format)),
                        )
                    })
                    .name(AppData::export_format_display.map(|format| format!("Export {}", format)))
                    .on_press(|cx| cx.emit(AppEvent::ExportAll))
                    .class("primary")
                    .disabled(AppData::controls_busy);
                }
            });
        })
        .class("transport");
        disclosure(
            cx,
            "Synth audition keyboard",
            AppData::app_state.map(|s| *s != AppState::Review),
            keyboard,
        )
        .class("keyboard-disclosure");
    })
    .class("stage");
}
