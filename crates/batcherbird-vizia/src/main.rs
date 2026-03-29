use vizia::prelude::*;
use batcherbird_vizia::app_data::AppData;
use batcherbird_vizia::views::sidebar;

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        cx.add_stylesheet(include_style!("src/style/theme.css"))
            .expect("Failed to load theme");

        AppData::default().build(cx);

        HStack::new(cx, |cx| {
            sidebar(cx);
            VStack::new(cx, |cx| {
                Label::new(cx, "Stage area — recordings will appear here").class("idle-text");
            })
            .class("stage");
        });
    })
    .title("BatcherBird")
    .inner_size((900, 550))
    .run()
}
