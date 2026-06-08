//! UU-encoding helper for embedding binary font and graphic data

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
    vec::Vec,
};

/// Helper function to UU-encode binary data
pub(super) fn uuencode_data(filename: &str, data: &[u8]) -> Vec<String> {
    let mut lines = Vec::new();

    // Add UU-encode header
    lines.push(format!("begin 644 {filename}"));

    // Process data in 45-byte chunks (standard UU-encoding)
    for chunk in data.chunks(45) {
        let mut encoded = String::new();

        // Length character
        encoded.push((32 + chunk.len()) as u8 as char);

        // Encode groups of 3 bytes into 4 characters
        for group in chunk.chunks(3) {
            let mut bytes = [0u8; 3];
            for (i, &byte) in group.iter().enumerate() {
                bytes[i] = byte;
            }

            // Convert 3 bytes to 4 UU-encoded characters
            encoded.push((32 + (bytes[0] >> 2)) as char);
            encoded.push((32 + (((bytes[0] & 0x03) << 4) | (bytes[1] >> 4))) as char);

            if group.len() > 1 {
                encoded.push((32 + (((bytes[1] & 0x0F) << 2) | (bytes[2] >> 6))) as char);
            } else {
                encoded.push((32 + ((bytes[1] & 0x0F) << 2)) as char);
            }

            if group.len() > 2 {
                encoded.push((32 + (bytes[2] & 0x3F)) as char);
            } else if group.len() > 1 {
                encoded.push(' ');
            }
        }

        lines.push(encoded);
    }

    // Add UU-encode footer
    lines.push("`".to_string());
    lines.push("end".to_string());

    lines
}
