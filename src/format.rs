use crate::Error;
use std::time::{SystemTime, UNIX_EPOCH};

const PREFIX_LENGTH: usize = 8;
const BASE_HEADER_LENGTH: usize = 36;
const PASSWORD_INDEX_LENGTH: usize = 8;
pub(crate) const SALT_LENGTH: usize = 24;
pub(crate) const DEFAULT_PASSWORD_INDEX: u64 = 2;
const AES_BLOCK_LENGTH: usize = 16;
const DOTNET_UNIX_EPOCH_UTC: u64 = 5_233_041_986_427_387_904;
const DOTNET_TICKS_PER_MS: u64 = 10_000;

/// Versioned metadata stored before the encryption parameters.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Header {
    /// Serialized header format version.
    pub version: u32,
    /// Counter incremented each time the game writes the save.
    pub save_count: u32,
    /// Save-specific raw GUID bytes.
    pub guid: [u8; 16],
    /// Save-specific field whose meaning is not yet known.
    pub unknown: [u8; 8],
    /// Binary .NET UTC `DateTime` value.
    pub timestamp: u64,
    /// Version-specific bytes following the known header fields.
    pub extension: Vec<u8>,
}

impl Header {
    /// Creates the default version 2 header used when no template is supplied.
    pub fn new_v2() -> Result<Self, Error> {
        Ok(Self {
            version: 2,
            save_count: 0,
            guid: [0; 16],
            unknown: [0; 8],
            timestamp: current_dotnet_timestamp()?,
            extension: vec![0],
        })
    }

    /// Returns the serialized header payload length, excluding its 8-byte prefix.
    pub fn encoded_length(&self) -> usize {
        BASE_HEADER_LENGTH + self.extension.len()
    }

    /// Increments the save counter and updates the timestamp.
    pub fn advance(&mut self) -> Result<(), Error> {
        self.save_count = self
            .save_count
            .checked_add(1)
            .ok_or(Error::SaveCountOverflow)?;
        self.timestamp = current_dotnet_timestamp()?;
        Ok(())
    }

    fn parse(version: u32, bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < BASE_HEADER_LENGTH {
            return Err(Error::HeaderTooShort(bytes.len()));
        }

        Ok(Self {
            version,
            save_count: read_u32(bytes, 0, "save count")?,
            guid: read_array(bytes, 4, "GUID")?,
            unknown: read_array(bytes, 20, "unknown header field")?,
            timestamp: read_u64(bytes, 28, "timestamp")?,
            extension: bytes[BASE_HEADER_LENGTH..].to_vec(),
        })
    }

    fn write_to(&self, output: &mut Vec<u8>) -> Result<(), Error> {
        let length = u32::try_from(self.encoded_length()).map_err(|_| Error::HeaderTooLarge)?;
        output.extend_from_slice(&self.version.to_le_bytes());
        output.extend_from_slice(&length.to_le_bytes());
        output.extend_from_slice(&self.save_count.to_le_bytes());
        output.extend_from_slice(&self.guid);
        output.extend_from_slice(&self.unknown);
        output.extend_from_slice(&self.timestamp.to_le_bytes());
        output.extend_from_slice(&self.extension);
        Ok(())
    }
}

/// Parsed Ninja Kiwi save container.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SaveFile {
    /// Versioned save metadata.
    pub header: Header,
    /// Unknown value that I called password index in the original `monke` implementation. The meaning is lost to time.
    pub password_index: u64,
    /// Salt used for PBKDF2 key derivation.
    pub salt: [u8; SALT_LENGTH],
    /// Zlib-compressed, AES-128-CBC encrypted payload.
    pub ciphertext: Vec<u8>,
}

impl SaveFile {
    /// Parses a complete encoded save file.
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        require_length(bytes, PREFIX_LENGTH, "header prefix")?;

        let version = read_u32(bytes, 0, "header version")?;
        let header_length = usize::try_from(read_u32(bytes, 4, "header length")?)
            .map_err(|_| Error::HeaderTooLarge)?;
        let header_end = PREFIX_LENGTH
            .checked_add(header_length)
            .ok_or(Error::HeaderTooLarge)?;
        require_length(bytes, header_end, "header")?;

        let metadata_end = header_end
            .checked_add(PASSWORD_INDEX_LENGTH + SALT_LENGTH)
            .ok_or(Error::HeaderTooLarge)?;
        require_length(bytes, metadata_end, "encryption metadata")?;

        let header = Header::parse(version, &bytes[PREFIX_LENGTH..header_end])?;
        let password_index = read_u64(bytes, header_end, "password index")?;
        let salt = read_array(bytes, header_end + PASSWORD_INDEX_LENGTH, "salt")?;
        let ciphertext = bytes[metadata_end..].to_vec();

        if ciphertext.is_empty() || !ciphertext.len().is_multiple_of(AES_BLOCK_LENGTH) {
            return Err(Error::InvalidCiphertextLength(ciphertext.len()));
        }

        Ok(Self {
            header,
            password_index,
            salt,
            ciphertext,
        })
    }

    /// Serializes the save file without changing its metadata or ciphertext.
    pub fn to_bytes(&self) -> Result<Vec<u8>, Error> {
        if self.ciphertext.is_empty() || !self.ciphertext.len().is_multiple_of(AES_BLOCK_LENGTH) {
            return Err(Error::InvalidCiphertextLength(self.ciphertext.len()));
        }

        let capacity = PREFIX_LENGTH
            + self.header.encoded_length()
            + PASSWORD_INDEX_LENGTH
            + SALT_LENGTH
            + self.ciphertext.len();
        let mut output = Vec::with_capacity(capacity);
        self.header.write_to(&mut output)?;
        output.extend_from_slice(&self.password_index.to_le_bytes());
        output.extend_from_slice(&self.salt);
        output.extend_from_slice(&self.ciphertext);
        Ok(output)
    }
}

fn current_dotnet_timestamp() -> Result<u64, Error> {
    let elapsed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::InvalidSystemTime)?;
    let ticks = u64::try_from(elapsed.as_millis())
        .map_err(|_| Error::InvalidSystemTime)?
        .checked_mul(DOTNET_TICKS_PER_MS)
        .ok_or(Error::InvalidSystemTime)?;
    DOTNET_UNIX_EPOCH_UTC
        .checked_add(ticks)
        .ok_or(Error::InvalidSystemTime)
}

fn read_u32(bytes: &[u8], offset: usize, section: &'static str) -> Result<u32, Error> {
    Ok(u32::from_le_bytes(read_array(bytes, offset, section)?))
}

fn read_u64(bytes: &[u8], offset: usize, section: &'static str) -> Result<u64, Error> {
    Ok(u64::from_le_bytes(read_array(bytes, offset, section)?))
}

fn read_array<const LENGTH: usize>(
    bytes: &[u8],
    offset: usize,
    section: &'static str,
) -> Result<[u8; LENGTH], Error> {
    let end = offset.checked_add(LENGTH).ok_or(Error::HeaderTooLarge)?;
    let source = bytes.get(offset..end).ok_or(Error::Truncated {
        section,
        expected: end,
        actual: bytes.len(),
    })?;
    let mut output = [0; LENGTH];
    output.copy_from_slice(source);
    Ok(output)
}

fn require_length(bytes: &[u8], expected: usize, section: &'static str) -> Result<(), Error> {
    if bytes.len() < expected {
        return Err(Error::Truncated {
            section,
            expected,
            actual: bytes.len(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn save_with_header(header: Header) -> SaveFile {
        SaveFile {
            header,
            password_index: 2,
            salt: [0x55; SALT_LENGTH],
            ciphertext: vec![0xaa; AES_BLOCK_LENGTH],
        }
    }

    #[test]
    fn parses_v1_header() {
        let original = save_with_header(Header {
            version: 1,
            save_count: 7,
            guid: [0x11; 16],
            unknown: [0x22; 8],
            timestamp: 0x48de_f44e_3623_2b6a,
            extension: Vec::new(),
        });

        let bytes = original.to_bytes().unwrap();
        assert_eq!(&bytes[..8], &[1, 0, 0, 0, 36, 0, 0, 0]);
        assert_eq!(bytes.len(), 76 + AES_BLOCK_LENGTH);
        assert_eq!(SaveFile::parse(&bytes).unwrap(), original);
    }

    #[test]
    fn parses_v2_header_and_preserves_extension() {
        let original = save_with_header(Header {
            version: 2,
            save_count: 8,
            guid: [0x33; 16],
            unknown: [0x44; 8],
            timestamp: 0x48de_f44e_3623_2b6a,
            extension: vec![0, 9, 8],
        });

        let bytes = original.to_bytes().unwrap();
        assert_eq!(&bytes[..8], &[2, 0, 0, 0, 39, 0, 0, 0]);
        assert_eq!(bytes.len(), 79 + AES_BLOCK_LENGTH);
        assert_eq!(SaveFile::parse(&bytes).unwrap(), original);
    }

    #[test]
    fn rejects_short_header_payload() {
        let mut bytes = vec![0; 8 + 35 + PASSWORD_INDEX_LENGTH + SALT_LENGTH + 16];
        bytes[4..8].copy_from_slice(&35_u32.to_le_bytes());
        assert!(matches!(
            SaveFile::parse(&bytes),
            Err(Error::HeaderTooShort(35))
        ));
    }

    #[test]
    fn rejects_truncated_encryption_metadata() {
        let mut bytes = vec![0; 8 + BASE_HEADER_LENGTH];
        bytes[4..8].copy_from_slice(&(BASE_HEADER_LENGTH as u32).to_le_bytes());
        assert!(matches!(
            SaveFile::parse(&bytes),
            Err(Error::Truncated {
                section: "encryption metadata",
                ..
            })
        ));
    }

    #[test]
    fn rejects_misaligned_ciphertext() {
        let mut bytes = save_with_header(Header {
            version: 1,
            save_count: 0,
            guid: [0; 16],
            unknown: [0; 8],
            timestamp: 0,
            extension: Vec::new(),
        })
        .to_bytes()
        .unwrap();
        bytes.push(0);

        assert!(matches!(
            SaveFile::parse(&bytes),
            Err(Error::InvalidCiphertextLength(17))
        ));
    }
}
