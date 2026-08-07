use crate::Error;
use aes::Aes128;
use aes::cipher::{BlockModeDecrypt, BlockModeEncrypt, KeyIvInit, block_padding::Pkcs7};
use cbc::{Decryptor, Encryptor};
use flate2::{Compress, Compression, read::ZlibDecoder, write::ZlibEncoder};
use pbkdf2::pbkdf2_hmac_array;
use sha1::Sha1;
use std::io::{Read, Write};

const PBKDF2_ITERATIONS: u32 = 10;
const KEY_LENGTH: usize = 16;
const IV_LENGTH: usize = 16;
const DERIVED_LENGTH: usize = KEY_LENGTH + IV_LENGTH;

// these are the sane constants used for packing only
const COMPRESSION_LEVEL: u32 = 3;
const COMPRESSION_WINDOW_BITS: u8 = 15;

type Aes128CbcEncryptor = Encryptor<Aes128>;
type Aes128CbcDecryptor = Decryptor<Aes128>;

pub(crate) fn encrypt(compressed: &[u8], password: &str, salt: &[u8; 24]) -> Vec<u8> {
    let (key, iv) = derive_key_and_iv(password, salt);
    Aes128CbcEncryptor::new((&key).into(), (&iv).into()).encrypt_padded_vec::<Pkcs7>(compressed)
}

pub(crate) fn decrypt(
    ciphertext: &[u8],
    password: &str,
    salt: &[u8; 24],
) -> Result<Vec<u8>, Error> {
    let (key, iv) = derive_key_and_iv(password, salt);
    Aes128CbcDecryptor::new((&key).into(), (&iv).into())
        .decrypt_padded_vec::<Pkcs7>(ciphertext)
        .map_err(|_| Error::InvalidPadding)
}

pub(crate) fn compress(contents: &[u8]) -> Result<Vec<u8>, Error> {
    let compressor = Compress::new_with_window_bits(
        Compression::new(COMPRESSION_LEVEL),
        true,
        COMPRESSION_WINDOW_BITS,
    );
    let mut encoder = ZlibEncoder::new_with_compress(Vec::new(), compressor);
    encoder.write_all(contents).map_err(Error::Zlib)?;
    encoder.finish().map_err(Error::Zlib)
}

pub(crate) fn decompress(compressed: &[u8]) -> Result<Vec<u8>, Error> {
    let mut decoder = ZlibDecoder::new(compressed);
    let mut contents = Vec::new();
    decoder.read_to_end(&mut contents).map_err(Error::Zlib)?;
    Ok(contents)
}

fn derive_key_and_iv(password: &str, salt: &[u8; 24]) -> ([u8; 16], [u8; 16]) {
    let derived =
        pbkdf2_hmac_array::<Sha1, DERIVED_LENGTH>(password.as_bytes(), salt, PBKDF2_ITERATIONS);
    let mut key = [0; KEY_LENGTH];
    let mut iv = [0; IV_LENGTH];
    key.copy_from_slice(&derived[IV_LENGTH..]);
    iv.copy_from_slice(&derived[..IV_LENGTH]);
    (key, iv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crypto_and_compression_round_trip() {
        let contents = b"\xef\xbb\xbf{\"name\":\"monke\",\"version\":2}";
        let salt = [0x42; 24];

        let compressed = compress(contents).unwrap();
        let encrypted = encrypt(&compressed, "11", &salt);
        let decrypted = decrypt(&encrypted, "11", &salt).unwrap();

        assert_eq!(decompress(&decrypted).unwrap(), contents);
    }

    #[test]
    fn compression_uses_the_game_window_size() {
        assert_eq!(compress(b"test").unwrap()[0], 0x78);
    }

    #[test]
    fn rejects_wrong_password() {
        let salt = [0x42; 24];
        let encrypted = encrypt(&compress(b"test").unwrap(), "11", &salt);
        assert!(matches!(
            decrypt(&encrypted, "wrong", &salt),
            Err(Error::InvalidPadding)
        ));
    }

    #[test]
    fn matches_openssl_vector() {
        let salt = [
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
        ];
        let expected = [
            0xdb, 0x8a, 0xf7, 0xf5, 0xac, 0x13, 0x18, 0x8d, 0xd2, 0x1a, 0xc3, 0xb0, 0x27, 0xdc,
            0xf3, 0x4e, 0x39, 0x1a, 0x5a, 0xcd, 0x93, 0xe6, 0x17, 0x8c, 0xb6, 0xd1, 0xb9, 0xe4,
            0x45, 0x61, 0x3b, 0x62,
        ];

        assert_eq!(encrypt(b"known compressed bytes", "11", &salt), expected);
    }
}
