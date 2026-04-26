use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ByteSize(pub u64);

impl ByteSize {
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

pub fn parse_size(s: &str) -> Result<u64, String> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return Err(format!("invalid size {s:?} (empty)"));
    }

    let split_idx = trimmed
        .find(|c: char| !(c.is_ascii_digit() || c == '.'))
        .unwrap_or(trimmed.len());
    let (num_part, unit_part) = trimmed.split_at(split_idx);
    if num_part.is_empty() {
        return Err(format!(
            "invalid size {s:?} (expected format: \"60mb\", \"100kb\", \"1gb\")"
        ));
    }

    let val: f64 = num_part
        .parse()
        .map_err(|_| format!("invalid size number {num_part:?}"))?;

    let unit = unit_part.trim().to_ascii_lowercase();
    let multiplier: f64 = match unit.as_str() {
        "tb" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        "gb" => 1024.0 * 1024.0 * 1024.0,
        "mb" | "" => 1024.0 * 1024.0,
        "kb" => 1024.0,
        "b" => 1.0,
        other => {
            return Err(format!(
                "invalid size unit {other:?} (expected b, kb, mb, gb, tb)"
            ));
        }
    };

    Ok((val * multiplier) as u64)
}

pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

impl<'de> Deserialize<'de> for ByteSize {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s = String::deserialize(d)?;
        parse_size(&s).map(ByteSize).map_err(de::Error::custom)
    }
}

impl Serialize for ByteSize {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format_size(self.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_units() {
        assert_eq!(parse_size("60mb").unwrap(), 60 * 1024 * 1024);
        assert_eq!(parse_size("1gb").unwrap(), 1024 * 1024 * 1024);
        assert_eq!(parse_size("100kb").unwrap(), 100 * 1024);
        assert_eq!(parse_size("1024b").unwrap(), 1024);
        assert_eq!(parse_size("2").unwrap(), 2 * 1024 * 1024);
        assert_eq!(parse_size("1.5gb").unwrap(), (1.5 * 1024.0 * 1024.0 * 1024.0) as u64);
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_size("").is_err());
        assert!(parse_size("abc").is_err());
        assert!(parse_size("10pb").is_err());
    }
}
