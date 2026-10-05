use vizia::prelude::*;

#[derive(Lens)]
pub(super) struct Disclosure {
    is_open: bool,
}

struct ToggleDisclosure;

impl View for Disclosure {
    fn element(&self) -> Option<&'static str> {
        Some("disclosure")
    }

    fn event(&mut self, _cx: &mut EventContext, event: &mut Event) {
        event.map(|_: &ToggleDisclosure, meta| {
            self.is_open = !self.is_open;
            meta.consume();
        });
    }
}

/// A native disclosure with a real button header, so Tab, Space and Enter work.
/// State follows the workflow initially and when the app changes capture stage;
/// manual expansion remains local to this view between those transitions.
pub(super) fn disclosure<'a>(
    cx: &'a mut Context,
    title: &str,
    open: impl Res<bool>,
    content: impl FnOnce(&mut Context),
) -> Handle<'a, Disclosure> {
    Disclosure { is_open: false }
        .build(cx, |cx| {
            let target = cx.current();
            Button::new(cx, |cx| {
                HStack::new(cx, |cx| {
                    Label::new(cx, title).width(Stretch(1.0)).hoverable(false);
                    Label::new(
                        cx,
                        Disclosure::is_open.map(|open| if *open { "−" } else { "+" }),
                    )
                    .hoverable(false);
                })
                .width(Stretch(1.0))
                .height(Auto)
                .alignment(Alignment::Center)
            })
            .name(title.to_owned())
            .text_value(
                Disclosure::is_open.map(|open| if *open { "Expanded" } else { "Collapsed" }),
            )
            .checked(Disclosure::is_open)
            .on_press(move |cx| cx.emit_to(target, ToggleDisclosure))
            .class("disclosure-header")
            .width(Stretch(1.0));
            VStack::new(cx, content)
                .class("disclosure-content")
                .display(Disclosure::is_open.map(|open| {
                    if *open {
                        Display::Flex
                    } else {
                        Display::None
                    }
                }));
        })
        .bind(open, |handle, open| {
            let is_open = open.get(&handle);
            handle.modify(|view| view.is_open = is_open);
        })
}
