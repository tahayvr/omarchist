use gpui::{Hsla, Rgba, rgb};

// Parse a #RRGGBB hex string into GPUI's Hsla type.
pub fn hex_to_hsla(hex: &str) -> Option<Hsla> {
    let hex = hex.trim_start_matches('#');
    // Byte offsets below need ASCII; a hand-edited file can hold anything.
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some(rgb(u32::from_be_bytes([0, r, g, b])).into())
}

/// The opaque `#rrggbb` of a picker value. The picker's `to_hex` appends an
/// alpha byte (`#rrggbbaa`) when the alpha slider is below 100%, and none
/// of Omarchy's consumers (`colors.toml` templates, btop, Plymouth) accept
/// it, so every write from a picker goes through here.
pub fn hex6(hex: &str) -> String {
    let body = hex.trim_start_matches('#');
    let body = body.get(..6).unwrap_or(body);
    format!("#{}", body.to_lowercase())
}

/// The first color in a Hyprland color value: `rgba(rrggbbaa)`,
/// `rgb(rrggbb)` or `0xaarrggbb`, alone or at the head of a gradient
/// such as `rgba(26a269ee) rgba(2ec27eee) 45deg`.
pub fn hypr_color(value: &str) -> Option<Hsla> {
    let (_, text) = first_hypr_color(value)?;
    let (r, g, b, a) = if let Some(body) = text.strip_prefix("rgba(") {
        let h = body.trim_end_matches(')');
        (
            byte(h, 0)?,
            byte(h, 2)?,
            byte(h, 4)?,
            byte(h, 6).unwrap_or(255),
        )
    } else if let Some(body) = text.strip_prefix("rgb(") {
        let h = body.trim_end_matches(')');
        (byte(h, 0)?, byte(h, 2)?, byte(h, 4)?, 255)
    } else {
        let h = text.strip_prefix("0x")?;
        (byte(h, 2)?, byte(h, 4)?, byte(h, 6)?, byte(h, 0)?)
    };
    Some(
        Rgba {
            r: r as f32 / 255.,
            g: g as f32 / 255.,
            b: b as f32 / 255.,
            a: a as f32 / 255.,
        }
        .into(),
    )
}

/// A picker value as Hyprland writes a color: `rgb(rrggbb)` when opaque,
/// `rgba(rrggbbaa)` otherwise.
pub fn hypr_color_text(color: Hsla) -> String {
    let rgba = color.to_rgb();
    let channel = |v: f32| (v.clamp(0., 1.) * 255.).round() as u8;
    let (r, g, b, a) = (
        channel(rgba.r),
        channel(rgba.g),
        channel(rgba.b),
        channel(rgba.a),
    );
    if a == 255 {
        format!("rgb({r:02x}{g:02x}{b:02x})")
    } else {
        format!("rgba({r:02x}{g:02x}{b:02x}{a:02x})")
    }
}

/// `value` with its first color replaced by `color`, so a gradient keeps
/// its other stops and its angle; a value without a color becomes `color`.
pub fn with_first_hypr_color(value: &str, color: &str) -> String {
    match first_hypr_color(value) {
        Some((start, text)) => {
            format!("{}{color}{}", &value[..start], &value[start + text.len()..])
        }
        None => color.to_string(),
    }
}

/// Where the first color of a Hyprland value starts, and its text.
fn first_hypr_color(value: &str) -> Option<(usize, &str)> {
    let paren = value
        .match_indices("rgb")
        .filter_map(|(start, _)| {
            let rest = &value[start..];
            let open = rest.find('(')?;
            if !rest[..open].eq_ignore_ascii_case("rgba")
                && !rest[..open].eq_ignore_ascii_case("rgb")
            {
                return None;
            }
            let close = rest.find(')')?;
            Some((start, &rest[..=close]))
        })
        .next();
    let hex = value.match_indices("0x").find_map(|(start, _)| {
        let rest = &value[start..];
        let digits = rest[2..]
            .bytes()
            .take_while(|b| b.is_ascii_hexdigit())
            .count();
        (digits == 8).then_some((start, &rest[..10]))
    });
    match (paren, hex) {
        (Some(p), Some(h)) => Some(if p.0 <= h.0 { p } else { h }),
        (p, h) => p.or(h),
    }
}

fn byte(hex: &str, at: usize) -> Option<u8> {
    u8::from_str_radix(hex.get(at..at + 2)?, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::{hex_to_hsla, hex6, hypr_color, hypr_color_text, with_first_hypr_color};

    #[test]
    fn hyprland_colors_are_read_in_every_form() {
        let c = hypr_color("rgba(26a269ee)").unwrap();
        assert_eq!(hypr_color_text(c), "rgba(26a269ee)");
        assert_eq!(
            hypr_color_text(hypr_color("rgb(26A269)").unwrap()),
            "rgb(26a269)"
        );
        assert_eq!(
            hypr_color_text(hypr_color("0xee26a269").unwrap()),
            "rgba(26a269ee)"
        );
        assert_eq!(
            hypr_color_text(hypr_color("rgba(26a269ee) rgba(2ec27eee) 45deg").unwrap()),
            "rgba(26a269ee)"
        );
        assert!(hypr_color("").is_none());
        assert!(hypr_color("$accent").is_none());
    }

    #[test]
    fn a_picked_color_replaces_only_the_first_stop() {
        assert_eq!(
            with_first_hypr_color("rgba(26a269ee) rgba(2ec27eee) 45deg", "rgb(ffffff)"),
            "rgb(ffffff) rgba(2ec27eee) 45deg"
        );
        assert_eq!(with_first_hypr_color("", "rgb(ffffff)"), "rgb(ffffff)");
        assert_eq!(
            with_first_hypr_color("0xee26a269", "rgb(ffffff)"),
            "rgb(ffffff)"
        );
    }

    #[test]
    fn hex6_drops_alpha_and_lowercases() {
        assert_eq!(hex6("#0F0F19CC"), "#0f0f19");
        assert_eq!(hex6("#ABCDEF"), "#abcdef");
        assert_eq!(hex6("abc"), "#abc");
    }

    #[test]
    fn hex_to_hsla_rejects_non_ascii_without_panicking() {
        assert!(hex_to_hsla("a€aa").is_none());
        assert!(hex_to_hsla("#éa€").is_none());
        assert!(hex_to_hsla("#12345g").is_none());
        assert!(hex_to_hsla("#0f0f19").is_some());
    }
}
