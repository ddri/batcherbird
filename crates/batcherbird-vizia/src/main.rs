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

        HStack::new(cx, |cx| {
            sidebar(cx);
            stage(cx);
        });
    })
    .title("BatcherBird")
    .inner_size((900, 550))
    .run()
}
