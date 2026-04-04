use crate::app_data::AppData;
use vizia::prelude::*;
use vizia::vg;

fn is_black_key(note: u8) -> bool {
    matches!(note % 12, 1 | 3 | 6 | 8 | 10)
}

pub struct KeyboardView;

impl KeyboardView {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        Self.build(cx, |cx| {
            let id = cx.current();
            Binding::new(cx, AppData::start_note, move |cx, _| cx.needs_redraw(id));
            Binding::new(cx, AppData::end_note, move |cx, _| cx.needs_redraw(id));
            Binding::new(cx, AppData::current_note, move |cx, _| cx.needs_redraw(id));
        })
    }
}

impl View for KeyboardView {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();

        let start_note = AppData::start_note.get(cx);
        let end_note = AppData::end_note.get(cx);
        let current_note = AppData::current_note.get(cx);

        // Draw background
        let bg_path = vg::Path::rect(
            vg::Rect::from_xywh(bounds.x, bounds.y, bounds.w, bounds.h),
            None,
        );
        let mut bg_paint = vg::Paint::default();
        bg_paint.set_color(vg::Color::from_rgb(0x0c, 0x0c, 0x12));
        canvas.draw_path(&bg_path, &bg_paint);

        // Display range: 4 octaves anchored at start_note's octave
        let display_start = (start_note / 12) * 12;
        let display_end = (display_start + 48).min(127);

        // Count white keys
        let white_count = (display_start..=display_end)
            .filter(|&n| !is_black_key(n))
            .count() as f32;

        if white_count < 1.0 {
            return;
        }

        let white_w = bounds.w / white_count;
        let white_h = bounds.h;
        let black_w = white_w * 0.6;
        let black_h = white_h * 0.6;

        // Draw white keys
        let mut white_x = bounds.x;
        for note in display_start..=display_end {
            if is_black_key(note) {
                continue;
            }

            let color = if note == current_note {
                vg::Color::from_rgb(0x4a, 0x9e, 0xff)
            } else if note >= start_note && note <= end_note {
                vg::Color::from_rgb(0x88, 0xbb, 0xee)
            } else {
                vg::Color::from_rgb(0xee, 0xee, 0xee)
            };

            let key_path = vg::Path::rect(
                vg::Rect::from_xywh(white_x + 0.5, bounds.y + 0.5, white_w - 1.0, white_h - 1.0),
                None,
            );
            let mut key_paint = vg::Paint::default();
            key_paint.set_color(color);
            canvas.draw_path(&key_path, &key_paint);

            let border_path = vg::Path::rect(
                vg::Rect::from_xywh(white_x + 0.5, bounds.y + 0.5, white_w - 1.0, white_h - 1.0),
                None,
            );
            let mut border_paint = vg::Paint::default();
            border_paint.set_color(vg::Color::from_rgb(0x33, 0x33, 0x44));
            border_paint.set_style(vg::PaintStyle::Stroke);
            border_paint.set_stroke_width(0.5);
            canvas.draw_path(&border_path, &border_paint);

            white_x += white_w;
        }

        // Draw black keys on top
        let mut white_x = bounds.x;
        for note in display_start..=display_end {
            if is_black_key(note) {
                let bx = white_x - black_w * 0.5;

                let color = if note == current_note {
                    vg::Color::from_rgb(0x4a, 0x9e, 0xff)
                } else if note >= start_note && note <= end_note {
                    vg::Color::from_rgb(0x22, 0x66, 0xaa)
                } else {
                    vg::Color::from_rgb(0x18, 0x18, 0x22)
                };

                let key_path =
                    vg::Path::rect(vg::Rect::from_xywh(bx, bounds.y, black_w, black_h), None);
                let mut key_paint = vg::Paint::default();
                key_paint.set_color(color);
                canvas.draw_path(&key_path, &key_paint);
            } else {
                white_x += white_w;
            }
        }
    }
}

pub fn keyboard(cx: &mut Context) {
    KeyboardView::new(cx)
        .height(Pixels(24.0))
        .width(Stretch(1.0));
}
