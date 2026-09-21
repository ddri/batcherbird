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

            let color = if Some(note) == audition_note {
                vg::Color::from_rgb(0x00, 0xe5, 0xff) // bright audition cyan
            } else if note == current_note {
                vg::Color::from_rgb(0x4a, 0x9e, 0xff) // bright active blue
            } else if is_stepped_target(note) {
                vg::Color::from_rgb(0x6a, 0x9a, 0xcc) // target stepped note
            } else if note >= start_note && note <= end_note {
                vg::Color::from_rgb(0x75, 0x7c, 0x88) // span between steps
            } else {
                vg::Color::from_rgb(0x88, 0x88, 0x88) // muted gray
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

                let color = if Some(note) == audition_note {
                    vg::Color::from_rgb(0x00, 0xb0, 0xff) // bright audition cyan/blue
                } else if note == current_note {
                    vg::Color::from_rgb(0x4a, 0x9e, 0xff)
                } else if is_stepped_target(note) {
                    vg::Color::from_rgb(0x2a, 0x4a, 0x77) // target stepped note
                } else if note >= start_note && note <= end_note {
                    vg::Color::from_rgb(0x22, 0x28, 0x38)
                } else {
                    vg::Color::from_rgb(0x22, 0x22, 0x2a)
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
    VStack::new(cx, |cx| {
        KeyboardView::new(cx)
            .height(Pixels(48.0))
            .width(Stretch(1.0))
            .corner_radius(Pixels(3.0))
            .cursor(CursorIcon::Hand);

        HStack::new(cx, |cx| {
            Label::new(
                cx,
                "Left-click: Audition · Right-click: Set Start · Shift-click: Set End",
            )
            .font_size(10.0)
            .color(Color::from("#555566"))
            .width(Stretch(1.0))
            .alignment(Alignment::Center);
        })
        .height(Auto)
        .width(Stretch(1.0));
    })
    .height(Auto)
    .width(Stretch(1.0))
    .vertical_gap(Pixels(3.0));
}
