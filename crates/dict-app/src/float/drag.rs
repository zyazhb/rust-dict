//! Native window drag for frameless float windows (winit `drag_window` on macOS).

use eframe::egui::{Context, Response, ViewportCommand};

/// Start OS-native window drag when the user begins dragging this widget.
///
/// Uses [`ViewportCommand::StartDrag`] (winit `Window::drag_window`) — do not mix with
/// manual [`ViewportCommand::OuterPosition`] updates; that causes jitter on macOS.
pub fn handle_native_window_drag(ctx: &Context, response: &Response) {
    if response.drag_started() {
        ctx.send_viewport_cmd(ViewportCommand::Focus);
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
    }
}

/// True when the user released a click without meaningful movement (expand icon, etc.).
pub fn is_plain_click(response: &Response) -> bool {
    response.clicked() && response.drag_delta().length_sq() < 4.0
}
