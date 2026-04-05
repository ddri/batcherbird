use crate::app_data::AppData;
use vizia::prelude::*;
use vizia::vg;

fn draw_meter_bar(level: f32, bounds: &BoundingBox, canvas: &Canvas) {
    // Background track
    let bg_path = vg::Path::rect(
        vg::Rect::from_xywh(bounds.x, bounds.y, bounds.w, bounds.h),
        None,
    );
    let mut bg_paint = vg::Paint::default();
    bg_paint.set_color(vg::Color::from_rgb(0x16, 0x16, 0x20));
    canvas.draw_path(&bg_path, &bg_paint);

    // Only draw fill if signal is meaningful
    if level > 0.005 {
        let clamped = level.clamp(0.0, 1.0);
        let filled_w = bounds.w * clamped;
        let fill_path = vg::Path::rect(
            vg::Rect::from_xywh(bounds.x, bounds.y, filled_w, bounds.h),
            None,
        );
        let mut fill_paint = vg::Paint::default();
        // Color by level: green < 0.6, yellow < 0.8, red >= 0.8
        fill_paint.set_color(if clamped > 0.8 {
            vg::Color::from_rgb(0xe5, 0x39, 0x35)
        } else if clamped > 0.6 {
            vg::Color::from_rgb(0xfe, 0xbc, 0x2e)
        } else {
            vg::Color::from_rgb(0x28, 0xc8, 0x40)
        });
        canvas.draw_path(&fill_path, &fill_paint);
    }
}

pub struct MeterBarLeft;
impl MeterBarLeft {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        Self.build(cx, |cx| {
            let id = cx.current();
            Binding::new(cx, AppData::meter_left, move |cx, _| cx.needs_redraw(id));
        })
    }
}
impl View for MeterBarLeft {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        draw_meter_bar(AppData::meter_left.get(cx), &cx.bounds(), canvas);
    }
}

pub struct MeterBarRight;
impl MeterBarRight {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        Self.build(cx, |cx| {
            let id = cx.current();
            Binding::new(cx, AppData::meter_right, move |cx, _| cx.needs_redraw(id));
        })
    }
}
impl View for MeterBarRight {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        draw_meter_bar(AppData::meter_right.get(cx), &cx.bounds(), canvas);
    }
}

fn meter_row(
    cx: &mut Context,
    label: &str,
    bar: impl FnOnce(&mut Context),
    db_lens: impl Lens<Target = f32>,
) {
    HStack::new(cx, |cx| {
        Label::new(cx, label)
            .font_size(10.0)
            .color(Color::from("#444444"))
            .width(Pixels(12.0));
        bar(cx);
        Label::new(
            cx,
            db_lens.map(|db: &f32| {
                if *db <= -60.0 {
                    String::new() // show nothing at idle
                } else {
                    format!("{:.0}dB", db)
                }
            }),
        )
        .font_size(9.0)
        .color(Color::from("#444444"))
        .width(Pixels(32.0));
    })
    .width(Stretch(1.0))
    .height(Auto)
    .horizontal_gap(Pixels(6.0))
    .alignment(Alignment::Left);
}

pub fn meters(cx: &mut Context) {
    VStack::new(cx, |cx| {
        meter_row(cx, "L", |cx| {
            MeterBarLeft::new(cx)
                .width(Stretch(1.0))
                .height(Pixels(4.0));
        }, AppData::meter_left_db);
        meter_row(cx, "R", |cx| {
            MeterBarRight::new(cx)
                .width(Stretch(1.0))
                .height(Pixels(4.0));
        }, AppData::meter_right_db);
    })
    .width(Stretch(1.0))
    .height(Auto)
    .vertical_gap(Pixels(3.0));
}
