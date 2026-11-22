use crate::themes_gen::BUILT_IN;
use crate::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb(pub u8, pub u8, pub u8);

impl Rgb {
  pub fn hex(s: &str) -> Option<Rgb> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
      return None;
    }
    let v = u32::from_str_radix(s, 16).ok()?;
    Some(Rgb((v >> 16) as u8, (v >> 8) as u8, v as u8))
  }

  pub fn mix(self, tint: Rgb, amount: f32) -> Rgb {
    let m = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * amount).round() as u8;
    Rgb(m(self.0, tint.0), m(self.1, tint.1), m(self.2, tint.2))
  }

  pub fn argb(self) -> u32 {
    0xff00_0000 | (self.0 as u32) << 16 | (self.1 as u32) << 8 | self.2 as u32
  }

  pub fn luminance(self) -> f32 {
    (self.0 as f32 * 0.2126 + self.1 as f32 * 0.7152 + self.2 as f32 * 0.0722) / 255.0
  }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
  pub name: String,
  pub background: Rgb,
  pub foreground: Rgb,
  pub accent: Rgb,
  pub dark: bool,
}

impl Theme {
  pub fn all() -> Vec<Theme> {
    BUILT_IN
      .iter()
      .map(|(name, bg, fg, ac, dark)| Theme {
        name: name.to_string(),
        background: Rgb::hex(bg).expect("valid background"),
        foreground: Rgb::hex(fg).expect("valid foreground"),
        accent: Rgb::hex(ac).expect("valid accent"),
        dark: *dark,
      })
      .collect()
  }

  pub fn named(name: &str) -> Theme {
    let all = Theme::all();
    all
      .iter()
      .find(|t| t.name.eq_ignore_ascii_case(name))
      .or_else(|| all.iter().find(|t| t.name == "Tokyo Night"))
      .cloned()
      .expect("Tokyo Night is built in")
  }

  pub fn from_omarchy_colors(name: impl Into<String>, source: &str) -> Result<Theme> {
    #[derive(Deserialize)]
    struct Colors {
      mode: Option<String>,
      background: String,
      foreground: String,
      accent: Option<String>,
      blue: Option<String>,
    }

    let colors: Colors = toml::from_str(source)?;
    let accent = colors
      .accent
      .as_deref()
      .or(colors.blue.as_deref())
      .and_then(Rgb::hex)
      .unwrap_or_else(|| Rgb::hex("#7aa2f7").expect("valid fallback"));

    let background = Rgb::hex(&colors.background)
      .ok_or_else(|| crate::Error::Config("invalid background color".into()))?;
    let foreground = Rgb::hex(&colors.foreground)
      .ok_or_else(|| crate::Error::Config("invalid foreground color".into()))?;

    Ok(Theme {
      name: name.into(),
      background,
      foreground,
      accent,
      dark: colors.mode.as_deref() != Some("light"),
    })
  }

  pub fn from_omarchy_current() -> Result<Theme> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/abrahamukachi".into());
    let path = format!("{home}/.local/state/omarchy/current/theme/colors.toml");
    let source = std::fs::read_to_string(path)?;
    Self::from_omarchy_colors("Omarchy", &source)
  }

  pub fn mix(&self, amount: f32) -> Rgb {
    self.background.mix(self.foreground, amount)
  }

  pub fn surface(&self) -> Rgb {
    self.mix(0.05)
  }

  pub fn hover(&self) -> Rgb {
    self.mix(0.14)
  }

  pub fn border(&self) -> Rgb {
    self.mix(0.16)
  }

  pub fn muted(&self) -> Rgb {
    self.mix(0.5)
  }

  pub fn accent_text(&self) -> Rgb {
    if self.accent.luminance() > 0.55 {
      Rgb(4, 12, 20)
    } else {
      Rgb(242, 247, 255)
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn parses_omarchy_colors() {
    let source = "mode = \"dark\"\nbackground = \"#2c2525\"\nforeground = \"#e6d9db\"\naccent = \"#f38d70\"\n";
    let theme = Theme::from_omarchy_colors("Test", source).unwrap();
    assert_eq!(theme.background, Rgb(44, 37, 37));
    assert_eq!(theme.accent, Rgb(243, 141, 112));
    assert!(theme.dark);
  }

  #[test]
  fn mixes_towards_foreground() {
    let source = "background = \"#000000\"\nforeground = \"#ffffff\"\n";
    let theme = Theme::from_omarchy_colors("Test", source).unwrap();
    assert_eq!(theme.muted(), Rgb(128, 128, 128));
  }
}
