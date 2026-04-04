use crate::app_data::AppData;
use vizia::prelude::*;
use vizia::vg;

fn meter_color(level: f32) -> vg::Color {
    if level > 0.8 {
        vg::Color::from_rgb(0xe5, 0x39, 0x35) // red
    } else if level > 0.6 {
        vg::Color::from_rgb(0xfe, 0xbc, 0x2e) // yellow
    } else {
        vg::Color::from_rgb(0x28, 0xc8, 0x40) // green
    }
}

fn draw_meter_bar(level: f32, bounds: &BoundingBox, canvas: &Canvas) {
    // Background
    let bg_path = vg::Path::rect(
        vg::Rect::from_xywh(bounds.x, bounds.y, bounds.w, bounds.h),
        None,
    );
    let mut bg_paint = vg::Paint::default();
    bg_paint.set_color(vg::Color::from_rgb(0x1a, 0x1a, 0x25));
    canvas.draw_path(&bg_path, &bg_paint);

    // Fill
    if level > 0.0 {
        let filled_w = bounds.w * level.clamp(0.0, 1.0);
        let fill_path = vg::Path::rect(
            vg::Rect::from_xywh(bounds.x, bounds.y, filled_w, bounds.h),
            None,
        );
        let mut fill_paint = vg::Paint::default();
        fill_paint.set_color(meter_color(level));
        canvas.draw_path(&fill_path, &fill_paint);
    }
}

// ---- Left meter bar ----

pub struct MeterBarLeft;

impl MeterBarLeft {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        let entity = Self.build(cx, |cx| {
            // Bind to meter_left so we redraw when it changes
            let id = cx.current();
            Binding::new(cx, AppData::meter_left, move |cx, _val| {
                cx.needs_redraw(id);
            });
        });
        entity
    }
}

impl View for MeterBarLeft {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let level = AppData::meter_left.get(cx);
        let bounds = cx.bounds();
        draw_meter_bar(level, &bounds, canvas);
    }
}

// ---- Right meter bar ----

pub struct MeterBarRight;

impl MeterBarRight {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        Self.build(cx, |cx| {
            let id = cx.current();
            Binding::new(cx, AppData::meter_right, move |cx, _val| {
                cx.needs_redraw(id);
            });
        })
    }
}

impl View for MeterBarRight {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let level = AppData::meter_right.get(cx);
        let bounds = cx.bounds();
        draw_meter_bar(level, &bounds, canvas);
    }
}

// ---- meters() layout function ----

pub fn meters(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // Left channel row
        HStack::new(cx, |cx| {
            Label::new(cx, "L").class("meter-label");
            MeterBarLeft::new(cx)
                .width(Stretch(1.0))
                .height(Pixels(8.0));
            Label::new(
                cx,
                AppData::meter_left_db.map(|db: &f32| {
                    if *db <= -60.0 {
                        "-inf".to_string()
                    } else {
                        format!("{:.0}", db)
                    }
                }),
            )
            .class("meter-db");
        })
        .class("meter-row");

        // Right channel row
        HStack::new(cx, |cx| {
            Label::new(cx, "R").class("meter-label");
            MeterBarRight::new(cx)
                .width(Stretch(1.0))
                .height(Pixels(8.0));
            Label::new(
                cx,
                AppData::meter_right_db.map(|db: &f32| {
                    if *db <= -60.0 {
                        "-inf".to_string()
                    } else {
                        format!("{:.0}", db)
                    }
                }),
            )
            .class("meter-db");
        })
        .class("meter-row");
    })
    .class("meters");
}
