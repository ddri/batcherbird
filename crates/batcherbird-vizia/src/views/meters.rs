use crate::app_data::AppData;
use vizia::prelude::*;
use vizia::vg;

fn draw_segmented_meter(level: f32, bounds: &BoundingBox, canvas: &Canvas) {
    if bounds.w <= 0.0 || bounds.h <= 0.0 {
        return;
    }

    // 1. Background track
    let track_rect = vg::Rect::from_xywh(bounds.x, bounds.y, bounds.w, bounds.h);
    let track_path = vg::Path::rect(track_rect, None);
    let mut track_paint = vg::Paint::default();
    track_paint.set_anti_alias(true);
    track_paint.set_color(vg::Color::from_rgb(0x0e, 0x11, 0x1a));
    canvas.draw_path(&track_path, &track_paint);

    // Track border
    let mut track_border = vg::Paint::default();
    track_border.set_anti_alias(true);
    track_border.set_color(vg::Color::from_rgb(0x1a, 0x22, 0x33));
    track_border.set_style(vg::PaintStyle::Stroke);
    track_border.set_stroke_width(1.0);
    canvas.draw_path(&track_path, &track_border);

    // 2. Active Level Fill
    if level > 0.003 {
        let clamped = level.clamp(0.0, 1.0);
        let filled_w = bounds.w * clamped;

        let fill_rect = vg::Rect::from_xywh(bounds.x, bounds.y, filled_w, bounds.h);
        let fill_path = vg::Path::rect(fill_rect, None);
        let mut fill_paint = vg::Paint::default();
        fill_paint.set_anti_alias(true);

        // Hardware color grading:
        // Normal: Green (< 0.65)
        // Caution: Yellow/Amber (0.65 - 0.85)
        // Hot/Clip: Red (>= 0.85)
        let fill_color = if clamped >= 0.85 {
            vg::Color::from_rgb(0xef, 0x44, 0x44) // Clip Red
        } else if clamped >= 0.65 {
            vg::Color::from_rgb(0xf5, 0x9e, 0x0b) // Amber Caution
        } else {
            vg::Color::from_rgb(0x10, 0xb9, 0x81) // Studio Green
        };
        fill_paint.set_color(fill_color);
        canvas.draw_path(&fill_path, &fill_paint);
    }

    // 3. Segment divider lines (simulating hardware LED segments)
    let num_segments = 24;
    let seg_w = bounds.w / num_segments as f32;
    let mut seg_paint = vg::Paint::default();
    seg_paint.set_anti_alias(true);
    seg_paint.set_color(vg::Color::from_rgb(0x08, 0x0a, 0x10));
    seg_paint.set_style(vg::PaintStyle::Stroke);
    seg_paint.set_stroke_width(1.0);

    for i in 1..num_segments {
        let sx = bounds.x + i as f32 * seg_w;
        let mut sp = vg::Path::new();
        sp.move_to(vg::Point::new(sx, bounds.y));
        sp.line_to(vg::Point::new(sx, bounds.y + bounds.h));
        canvas.draw_path(&sp, &seg_paint);
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
        draw_segmented_meter(AppData::meter_left.get(cx), &cx.bounds(), canvas);
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
        draw_segmented_meter(AppData::meter_right.get(cx), &cx.bounds(), canvas);
    }
}

fn meter_channel_row(
    cx: &mut Context,
    channel_name: &str,
    bar: impl FnOnce(&mut Context),
    db_lens: impl Lens<Target = f32>,
) {
    HStack::new(cx, |cx| {
        Label::new(cx, channel_name).class("meter-channel-tag");
        bar(cx);
        Label::new(
            cx,
            db_lens.map(|db: &f32| {
                if *db <= -60.0 {
                    "-inf".to_string()
                } else {
                    format!("{:.1} dB", db)
                }
            }),
        )
        .class("meter-db-tag");
    })
    .class("meter-row");
}

pub fn meters(cx: &mut Context) {
    VStack::new(cx, |cx| {
        // Calibrated scale header
        HStack::new(cx, |cx| {
            Label::new(cx, "VU").class("field-label").width(Pixels(18.0));
            HStack::new(cx, |cx| {
                Label::new(cx, "-48").font_size(9.0).color(Color::from("#475569")).width(Stretch(1.0));
                Label::new(cx, "-24").font_size(9.0).color(Color::from("#475569")).width(Stretch(1.0));
                Label::new(cx, "-12").font_size(9.0).color(Color::from("#475569")).width(Stretch(1.0));
                Label::new(cx, "-6").font_size(9.0).color(Color::from("#64748b")).width(Stretch(1.0));
                Label::new(cx, "0").font_size(9.0).color(Color::from("#f59e0b")).width(Stretch(1.0));
                Label::new(cx, "CLIP").font_size(9.0).color(Color::from("#ef4444")).width(Stretch(1.0));
            })
            .width(Stretch(1.0));
            Element::new(cx).width(Pixels(44.0));
        })
        .height(Auto)
        .horizontal_gap(Pixels(8.0))
        .alignment(Alignment::Center);

        // L channel
        meter_channel_row(
            cx,
            "L",
            |cx| {
                MeterBarLeft::new(cx)
                    .width(Stretch(1.0))
                    .height(Pixels(7.0));
            },
            AppData::meter_left_db,
        );

        // R channel
        meter_channel_row(
            cx,
            "R",
            |cx| {
                MeterBarRight::new(cx)
                    .width(Stretch(1.0))
                    .height(Pixels(7.0));
            },
            AppData::meter_right_db,
        );
    })
    .class("meters-deck");
}
