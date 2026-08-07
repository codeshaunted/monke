//! Parser and codec for Ninja Kiwi save files.

mod crypto;
mod format;

pub use format::{Header, SaveFile};
use std::io;
use thiserror::Error;

/// Errors produced while parsing, packing, or unpacking a save file.
#[derive(Debug, Error)]
pub enum Error {
    #[error(
        "file is too short to contain {section}: expected at least {expected} bytes, found {actual}"
    )]
    Truncated {
        section: &'static str,
        expected: usize,
        actual: usize,
    },

    #[error("header payload is too short: expected at least 36 bytes, found {0}")]
    HeaderTooShort(usize),

    #[error("header length does not fit in this process")]
    HeaderTooLarge,

    #[error("ciphertext must be a non-empty multiple of 16 bytes, found {0} bytes")]
    InvalidCiphertextLength(usize),

    #[error("ciphertext has invalid PKCS#7 padding; the password may be wrong")]
    InvalidPadding,

    #[error("save count overflowed")]
    SaveCountOverflow,

    #[error("system clock is before the Unix epoch")]
    InvalidSystemTime,

    #[error("zlib operation failed: {0}")]
    Zlib(#[source] io::Error),
}

/// Decrypts and decompresses a parsed save file.
pub fn unpack(save: &SaveFile, password: &str) -> Result<Vec<u8>, Error> {
    let compressed = crypto::decrypt(&save.ciphertext, password, &save.salt)?;
    crypto::decompress(&compressed)
}

/// Compresses and encrypts save contents.
///
/// A template preserves format metadata and advances its save counter. Without
/// a template, this creates a version 2 header with default metadata.
pub fn pack(
    contents: &[u8],
    password: &str,
    template: Option<&SaveFile>,
) -> Result<SaveFile, Error> {
    let (mut header, password_index) = match template {
        Some(template) => (template.header.clone(), template.password_index),
        None => (Header::new_v2()?, format::DEFAULT_PASSWORD_INDEX),
    };

    if template.is_some() {
        header.advance()?;
    }

    let mut salt = [0; format::SALT_LENGTH];
    rand::fill(&mut salt);

    let compressed = crypto::compress(contents)?;
    let ciphertext = crypto::encrypt(&compressed, password, &salt);

    Ok(SaveFile {
        header,
        password_index,
        salt,
        ciphertext,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_without_template_creates_v2_save() {
        let contents = b"{\"version\":2}";
        let save = pack(contents, "11", None).unwrap();

        assert_eq!(save.header.version, 2);
        assert_eq!(save.header.encoded_length(), 37);
        assert_eq!(save.header.extension, [0]);
        assert_eq!(save.password_index, 2);
        assert_eq!(unpack(&save, "11").unwrap(), contents);
    }

    #[test]
    fn template_metadata_and_extensions_are_preserved() {
        let template = SaveFile {
            header: Header {
                version: 9,
                save_count: 41,
                guid: [0x11; 16],
                unknown: [0x22; 8],
                timestamp: 1,
                extension: vec![3, 4, 5],
            },
            password_index: 17,
            salt: [0x33; 24],
            ciphertext: vec![0x44; 16],
        };

        let packed = pack(b"{}", "11", Some(&template)).unwrap();

        assert_eq!(packed.header.version, template.header.version);
        assert_eq!(packed.header.save_count, 42);
        assert_eq!(packed.header.guid, template.header.guid);
        assert_eq!(packed.header.unknown, template.header.unknown);
        assert_eq!(packed.header.extension, template.header.extension);
        assert_eq!(packed.password_index, template.password_index);
        assert_ne!(packed.header.timestamp, template.header.timestamp);
    }

    #[test]
    fn overflowing_template_save_count_is_rejected() {
        let template = SaveFile {
            header: Header {
                version: 1,
                save_count: u32::MAX,
                guid: [0; 16],
                unknown: [0; 8],
                timestamp: 0,
                extension: Vec::new(),
            },
            password_index: 2,
            salt: [0; 24],
            ciphertext: vec![0; 16],
        };

        assert!(matches!(
            pack(b"{}", "11", Some(&template)),
            Err(Error::SaveCountOverflow)
        ));
    }
}
