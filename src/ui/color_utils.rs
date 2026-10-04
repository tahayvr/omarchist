use gpui::{Hsla, rgb};

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

#[cfg(test)]
mod tests {
    use super::{hex_to_hsla, hex6};

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
