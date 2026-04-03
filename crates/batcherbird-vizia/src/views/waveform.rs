use vizia::prelude::*;
use vizia::vg;
use crate::app_data::AppData;

pub struct WaveformView;

impl WaveformView {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        Self.build(cx, |cx| {
            let id = cx.current();
            // Redraw whenever the peaks list changes
            Binding::new(cx, AppData::viz_peaks, move |cx, _val| {
                cx.needs_redraw(id);
            });
        })
    }
}

impl View for WaveformView {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();

        // Draw background
        let bg_path = vg::Path::rect(
            vg::Rect::from_xywh(bounds.x, bounds.y, bounds.w, bounds.h),
            None,
        );
        let mut bg_paint = vg::Paint::default();
        bg_paint.set_color(vg::Color::from_rgb(0x0c, 0x0c, 0x12));
        canvas.draw_path(&bg_path, &bg_paint);

        // Draw center line
        let center_y = bounds.y + bounds.h * 0.5;
        let mut center_line = vg::Path::new();
        center_line.move_to(vg::Point::new(bounds.x, center_y));
        center_line.line_to(vg::Point::new(bounds.x + bounds.w, center_y));

        let mut center_paint = vg::Paint::default();
        center_paint.set_color(vg::Color::from_rgb(0x1e, 0x1e, 0x28));
        center_paint.set_style(vg::PaintStyle::Stroke);
        center_paint.set_stroke_width(1.0);
        canvas.draw_path(&center_line, &center_paint);

        // Read peaks from context
        let peaks = AppData::viz_peaks.get(cx);
        if peaks.is_empty() {
            return;
        }

        let n = peaks.len();
        let x_step = bounds.w / n as f32;

        // Build filled polygon (top edge forward, bottom edge in reverse)
        let mut fill_path = vg::Path::new();

        for (i, &peak) in peaks.iter().enumerate() {
            let x = bounds.x + i as f32 * x_step;
            let y = bounds.y + bounds.h * 0.5 - (peak * bounds.h * 0.5);
            if i == 0 {
                fill_path.move_to(vg::Point::new(x, y));
            } else {
                fill_path.line_to(vg::Point::new(x, y));
            }
        }

        for i in (0..n).rev() {
            let peak = peaks[i];
            let x = bounds.x + i as f32 * x_step;
            let y = bounds.y + bounds.h * 0.5 + (peak * bounds.h * 0.5);
            fill_path.line_to(vg::Point::new(x, y));
        }
        fill_path.close();

        // Fill: translucent blue (~12% opacity = 30/255)
        let mut fill_paint = vg::Paint::default();
        fill_paint.set_color(vg::Color::from_argb(30, 0x4a, 0x9e, 0xff));
        fill_paint.set_style(vg::PaintStyle::Fill);
        fill_paint.set_anti_alias(true);
        canvas.draw_path(&fill_path, &fill_paint);

        // Stroke paint: ~70% opacity blue (178/255)
        let mut stroke_paint = vg::Paint::default();
        stroke_paint.set_color(vg::Color::from_argb(178, 0x4a, 0x9e, 0xff));
        stroke_paint.set_style(vg::PaintStyle::Stroke);
        stroke_paint.set_stroke_width(1.5);
        stroke_paint.set_anti_alias(true);

        // Top edge
        let mut top_path = vg::Path::new();
        for (i, &peak) in peaks.iter().enumerate() {
            let x = bounds.x + i as f32 * x_step;
            let y = bounds.y + bounds.h * 0.5 - (peak * bounds.h * 0.5);
            if i == 0 {
                top_path.move_to(vg::Point::new(x, y));
            } else {
                top_path.line_to(vg::Point::new(x, y));
            }
        }
        canvas.draw_path(&top_path, &stroke_paint);

        // Bottom edge
        let mut bot_path = vg::Path::new();
        for (i, &peak) in peaks.iter().enumerate() {
            let x = bounds.x + i as f32 * x_step;
            let y = bounds.y + bounds.h * 0.5 + (peak * bounds.h * 0.5);
            if i == 0 {
                bot_path.move_to(vg::Point::new(x, y));
            } else {
                bot_path.line_to(vg::Point::new(x, y));
            }
        }
        canvas.draw_path(&bot_path, &stroke_paint);
    }
}

pub fn waveform(cx: &mut Context) {
    WaveformView::new(cx);
}
