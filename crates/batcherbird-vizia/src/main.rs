use batcherbird_vizia::app_data::{AppData, AppState};
use batcherbird_vizia::app_event::AppEvent;
use batcherbird_vizia::views::{sidebar, stage, toolbar};
use std::time::Duration;
use vizia::prelude::*;

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        cx.emit(EnvironmentEvent::SetThemeMode(AppTheme::BuiltIn(
            ThemeMode::DarkMode,
        )));
        cx.add_stylesheet(include_style!("src/style/theme.css"))
            .expect("Failed to load theme");

        let demo = std::env::var("BATCHERBIRD_DEMO").as_deref() == Ok("1")
            || std::env::args().any(|arg| {
                arg == "--demo" || arg == "--demo-setup" || arg.starts_with("--demo-state=")
            });
        let mut data = if demo {
            AppData::demo()
        } else {
            AppData::load_preferences()
        };
        if demo {
            let state = std::env::args()
                .find_map(|arg| arg.strip_prefix("--demo-state=").map(str::to_owned))
                .unwrap_or_else(|| std::env::var("BATCHERBIRD_DEMO_STATE").unwrap_or_default());
            data.app_state = match state.as_str() {
                "idle" | "setup" => AppState::Idle,
                "armed" => AppState::Armed,
                "recording" => AppState::Recording,
                "stopping" => AppState::Stopping,
                _ if std::env::args().any(|arg| arg == "--demo-setup") => AppState::Idle,
                _ => AppState::Review,
            };
            if matches!(data.app_state, AppState::Recording | AppState::Stopping) {
                data.notes_completed = 17;
                data.notes_total = 49;
                data.current_note = 52;
                data.current_velocity = 96;
                data.current_layer = 2;
                data.total_layers = 3;
            }
            data.controls_busy = data.is_busy();
        }
        data.build(cx);
        Keymap::from(vec![
            (
                KeyChord::new(Modifiers::SUPER, Code::KeyS),
                KeymapEntry::new("Save session", |cx| cx.emit(AppEvent::SaveSession)),
            ),
            (
                KeyChord::new(Modifiers::SUPER, Code::KeyO),
                KeymapEntry::new("Open session", |cx| cx.emit(AppEvent::OpenSession)),
            ),
            (
                KeyChord::new(Modifiers::CTRL, Code::KeyS),
                KeymapEntry::new("Save session", |cx| cx.emit(AppEvent::SaveSession)),
            ),
            (
                KeyChord::new(Modifiers::CTRL, Code::KeyO),
                KeymapEntry::new("Open session", |cx| cx.emit(AppEvent::OpenSession)),
            ),
        ])
        .build(cx);

        if !demo {
            cx.emit(AppEvent::RefreshDevices);
        }

        let timer = cx.add_timer(
            Duration::from_millis(16), // ~60fps
            None,                      // run forever
            |cx, action| {
                if let TimerAction::Tick(_) = action {
                    cx.emit(AppEvent::Tick);
                }
            },
        );
        cx.start_timer(timer);

        VStack::new(cx, |cx| {
            toolbar(cx);
            HStack::new(cx, |cx| {
                sidebar(cx);
                stage(cx);
            })
            .width(Stretch(1.0))
            .height(Stretch(1.0));
        })
        .width(Stretch(1.0))
        .height(Stretch(1.0));
    })
    .title("BatcherBird — Hardware Auto-Sampler")
    .inner_size((1240, 820))
    .min_inner_size(Some((1040, 720)))
    .run()
}
