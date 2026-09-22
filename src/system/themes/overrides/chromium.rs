//! `chromium.theme`: the browser toolbar color as `R,G,B` in decimal, the
//! form Omarchy's browser policy reads.

/// The color as `#RRGGBB`, or `None` when the file is malformed.
pub fn to_hex(content: &str) -> Option<String> {
    let mut parts = content
        .trim()
        .split(',')
        .map(|p| p.trim().parse::<u8>().ok());
    let (r, g, b) = (parts.next()??, parts.next()??, parts.next()??);
    parts
        .next()
        .is_none()
        .then(|| format!("#{r:02X}{g:02X}{b:02X}"))
}

/// The file content for a `#RRGGBB` color.
pub fn from_hex(hex: &str) -> Option<String> {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() < 6 || !hex.is_char_boundary(6) {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some(format!("{},{},{}\n", channel(0)?, channel(2)?, channel(4)?))
}

#[cfg(test)]
mod tests {
    use super::{from_hex, to_hex};

    #[test]
    fn round_trips() {
        assert_eq!(to_hex("15,15,25\n").as_deref(), Some("#0F0F19"));
        assert_eq!(from_hex("#0F0F19").as_deref(), Some("15,15,25\n"));
        assert_eq!(from_hex("#0f0f19ff").as_deref(), Some("15,15,25\n"));
        assert_eq!(to_hex("garbage"), None);
        assert_eq!(to_hex("1,2,3,4"), None);
        assert_eq!(from_hex("#12"), None);
    }
}
