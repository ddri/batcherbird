use vizia::prelude::*;
use batcherbird_vizia::app_data::AppData;

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        AppData::default().build(cx);
        Label::new(cx, "BatcherBird");
    })
    .title("BatcherBird")
    .inner_size((900, 550))
    .run()
}
