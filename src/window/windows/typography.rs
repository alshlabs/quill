use gpui_kit::Font;
use gpui_kit::FontFallbacks;

pub const FONT_FAMILY: &str = "Arial";
pub const FONT_FALLBACKS: [&str; 3] = ["Segoe UI", "Segoe UI Emoji", "Segoe UI Symbol"];

pub fn app_font() -> Font {
  Font {
    family: FONT_FAMILY.into(),
    fallbacks: Some(FontFallbacks::from_fonts(
      FONT_FALLBACKS.map(String::from).to_vec(),
    )),
    ..Font::default()
  }
}
