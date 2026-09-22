use std::collections::BTreeMap;

/// A theme's resolved colors, as `omarchy-theme-color --all` prints them:
/// the `colors.toml` keys plus Omarchy's aliases and derived shades.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Palette(BTreeMap<String, String>);

impl Palette {
    /// Parses `key<TAB>value` lines.
    pub fn parse(output: &str) -> Self {
        Self(
            output
                .lines()
                .filter_map(|line| line.split_once('\t'))
                .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
                .filter(|(key, _)| !key.is_empty())
                .collect(),
        )
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(String::as_str)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

/// Renders a template the way `omarchy-theme-set-templates` does. A token it
/// cannot resolve is left in place, as Omarchy's sed script leaves it.
pub fn render(template: &str, palette: &Palette) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find("{{") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            out.push_str(&rest[start..]);
            return out;
        };
        let token = &rest[start..start + 2 + end + 2];
        match resolve(after[..end].trim(), palette) {
            Some(value) => out.push_str(&value),
            None => out.push_str(token),
        }
        rest = &after[end + 2..];
    }

    out.push_str(rest);
    out
}

fn resolve(content: &str, palette: &Palette) -> Option<String> {
    let mut words = content.split_whitespace();
    let head = words.next()?;

    match head {
        "mix" | "mix_strip" | "mix_rgb" => {
            let start = palette.get(words.next()?)?;
            let end = palette.get(words.next()?)?;
            let amount = parse_amount(words.next()?)?;
            if words.next().is_some() {
                return None;
            }
            let mixed = mix(start, end, amount)?;
            Some(match head {
                "mix" => mixed,
                "mix_strip" => mixed.trim_start_matches('#').to_string(),
                _ => hex_to_rgb(&mixed)?,
            })
        }
        "hypr_gradient" | "gradient_start" | "shell_gradient" => {
            let key = words.next()?;
            let fallback = words.collect::<Vec<_>>().join(" ");
            let spec = resolve_theme_ref(key, &fallback, palette);
            let (colors, angle) = parse_gradient(&spec, palette);
            Some(match head {
                "hypr_gradient" => hypr_gradient(&spec, &colors, angle.as_deref()),
                "gradient_start" => color_to_shell_hex(colors.first().unwrap_or(&spec), palette),
                _ => shell_gradient(&spec, &colors, angle.as_deref()),
            })
        }
        key if words.next().is_none() => {
            if let Some(value) = palette.get(key) {
                return Some(value.to_string());
            }
            if let Some(value) = key.strip_suffix("_strip").and_then(|k| palette.get(k)) {
                return Some(value.trim_start_matches('#').to_string());
            }
            let value = palette.get(key.strip_suffix("_rgb")?)?;
            if is_hex6(value) {
                hex_to_rgb(value)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_hex6(value: &str) -> bool {
    value.len() == 7 && value.starts_with('#') && value[1..].chars().all(|c| c.is_ascii_hexdigit())
}

fn channels(hex: &str) -> Option<(u8, u8, u8)> {
    if !is_hex6(hex) {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some((channel(1)?, channel(3)?, channel(5)?))
}

fn hex_to_rgb(hex: &str) -> Option<String> {
    let (r, g, b) = channels(hex)?;
    Some(format!("{r},{g},{b}"))
}

// A fraction (0.30) or a percentage (30%); values above 1 are percentages.
fn parse_amount(raw: &str) -> Option<f64> {
    let amount = match raw.strip_suffix('%') {
        Some(percent) => percent.parse::<f64>().ok()? / 100.0,
        None => {
            let value = raw.parse::<f64>().ok()?;
            if value > 1.0 { value / 100.0 } else { value }
        }
    };
    Some(amount.clamp(0.0, 1.0))
}

fn mix(start: &str, end: &str, amount: f64) -> Option<String> {
    let (sr, sg, sb) = channels(start)?;
    let (er, eg, eb) = channels(end)?;
    let blend = |s: u8, e: u8| (s as f64 * (1.0 - amount) + e as f64 * amount + 0.5).floor() as u8;
    Some(format!(
        "#{:02x}{:02x}{:02x}",
        blend(sr, er),
        blend(sg, eg),
        blend(sb, eb)
    ))
}

fn resolve_theme_ref(key: &str, fallback: &str, palette: &Palette) -> String {
    if let Some(value) = palette.get(key) {
        value.to_string()
    } else if let Some(value) = palette.get(fallback).filter(|_| !fallback.is_empty()) {
        value.to_string()
    } else if !fallback.is_empty() {
        fallback.to_string()
    } else {
        key.to_string()
    }
}

fn is_angle(part: &str) -> bool {
    let Some(number) = part.strip_suffix("deg") else {
        return false;
    };
    let digits = number.strip_prefix('-').unwrap_or(number);
    let (whole, fraction) = match digits.split_once('.') {
        Some((whole, fraction)) => (whole, Some(fraction)),
        None => (digits, None),
    };
    !whole.is_empty()
        && whole.chars().all(|c| c.is_ascii_digit())
        && fraction.is_none_or(|f| !f.is_empty() && f.chars().all(|c| c.is_ascii_digit()))
}

fn parse_gradient(spec: &str, palette: &Palette) -> (Vec<String>, Option<String>) {
    let mut colors = Vec::new();
    let mut angle = None;
    for part in spec.split_whitespace() {
        if is_angle(part) {
            angle = Some(part.trim_end_matches("deg").to_string());
        } else {
            colors.push(palette.get(part).unwrap_or(part).to_string());
        }
    }
    (colors, angle)
}

fn hypr_gradient(spec: &str, colors: &[String], angle: Option<&str>) -> String {
    match colors {
        [] => format!("\"{spec}\""),
        [color] => format!("\"{color}\""),
        _ => {
            let list = colors
                .iter()
                .map(|c| format!(" \"{c}\""))
                .collect::<Vec<_>>()
                .join(",");
            let angle = angle.map(|a| format!(", angle = {a}")).unwrap_or_default();
            format!("{{ colors = {{{list} }}{angle} }}")
        }
    }
}

fn shell_gradient(spec: &str, colors: &[String], angle: Option<&str>) -> String {
    if colors.is_empty() {
        return spec.to_string();
    }
    let mut out = colors.join(" ");
    if let Some(angle) = angle {
        out.push_str(&format!(" {angle}deg"));
    }
    out
}

// Reduces any Hyprland color spelling to `#rrggbb`, as the shell expects.
fn color_to_shell_hex(color: &str, palette: &Palette) -> String {
    let color = palette.get(color.trim()).unwrap_or(color.trim());
    let is_hex = |s: &str| s.chars().all(|c| c.is_ascii_hexdigit());

    if let Some(hex) = color.strip_prefix('#')
        && (hex.len() == 6 || hex.len() == 8)
        && is_hex(hex)
    {
        return format!("#{}", &hex[..6]);
    }

    let lower = color.to_ascii_lowercase();
    if let Some(inner) = lower
        .strip_prefix("rgba(")
        .or_else(|| lower.strip_prefix("rgb("))
        .and_then(|s| s.strip_suffix(')'))
    {
        if (inner.len() == 6 || inner.len() == 8) && is_hex(inner) {
            return format!("#{}", &color[color.len() - 1 - inner.len()..][..6]);
        }
        let parts: Vec<&str> = inner.split(',').collect();
        if (3..=4).contains(&parts.len())
            && let [Ok(r), Ok(g), Ok(b)] = [0, 1, 2].map(|i| parts[i].parse::<u32>())
        {
            return format!("#{:02x}{:02x}{:02x}", r.min(255), g.min(255), b.min(255));
        }
    }

    if let Some(hex) = color.strip_prefix("0x")
        && hex.len() == 8
        && is_hex(hex)
    {
        return format!("#{}", &hex[2..]);
    }

    color.to_string()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::{Palette, render};
    use crate::system::omarchy_paths::themed_templates_dir;

    fn palette() -> Palette {
        Palette::parse(
            "accent\t#7aa2f7\nbackground\t#1a1b26\nforeground\t#a9b1d6\nred\t#f7768e\n\
             hyprland_active_border\trgba(33ccffee) rgba(00ff99ee) 45deg\n",
        )
    }

    #[test]
    fn plain_strip_and_rgb_tokens() {
        let p = palette();
        assert_eq!(render("{{ accent }}", &p), "#7aa2f7");
        assert_eq!(render("{{ accent_strip }}", &p), "7aa2f7");
        assert_eq!(render("{{ background_rgb }}", &p), "26,27,38");
        assert_eq!(render("x {{ unknown }} y", &p), "x {{ unknown }} y");
    }

    #[test]
    fn mix_matches_omarchy_rounding() {
        let p = palette();
        // awk: int(x * (1 - a) + y * a + 0.5)
        assert_eq!(render("{{ mix foreground background 34% }}", &p), "#787e9a");
        assert_eq!(
            render("{{ mix_strip foreground background 0.34 }}", &p),
            "787e9a"
        );
        assert_eq!(
            render("{{ mix_rgb foreground background 34 }}", &p),
            "120,126,154"
        );
    }

    #[test]
    fn gradients() {
        let p = palette();
        assert_eq!(
            render("{{ hypr_gradient hyprland_active_border accent }}", &p),
            "{ colors = { \"rgba(33ccffee)\", \"rgba(00ff99ee)\" }, angle = 45 }"
        );
        assert_eq!(
            render(
                "{{ hypr_gradient hyprland_inactive_border rgba(595959aa) }}",
                &p
            ),
            "\"rgba(595959aa)\""
        );
        assert_eq!(
            render("{{ shell_gradient hyprland_active_border accent }}", &p),
            "rgba(33ccffee) rgba(00ff99ee) 45deg"
        );
        assert_eq!(
            render("{{ gradient_start hyprland_active_border accent }}", &p),
            "#33ccff"
        );
        assert_eq!(
            render("{{ gradient_start hyprland_inactive_border accent }}", &p),
            "#7aa2f7"
        );
    }

    #[test]
    fn installed_templates_render_completely() {
        let Ok(output) = std::process::Command::new("omarchy-theme-color")
            .args(["--file"])
            .arg(crate::system::omarchy_paths::system_themes_dir().join("tokyo-night/colors.toml"))
            .arg("--all")
            .output()
        else {
            return;
        };
        let palette = Palette::parse(&String::from_utf8_lossy(&output.stdout));
        let Ok(entries) = fs::read_dir(themed_templates_dir()) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "tpl") {
                let rendered = render(&fs::read_to_string(&path).unwrap(), &palette);
                assert!(!rendered.contains("{{"), "{} left a token", path.display());
            }
        }
    }
}
