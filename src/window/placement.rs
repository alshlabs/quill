use gpui_kit::*;

const CASCADE_OFFSET: f32 = 30.;

/// Must not run while the active window is being updated, such as from an action
/// dispatched on it: gpui takes that window out of the app meanwhile, so it reads
/// as missing and the new window opens at the default spot, over the old one.
pub fn main_window_placement(cx: &mut App) -> (Bounds<Pixels>, Option<DisplayId>) {
  let default = Bounds::new(point(px(120.), px(120.)), size(px(1460.), px(978.)));

  let Some((bounds, display)) = cx.active_window().and_then(|handle| {
    handle
      .update(cx, |_, window, cx| {
        (window.window_bounds().get_bounds(), window.display(cx))
      })
      .ok()
  }) else {
    return (default, None);
  };

  let offset = point(px(CASCADE_OFFSET), px(CASCADE_OFFSET));
  let mut cascaded = Bounds::new(bounds.origin + offset, bounds.size);
  if let Some(display) = &display {
    let visible = display.visible_bounds();
    if !cascaded.is_contained_within(&visible) {
      cascaded.origin = visible.origin + offset;
    }
  }

  (cascaded, display.map(|display| display.id()))
}
