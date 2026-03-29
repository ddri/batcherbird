use vizia::prelude::*;

fn main() -> Result<(), ApplicationError> {
    Application::new(|cx| {
        Label::new(cx, "BatcherBird");
    })
    .title("BatcherBird")
    .inner_size((900, 550))
    .run()
}
