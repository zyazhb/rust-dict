use std::time::{Duration, Instant};

use eframe::egui::{self, vec2, ViewportCommand, WindowLevel};

use super::{FloatState, UiMode, EXPANDED_SIZE, ICON_SIZE};

const ANIM_DURATION: Duration = Duration::from_millis(200);
const ICON_SIZE_EPS: f32 = 6.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatAnimTarget {
    Collapsed,
    Expanded,
}

#[derive(Debug, Clone)]
pub struct FloatResizeAnim {
    pub(crate) from: egui::Vec2,
    pub(crate) to: egui::Vec2,
    pub(crate) started: Instant,
    pub(crate) target: FloatAnimTarget,
}

impl FloatResizeAnim {
    pub fn new(from: egui::Vec2, to: egui::Vec2, target: FloatAnimTarget) -> Self {
        Self {
            from,
            to,
            started: Instant::now(),
            target,
        }
    }
}

fn ease_out_cubic(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

pub fn icon_layout_rect(panel: egui::Rect) -> egui::Rect {
    if panel.width() > ICON_SIZE + ICON_SIZE_EPS || panel.height() > ICON_SIZE + ICON_SIZE_EPS {
        egui::Rect::from_center_size(panel.center(), egui::vec2(ICON_SIZE, ICON_SIZE))
    } else {
        panel
    }
}

pub fn needs_icon_settle(ctx: &egui::Context) -> bool {
    let size = viewport_inner_size(ctx, collapsed_fallback_size());
    size.x > ICON_SIZE + ICON_SIZE_EPS || size.y > ICON_SIZE + ICON_SIZE_EPS
}

pub fn viewport_inner_size(ctx: &egui::Context, fallback: egui::Vec2) -> egui::Vec2 {
    ctx.input(|input| {
        input
            .viewport()
            .inner_rect
            .map(|rect| rect.size())
            .unwrap_or(fallback)
    })
}

/// Drive window resize; returns `true` while the animation is still running.
pub fn tick(ctx: &egui::Context, anim: &mut FloatResizeAnim) -> bool {
    let elapsed = anim.started.elapsed();
    let t = (elapsed.as_secs_f32() / ANIM_DURATION.as_secs_f32()).clamp(0.0, 1.0);
    let eased = ease_out_cubic(t);
    let size = anim.from + (anim.to - anim.from) * eased;

    apply_float_chrome(ctx, anim.target);
    ctx.send_viewport_cmd(ViewportCommand::InnerSize(size));
    ctx.send_viewport_cmd(ViewportCommand::MinInnerSize(size));
    let max_side = anim.from.max_elem().max(anim.to.max_elem());
    ctx.send_viewport_cmd(ViewportCommand::MaxInnerSize(vec2(max_side, max_side)));

    if t < 1.0 {
        ctx.request_repaint_after(Duration::from_millis(16));
        return true;
    }

    // OS/winit may apply InnerSize a frame later — keep animating UI until viewport catches up.
    if anim.target == FloatAnimTarget::Collapsed && needs_icon_settle(ctx) {
        ctx.send_viewport_cmd(ViewportCommand::InnerSize(collapsed_fallback_size()));
        ctx.request_repaint_after(Duration::from_millis(16));
        return true;
    }

    false
}

fn apply_float_chrome(ctx: &egui::Context, target: FloatAnimTarget) {
    ctx.send_viewport_cmd(ViewportCommand::WindowLevel(WindowLevel::AlwaysOnTop));
    ctx.send_viewport_cmd(ViewportCommand::Decorations(false));
    match target {
        FloatAnimTarget::Collapsed => {
            ctx.send_viewport_cmd(ViewportCommand::Resizable(false));
        }
        FloatAnimTarget::Expanded => {
            ctx.send_viewport_cmd(ViewportCommand::Resizable(true));
        }
    }
}

pub fn collapsed_fallback_size() -> egui::Vec2 {
    vec2(ICON_SIZE, ICON_SIZE)
}

pub fn expanded_fallback_size() -> egui::Vec2 {
    EXPANDED_SIZE
}

pub fn ui_mode_for_target(target: FloatAnimTarget) -> UiMode {
    match target {
        FloatAnimTarget::Collapsed => UiMode::Float(FloatState::Collapsed),
        FloatAnimTarget::Expanded => UiMode::Float(FloatState::Expanded),
    }
}
