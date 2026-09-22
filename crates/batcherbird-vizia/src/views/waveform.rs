use crate::app_data::AppData;
use vizia::prelude::*;
use vizia::vg;

pub struct WaveformView;

impl WaveformView {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        Self.build(cx, |cx| {
            let id = cx.current();
            // Redraw whenever the peaks list or loop points change
            Binding::new(cx, AppData::viz_peaks, move |cx, _val| {
                cx.needs_redraw(id);
            });
            Binding::new(cx, AppData::loop_start, move |cx, _val| {
                cx.needs_redraw(id);
            });
            Binding::new(cx, AppData::loop_end, move |cx, _val| {
                cx.needs_redraw(id);
            });
        })
    }
}

impl View for WaveformView {
    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();
        if bounds.w <= 0.0 || bounds.h <= 0.0 {
            return;
        }

        // 1. Draw Oscilloscope Screen Enclosure
        let screen_rect = vg::Rect::from_xywh(bounds.x, bounds.y, bounds.w, bounds.h);
        let screen_path = vg::Path::rect(screen_rect, None);
        let mut screen_paint = vg::Paint::default();
        screen_paint.set_anti_alias(true);
        screen_paint.set_color(vg::Color::from_rgb(0x08, 0x0a, 0x10));
        canvas.draw_path(&screen_path, &screen_paint);

        // Border
        let mut border_paint = vg::Paint::default();
        border_paint.set_anti_alias(true);
        border_paint.set_color(vg::Color::from_rgb(0x1a, 0x22, 0x33));
        border_paint.set_style(vg::PaintStyle::Stroke);
        border_paint.set_stroke_width(1.0);
        canvas.draw_path(&screen_path, &border_paint);

        // 2. Oscilloscope Reticle Grid
        let mut grid_paint = vg::Paint::default();
        grid_paint.set_anti_alias(true);
        grid_paint.set_color(vg::Color::from_rgb(0x10, 0x15, 0x22));
        grid_paint.set_style(vg::PaintStyle::Stroke);
        grid_paint.set_stroke_width(0.75);

        // Vertical grid lines (8 divisions)
        let v_divisions = 8;
        let v_step = bounds.w / v_divisions as f32;
        for i in 1..v_divisions {
            let gx = bounds.x + i as f32 * v_step;
            let mut v_path = vg::Path::new();
            v_path.move_to(vg::Point::new(gx, bounds.y));
            v_path.line_to(vg::Point::new(gx, bounds.y + bounds.h));
            canvas.draw_path(&v_path, &grid_paint);
        }

        // Horizontal amplitude guide lines (headroom ±6 dB and ±12 dB)
        for factor in [0.15, 0.30, 0.70, 0.85] {
            let gy = bounds.y + bounds.h * factor;
            let mut h_path = vg::Path::new();
            h_path.move_to(vg::Point::new(bounds.x, gy));
            h_path.line_to(vg::Point::new(bounds.x + bounds.w, gy));
            canvas.draw_path(&h_path, &grid_paint);
        }

        // Center zero line (DC baseline)
        let center_y = bounds.y + bounds.h * 0.5;
        let mut center_line = vg::Path::new();
        center_line.move_to(vg::Point::new(bounds.x, center_y));
        center_line.line_to(vg::Point::new(bounds.x + bounds.w, center_y));

        let mut center_paint = vg::Paint::default();
        center_paint.set_anti_alias(true);
        center_paint.set_color(vg::Color::from_rgb(0x20, 0x2b, 0x40));
        center_paint.set_style(vg::PaintStyle::Stroke);
        center_paint.set_stroke_width(1.0);
        canvas.draw_path(&center_line, &center_paint);

        // 3. Draw Loop Region & Markers (if detected)
        let loop_start = AppData::loop_start.get(cx);
        let loop_end = AppData::loop_end.get(cx);
        let total_len = AppData::sample_total_len.get(cx);

        if let (Some(l_start), Some(l_end)) = (loop_start, loop_end) {
            if total_len > 0 && l_end > l_start && l_end <= total_len {
                let start_ratio = l_start as f32 / total_len as f32;
                let end_ratio = l_end as f32 / total_len as f32;

                let start_x = bounds.x + start_ratio * bounds.w;
                let end_x = bounds.x + end_ratio * bounds.w;

                // Loop sustained region fill (subtle emerald glow)
                let loop_rect = vg::Rect::from_xywh(start_x, bounds.y, end_x - start_x, bounds.h);
                let loop_bg_path = vg::Path::rect(loop_rect, None);
                let mut loop_bg_paint = vg::Paint::default();
                loop_bg_paint.set_color(vg::Color::from_argb(30, 0x10, 0xb9, 0x81));
                loop_bg_paint.set_style(vg::PaintStyle::Fill);
                canvas.draw_path(&loop_bg_path, &loop_bg_paint);

                // Loop Start Line (Neon Emerald)
                let mut start_line = vg::Path::new();
                start_line.move_to(vg::Point::new(start_x, bounds.y));
                start_line.line_to(vg::Point::new(start_x, bounds.y + bounds.h));

                let mut start_paint = vg::Paint::default();
                start_paint.set_anti_alias(true);
                start_paint.set_color(vg::Color::from_rgb(0x10, 0xb9, 0x81));
                start_paint.set_style(vg::PaintStyle::Stroke);
                start_paint.set_stroke_width(2.0);
                canvas.draw_path(&start_line, &start_paint);

                // Loop End Line (Hardware Amber)
                let mut end_line = vg::Path::new();
                end_line.move_to(vg::Point::new(end_x, bounds.y));
                end_line.line_to(vg::Point::new(end_x, bounds.y + bounds.h));

                let mut end_paint = vg::Paint::default();
                end_paint.set_anti_alias(true);
                end_paint.set_color(vg::Color::from_rgb(0xf5, 0x9e, 0x0b));
                end_paint.set_style(vg::PaintStyle::Stroke);
                end_paint.set_stroke_width(2.0);
                canvas.draw_path(&end_line, &end_paint);
            }
        }

        // 4. Waveform Data vs. Standby Oscilloscope Trace
        let peaks = AppData::viz_peaks.get(cx);
        if peaks.is_empty() {
            // Draw a subtle standby trace along the center line to eliminate empty void
            let mut standby_path = vg::Path::new();
            let num_points = 64;
            let step = bounds.w / num_points as f32;
            for i in 0..=num_points {
                let x = bounds.x + i as f32 * step;
                // Very subtle gentle sine wave envelope centered on screen
                let norm = (i as f32 / num_points as f32) * std::f32::consts::PI * 2.0;
                let amp = ((norm * 2.0).sin() * 3.0) * (norm.sin().abs());
                let y = center_y + amp;
                if i == 0 {
                    standby_path.move_to(vg::Point::new(x, y));
                } else {
                    standby_path.line_to(vg::Point::new(x, y));
                }
            }

            let mut standby_paint = vg::Paint::default();
            standby_paint.set_anti_alias(true);
            standby_paint.set_color(vg::Color::from_argb(60, 0x38, 0xbd, 0xf8));
            standby_paint.set_style(vg::PaintStyle::Stroke);
            standby_paint.set_stroke_width(1.0);
            canvas.draw_path(&standby_path, &standby_paint);
            return;
        }

        // Recorded Audio Waveform
        let n = peaks.len();
        let x_step = bounds.w / n as f32;

        // Build filled polygon
        let mut fill_path = vg::Path::new();
        for (i, &peak) in peaks.iter().enumerate() {
            let x = bounds.x + i as f32 * x_step;
            let y = bounds.y + bounds.h * 0.5 - (peak * bounds.h * 0.48);
            if i == 0 {
                fill_path.move_to(vg::Point::new(x, y));
            } else {
                fill_path.line_to(vg::Point::new(x, y));
            }
        }
        for i in (0..n).rev() {
            let peak = peaks[i];
            let x = bounds.x + i as f32 * x_step;
            let y = bounds.y + bounds.h * 0.5 + (peak * bounds.h * 0.48);
            fill_path.line_to(vg::Point::new(x, y));
        }
        fill_path.close();

        // Waveform interior gradient/fill
        let mut fill_paint = vg::Paint::default();
        fill_paint.set_anti_alias(true);
        fill_paint.set_color(vg::Color::from_argb(45, 0x00, 0xe5, 0xff));
        fill_paint.set_style(vg::PaintStyle::Fill);
        canvas.draw_path(&fill_path, &fill_paint);

        // Waveform contour strokes (electric cyan)
        let mut stroke_paint = vg::Paint::default();
        stroke_paint.set_anti_alias(true);
        stroke_paint.set_color(vg::Color::from_rgb(0x38, 0xbd, 0xf8));
        stroke_paint.set_style(vg::PaintStyle::Stroke);
        stroke_paint.set_stroke_width(1.5);

        // Top Edge
        let mut top_path = vg::Path::new();
        for (i, &peak) in peaks.iter().enumerate() {
            let x = bounds.x + i as f32 * x_step;
            let y = bounds.y + bounds.h * 0.5 - (peak * bounds.h * 0.48);
            if i == 0 {
                top_path.move_to(vg::Point::new(x, y));
            } else {
                top_path.line_to(vg::Point::new(x, y));
            }
        }
        canvas.draw_path(&top_path, &stroke_paint);

        // Bottom Edge
        let mut bot_path = vg::Path::new();
        for (i, &peak) in peaks.iter().enumerate() {
            let x = bounds.x + i as f32 * x_step;
            let y = bounds.y + bounds.h * 0.5 + (peak * bounds.h * 0.48);
            if i == 0 {
                bot_path.move_to(vg::Point::new(x, y));
            } else {
                bot_path.line_to(vg::Point::new(x, y));
            }
        }
        canvas.draw_path(&bot_path, &stroke_paint);
    }
}
