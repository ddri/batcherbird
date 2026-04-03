use std::time::Duration;
use vizia::prelude::*;
use batcherbird_vizia::app_data::AppData;
use batcherbird_vizia::app_event::AppEvent;
use batcherbird_vizia::views::{sidebar, stage};

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        cx.add_stylesheet(include_style!("src/style/theme.css"))
            .expect("Failed to load theme");

        AppData::default().build(cx);

        cx.emit(AppEvent::RefreshDevices);

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

        HStack::new(cx, |cx| {
            sidebar(cx);
            stage(cx);
        });
    })
    .title("BatcherBird")
    .inner_size((900, 550))
    .run()
}
