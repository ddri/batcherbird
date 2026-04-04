use crate::app_data::{AppData, AppState};
use vizia::prelude::*;
use vizia::vg;

pub struct ProgressBarView;

impl ProgressBarView {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        Self.build(cx, |cx| {
            let id = cx.current();
            Binding::new(cx, AppData::notes_completed, move |cx, _| {
                cx.needs_redraw(id);
            });
            Binding::new(cx, AppData::notes_total, move |cx, _| {
                cx.needs_redraw(id);
            });
        })
    }
}

impl View for ProgressBarView {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();
        let completed = AppData::notes_completed.get(cx);
        let total = AppData::notes_total.get(cx);

        // Background track
        let bg_path = vg::Path::rect(
            vg::Rect::from_xywh(bounds.x, bounds.y, bounds.w, bounds.h),
            None,
        );
        let mut bg_paint = vg::Paint::default();
        bg_paint.set_color(vg::Color::from_rgb(0x1a, 0x1a, 0x25));
        canvas.draw_path(&bg_path, &bg_paint);

        if total > 0 {
            let fraction = (completed as f32 / total as f32).clamp(0.0, 1.0);
            let filled_w = bounds.w * fraction;
            let fill_path = vg::Path::rect(
                vg::Rect::from_xywh(bounds.x, bounds.y, filled_w, bounds.h),
                None,
            );
            let mut fill_paint = vg::Paint::default();
            fill_paint.set_color(vg::Color::from_rgb(0x4a, 0x9e, 0xff));
            canvas.draw_path(&fill_path, &fill_paint);
        }

        // Border
        let border_path = vg::Path::rect(
            vg::Rect::from_xywh(bounds.x, bounds.y, bounds.w, bounds.h),
            None,
        );
        let mut border_paint = vg::Paint::default();
        border_paint.set_color(vg::Color::from_rgb(0x33, 0x33, 0x44));
        border_paint.set_style(vg::PaintStyle::Stroke);
        border_paint.set_stroke_width(1.0);
        canvas.draw_path(&border_path, &border_paint);
    }
}

pub fn progress_bar(cx: &mut Context) {
    Binding::new(cx, AppData::app_state, |cx, state| {
        if state.get(cx) == AppState::Recording {
            HStack::new(cx, |cx| {
                ProgressBarView::new(cx)
                    .width(Stretch(1.0))
                    .height(Pixels(8.0));

                // "X / Y" count
                Label::new(cx, AppData::notes_completed.map(|n: &u32| n.to_string()))
                    .class("progress-count");

                Label::new(cx, " / ").class("progress-count");

                Label::new(cx, AppData::notes_total.map(|n: &u32| n.to_string()))
                    .class("progress-count");

                // Percentage
                Binding::new(cx, AppData::notes_completed, |cx, completed| {
                    let completed = completed.get(cx);
                    Binding::new(cx, AppData::notes_total, move |cx, total| {
                        let total = total.get(cx);
                        let pct = if total > 0 {
                            (completed * 100) / total
                        } else {
                            0
                        };
                        Label::new(cx, &format!("{}%", pct)).class("progress-pct");
                    });
                });
            })
            .class("progress-bar-row");
        }
    });
}
