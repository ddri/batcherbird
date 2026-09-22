use crate::app_data::AppData;
use crate::app_event::AppEvent;
use vizia::prelude::*;
use vizia::vg;

pub fn is_black_key(note: u8) -> bool {
    matches!(note % 12, 1 | 3 | 6 | 8 | 10)
}

/// Hit-tests a coordinate against the virtual keyboard layout, returning the MIDI note if hit.
#[allow(clippy::too_many_arguments)]
pub fn hit_test_note(
    bounds_x: f32,
    bounds_y: f32,
    bounds_w: f32,
    bounds_h: f32,
    display_start: u8,
    display_end: u8,
    cursor_x: f32,
    cursor_y: f32,
) -> Option<u8> {
    if bounds_w <= 0.0 || bounds_h <= 0.0 {
        return None;
    }
    if cursor_x < bounds_x
        || cursor_x >= bounds_x + bounds_w
        || cursor_y < bounds_y
        || cursor_y >= bounds_y + bounds_h
    {
        return None;
    }

    let white_count = (display_start..=display_end)
        .filter(|&n| !is_black_key(n))
        .count() as f32;

    if white_count < 1.0 {
        return None;
    }

    let white_w = bounds_w / white_count;
    let black_w = white_w * 0.6;
    let black_h = bounds_h * 0.6;

    let rel_x = cursor_x - bounds_x;
    let rel_y = cursor_y - bounds_y;

    // Check black keys first if cursor is in the upper 60% of the keyboard
    if rel_y <= black_h {
        let mut white_x = 0.0;
        for note in display_start..=display_end {
            if is_black_key(note) {
                let bx = white_x - black_w * 0.5;
                if rel_x >= bx && rel_x <= bx + black_w {
                    return Some(note);
                }
            } else {
                white_x += white_w;
            }
        }
    }

    // Otherwise (or if no black key was hit), determine white key
    let white_index = (rel_x / white_w).floor() as usize;
    let mut current_white_idx = 0;
    for note in display_start..=display_end {
        if !is_black_key(note) {
            if current_white_idx == white_index {
                return Some(note);
            }
            current_white_idx += 1;
        }
    }

    None
}

pub struct KeyboardView {
    held_note: Option<u8>,
}

impl KeyboardView {
    pub fn new(cx: &mut Context) -> Handle<'_, Self> {
        Self { held_note: None }.build(cx, |cx| {
            let id = cx.current();
            Binding::new(cx, AppData::start_note, move |cx, _| cx.needs_redraw(id));
            Binding::new(cx, AppData::end_note, move |cx, _| cx.needs_redraw(id));
            Binding::new(cx, AppData::note_step, move |cx, _| cx.needs_redraw(id));
            Binding::new(cx, AppData::current_note, move |cx, _| cx.needs_redraw(id));
            Binding::new(cx, AppData::audition_note, move |cx, _| cx.needs_redraw(id));
        })
    }
}

impl View for KeyboardView {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|window_event: &WindowEvent, _| match window_event {
            WindowEvent::MouseDown(btn) => {
                let bounds = cx.bounds();
                let start_note = AppData::start_note.get(cx);
                let display_start = (start_note / 12) * 12;
                let display_end = (display_start + 48).min(127);

                let cursor_x = cx.mouse().cursor_x;
                let cursor_y = cx.mouse().cursor_y;

                if let Some(note) = hit_test_note(
                    bounds.x,
                    bounds.y,
                    bounds.w,
                    bounds.h,
                    display_start,
                    display_end,
                    cursor_x,
                    cursor_y,
                ) {
                    let is_shift = cx.modifiers().contains(Modifiers::SHIFT);
                    let is_right = *btn == MouseButton::Right;

                    if is_right {
                        // Right-click: Set Start Note
                        cx.emit(AppEvent::SetStartNote(note));
                        cx.emit(AppEvent::AuditionNoteOn(note));
                        self.held_note = Some(note);
                    } else if *btn == MouseButton::Left {
                        if is_shift {
                            // Shift + Left-click: Set End Note
                            cx.emit(AppEvent::SetEndNote(note));
                            cx.emit(AppEvent::AuditionNoteOn(note));
                            self.held_note = Some(note);
                        } else {
                            // Left-click: Audition Note
                            cx.emit(AppEvent::AuditionNoteOn(note));
                            self.held_note = Some(note);
                        }
                    }
                }
            }
            WindowEvent::MouseUp(btn) => {
                if (*btn == MouseButton::Left || *btn == MouseButton::Right)
                    && self.held_note.is_some()
                {
                    self.held_note = None;
                    cx.emit(AppEvent::AuditionNoteOff);
                }
            }
            WindowEvent::MouseLeave => {
                if self.held_note.is_some() {
                    self.held_note = None;
                    cx.emit(AppEvent::AuditionNoteOff);
                }
            }
            _ => {}
        });
    }

    fn draw(&self, cx: &mut DrawContext, canvas: &Canvas) {
        let bounds = cx.bounds();

        let start_note = AppData::start_note.get(cx);
        let end_note = AppData::end_note.get(cx);
        let note_step = AppData::note_step.get(cx).max(1);
        let current_note = AppData::current_note.get(cx);
        let audition_note = AppData::audition_note.get(cx);

        let is_stepped_target = |note: u8| -> bool {
            note >= start_note && note <= end_note && (note - start_note).is_multiple_of(note_step)
        };

        // Draw keybed backing
        let bg_path = vg::Path::rect(
            vg::Rect::from_xywh(bounds.x, bounds.y, bounds.w, bounds.h),
            None,
        );
        let mut bg_paint = vg::Paint::default();
        bg_paint.set_anti_alias(true);
        bg_paint.set_color(vg::Color::from_rgb(0x0a, 0x0b, 0x10));
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

        // -------------------------------------------------------------
        // Pass 1: Draw White Keys
        // -------------------------------------------------------------
        let mut white_x = bounds.x;
        for note in display_start..=display_end {
            if is_black_key(note) {
                continue;
            }

            let is_audition = Some(note) == audition_note;
            let is_current = note == current_note;
            let is_target = is_stepped_target(note);
            let in_range = note >= start_note && note <= end_note;

            // Base key color
            let key_color = if is_audition {
                vg::Color::from_rgb(0x00, 0xe5, 0xff) // Luminous cyan
            } else if is_current {
                vg::Color::from_rgb(0x3b, 0x82, 0xf6) // Active blue
            } else if in_range {
                vg::Color::from_rgb(0xeb, 0xf0, 0xfa) // Subtle studio ivory
            } else {
                vg::Color::from_rgb(0xd1, 0xd5, 0xdb) // Realistic off-white
            };

            let key_rect = vg::Rect::from_xywh(
                white_x + 0.5,
                bounds.y + 0.5,
                white_w - 1.0,
                white_h - 1.0,
            );
            let key_path = vg::Path::rect(key_rect, None);
            let mut key_paint = vg::Paint::default();
            key_paint.set_anti_alias(true);
            key_paint.set_color(key_color);
            canvas.draw_path(&key_path, &key_paint);

            // Subtle bottom bevel lip
            if !is_audition && !is_current {
                let lip_rect = vg::Rect::from_xywh(
                    white_x + 0.5,
                    bounds.y + white_h - 4.0,
                    white_w - 1.0,
                    3.5,
                );
                let lip_path = vg::Path::rect(lip_rect, None);
                let mut lip_paint = vg::Paint::default();
                lip_paint.set_anti_alias(true);
                lip_paint.set_color(vg::Color::from_rgb(0xb0, 0xb7, 0xc3));
                canvas.draw_path(&lip_path, &lip_paint);
            }

            // Key separator stroke
            let border_path = vg::Path::rect(key_rect, None);
            let mut border_paint = vg::Paint::default();
            border_paint.set_anti_alias(true);
            border_paint.set_color(vg::Color::from_rgb(0x18, 0x1b, 0x26));
            border_paint.set_style(vg::PaintStyle::Stroke);
            border_paint.set_stroke_width(1.0);
            canvas.draw_path(&border_path, &border_paint);

            // Stepped Target Note: draw glowing LED jewel pip at bottom
            if is_target && !is_audition && !is_current {
                let pip_w = (white_w * 0.38).clamp(4.0, 10.0);
                let pip_h = 4.0;
                let pip_x = white_x + (white_w - pip_w) * 0.5;
                let pip_y = bounds.y + white_h - 9.0;

                let pip_rect = vg::Rect::from_xywh(pip_x, pip_y, pip_w, pip_h);
                let pip_path = vg::Path::rect(pip_rect, None);
                let mut pip_paint = vg::Paint::default();
                pip_paint.set_anti_alias(true);
                pip_paint.set_color(vg::Color::from_rgb(0x02, 0x84, 0xc7)); // Studio cyan LED
                canvas.draw_path(&pip_path, &pip_paint);
            }

            white_x += white_w;
        }

        // -------------------------------------------------------------
        // Pass 2: Draw Black Keys
        // -------------------------------------------------------------
        let mut white_x = bounds.x;
        for note in display_start..=display_end {
            if is_black_key(note) {
                let bx = white_x - black_w * 0.5;

                let is_audition = Some(note) == audition_note;
                let is_current = note == current_note;
                let is_target = is_stepped_target(note);
                let in_range = note >= start_note && note <= end_note;

                // 1. Soft Drop Shadow onto white keys
                let shadow_rect = vg::Rect::from_xywh(
                    bx - 1.0,
                    bounds.y,
                    black_w + 2.0,
                    black_h + 3.0,
                );
                let shadow_path = vg::Path::rect(shadow_rect, None);
                let mut shadow_paint = vg::Paint::default();
                shadow_paint.set_anti_alias(true);
                shadow_paint.set_color(vg::Color::from_argb(80, 0, 0, 0));
                canvas.draw_path(&shadow_path, &shadow_paint);

                // 2. Black Key Body
                let key_color = if is_audition {
                    vg::Color::from_rgb(0x00, 0xb4, 0xd8)
                } else if is_current {
                    vg::Color::from_rgb(0x25, 0x63, 0xeb)
                } else if is_target {
                    vg::Color::from_rgb(0x18, 0x1c, 0x28)
                } else if in_range {
                    vg::Color::from_rgb(0x16, 0x19, 0x24)
                } else {
                    vg::Color::from_rgb(0x10, 0x12, 0x1a)
                };

                let key_rect = vg::Rect::from_xywh(bx, bounds.y, black_w, black_h);
                let key_path = vg::Path::rect(key_rect, None);
                let mut key_paint = vg::Paint::default();
                key_paint.set_anti_alias(true);
                key_paint.set_color(key_color);
                canvas.draw_path(&key_path, &key_paint);

                // 3. Top Sheen / Bevel
                if !is_audition && !is_current {
                    let sheen_w = (black_w - 2.0).max(1.0);
                    let sheen_h = (black_h - 4.0).max(1.0);
                    let sheen_rect = vg::Rect::from_xywh(bx + 1.0, bounds.y + 1.0, sheen_w, sheen_h);
                    let sheen_path = vg::Path::rect(sheen_rect, None);
                    let mut sheen_paint = vg::Paint::default();
                    sheen_paint.set_anti_alias(true);
                    sheen_paint.set_color(if is_target {
                        vg::Color::from_rgb(0x2d, 0x4f, 0x7c)
                    } else {
                        vg::Color::from_rgb(0x22, 0x26, 0x36)
                    });
                    canvas.draw_path(&sheen_path, &sheen_paint);
                }

                // Stepped Target Note on Black Key: bright cyan pip at bottom
                if is_target && !is_audition && !is_current {
                    let pip_w = (black_w * 0.45).clamp(3.0, 7.0);
                    let pip_h = 3.0;
                    let pip_x = bx + (black_w - pip_w) * 0.5;
                    let pip_y = bounds.y + black_h - 5.0;

                    let pip_rect = vg::Rect::from_xywh(pip_x, pip_y, pip_w, pip_h);
                    let pip_path = vg::Path::rect(pip_rect, None);
                    let mut pip_paint = vg::Paint::default();
                    pip_paint.set_anti_alias(true);
                    pip_paint.set_color(vg::Color::from_rgb(0x38, 0xbd, 0xf8));
                    canvas.draw_path(&pip_path, &pip_paint);
                }
            } else {
                white_x += white_w;
            }
        }
    }
}

pub fn keyboard(cx: &mut Context) {
    VStack::new(cx, |cx| {
        KeyboardView::new(cx)
            .height(Pixels(64.0))
            .width(Stretch(1.0))
            .corner_radius(Pixels(6.0))
            .cursor(CursorIcon::Hand);

        HStack::new(cx, |cx| {
            Label::new(
                cx,
                "Audition: Left-click  ·  Set Start: Right-click  ·  Set End: Shift-click",
            )
            .font_size(10.0)
            .color(Color::from("#64748b"))
            .width(Stretch(1.0))
            .alignment(Alignment::Center);
        })
        .height(Auto)
        .width(Stretch(1.0));
    })
    .height(Auto)
    .width(Stretch(1.0))
    .vertical_gap(Pixels(4.0));
}
