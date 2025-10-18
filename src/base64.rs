//! Minimal base64 implementation for GitHub API content decoding
//! Implements only the features needed for this project

/// Minimal base64 decoder
pub struct Base64Decoder;

impl Base64Decoder {
    /// Decode base64 string to bytes
    pub fn decode(input: &str) -> Result<Vec<u8>, &'static str> {
        let mut input = input.trim().to_string();

        // Remove common base64 variants
        input = input.replace('\n', "");
        input = input.replace('\r', "");
        input = input.replace("=", "");

        if input.is_empty() {
            return Ok(Vec::new());
        }

        let mut result = Vec::new();
        let mut buffer = 0u32;
        let mut bits_left = 0;

        for c in input.chars() {
            let value = match c {
                'A'..='Z' => c as u32 - 'A' as u32,
                'a'..='z' => c as u32 - 'a' as u32 + 26,
                '0'..='9' => c as u32 - '0' as u32 + 52,
                '+' => 62,
                '/' => 63,
                _ => return Err("Invalid base64 character"),
            };

            buffer = (buffer << 6) | value;
            bits_left += 6;

            if bits_left >= 8 {
                bits_left -= 8;
                result.push((buffer >> bits_left) as u8);
                buffer &= (1 << bits_left) - 1;
            }
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_simple() {
        let encoded = "SGVsbG8gV29ybGQ=";
        let decoded = Base64Decoder::decode(encoded).unwrap();
        assert_eq!(decoded, b"Hello World");
    }

    #[test]
    fn test_decode_no_padding() {
        let encoded = "SGVsbG8gV29ybGQ";
        let decoded = Base64Decoder::decode(encoded).unwrap();
        assert_eq!(decoded, b"Hello World");
    }

    #[test]
    fn test_decode_empty() {
        let encoded = "";
        let decoded = Base64Decoder::decode(encoded).unwrap();
        assert_eq!(decoded, b"");
    }
}