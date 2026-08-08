//! Purpose:
//! Native, tar, and ZIP PHAR signature generation and inspection.
//!
//! Called from:
//! - PHAR signing APIs and their C ABI wrappers.
//!
//! Key details:
//! - Hash and OpenSSL signatures cover the exact family-specific signed byte range.

use super::*;

/// Appends PHP's raw-SHA1 PHAR signature trailer to `archive`.
pub(super) fn append_sha1_signature(archive: &mut Vec<u8>) {
    use sha1::{Digest, Sha1};

    let digest = Sha1::digest(&archive);
    archive.extend_from_slice(&digest);
    archive.extend_from_slice(&PHAR_SHA1_SIGNATURE_TYPE.to_le_bytes());
    archive.extend_from_slice(b"GBMB");
}

/// Returns the raw digest length for a PHP hash-based PHAR signature flag
/// (MD5=1, SHA1=2, SHA256=3, SHA512=4); `None` for non-hash flags.
pub(super) fn signature_digest_len(flags: u32) -> Option<usize> {
    match flags {
        1 => Some(16),
        2 => Some(20),
        3 => Some(32),
        4 => Some(64),
        _ => None,
    }
}

/// Returns the archive bytes with any trailing PHP signature trailer removed
/// (native PHAR `digest ++ LE32(flag) ++ "GBMB"`, or the OpenSSL variant
/// `sig ++ LE32(sig_len) ++ LE32(0x10) ++ "GBMB"`). Returns the input unchanged
/// when no recognized trailer is present.
pub(super) fn strip_signature_trailer(archive: &[u8]) -> &[u8] {
    let n = archive.len();
    if n < 8 || &archive[n - 4..] != b"GBMB" {
        return archive;
    }
    let flags = u32::from_le_bytes(archive[n - 8..n - 4].try_into().unwrap());
    if flags == PHAR_OPENSSL_SIGNATURE_TYPE {
        if n >= 12 {
            let sig_len = u32::from_le_bytes(archive[n - 12..n - 8].try_into().unwrap()) as usize;
            if let Some(total) = sig_len.checked_add(12) {
                if n >= total {
                    return &archive[..n - total];
                }
            }
        }
    } else if let Some(dlen) = signature_digest_len(flags) {
        let total = dlen + 8;
        if n >= total {
            return &archive[..n - total];
        }
    }
    archive
}

/// Computes the PKCS#1 v1.5 RSA-SHA1 signature of `data` with a PEM private key
/// (PKCS#8 or PKCS#1), matching PHP's `openssl_sign(..., OPENSSL_ALGO_SHA1)`.
pub(super) fn rsa_sha1_sign(data: &[u8], key_pem: &[u8]) -> Option<Vec<u8>> {
    use rsa::pkcs1::DecodeRsaPrivateKey;
    use rsa::pkcs8::DecodePrivateKey;
    use rsa::{Pkcs1v15Sign, RsaPrivateKey};
    use sha1::{Digest, Sha1};

    let pem = std::str::from_utf8(key_pem).ok()?;
    let key = RsaPrivateKey::from_pkcs8_pem(pem)
        .ok()
        .or_else(|| RsaPrivateKey::from_pkcs1_pem(pem).ok())?;
    let hashed = Sha1::digest(data);
    key.sign(Pkcs1v15Sign::new::<Sha1>(), &hashed).ok()
}

/// Computes a PHP-compatible signature over `data` for a signature `flag`: a raw
/// MD5/SHA1/SHA256/SHA512 digest (flags 1..=4) or an RSA-SHA1 OpenSSL signature
/// (flag 0x10, requiring the PEM `key`). Returns `None` for an unknown flag or a
/// missing/invalid key.
pub(super) fn compute_signature(flag: u32, key: Option<&[u8]>, data: &[u8]) -> Option<Vec<u8>> {
    use md5::Md5;
    use sha1::{Digest, Sha1};
    use sha2::{Sha256, Sha512};

    match flag {
        1 => Some(Md5::digest(data).to_vec()),
        2 => Some(Sha1::digest(data).to_vec()),
        3 => Some(Sha256::digest(data).to_vec()),
        4 => Some(Sha512::digest(data).to_vec()),
        PHAR_OPENSSL_SIGNATURE_TYPE => rsa_sha1_sign(data, key?),
        _ => None,
    }
}

/// Builds the `.phar/signature.bin` payload for a tar/zip phar:
/// `LE32(sig_flag) ++ LE32(sig_len) ++ signature`.
pub(super) fn signature_bin_payload(flag: u32, sig: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(8 + sig.len());
    out.extend_from_slice(&flag.to_le_bytes());
    out.extend_from_slice(&u32::try_from(sig.len()).ok()?.to_le_bytes());
    out.extend_from_slice(sig);
    Some(out)
}

/// Detects the archive family of `data` for signature operations: zip (PK magic),
/// tar (ustar magic at offset 257), or native PHAR (default). Returns `None` for a
/// gzip/bzip2-wrapped archive, where signature rewriting is not supported.
pub(super) fn signing_format(data: &[u8]) -> Option<ArchiveFormat> {
    if data.starts_with(&[0x50, 0x4b, 0x03, 0x04]) || data.starts_with(&[0x50, 0x4b, 0x05, 0x06]) {
        Some(ArchiveFormat::Zip)
    } else if data.get(257..262) == Some(b"ustar") {
        Some(ArchiveFormat::Tar)
    } else if data.starts_with(&[0x1f, 0x8b]) || data.starts_with(b"BZh") {
        None
    } else {
        Some(ArchiveFormat::NativePhar)
    }
}

/// Re-signs the phar at `path` with an OpenSSL (RSA-SHA1) signature. Native PHARs
/// gain a `sig ++ LE32(sig_len) ++ LE32(0x10) ++ "GBMB"` trailer; tar/zip phars
/// gain a `.phar/signature.bin` entry. The caller-supplied public key is what
/// verifiers use; PHP does not auto-write a `.pubkey` here either.
pub(super) fn sign_archive_openssl(path: &[u8], key_pem: &[u8]) -> Option<()> {
    let data = read_path(path)?;
    match signing_format(&data)? {
        ArchiveFormat::Zip => {
            let signed =
                sign_zip_archive(&parse_zip_archive(&data)?, PHAR_OPENSSL_SIGNATURE_TYPE, Some(key_pem))?;
            write_path(path, &signed)
        }
        ArchiveFormat::Tar => {
            let signed =
                sign_tar_archive(&parse_tar_archive(&data)?, PHAR_OPENSSL_SIGNATURE_TYPE, Some(key_pem))?;
            write_path(path, &signed)
        }
        ArchiveFormat::NativePhar => {
            let mut out = strip_signature_trailer(&data).to_vec();
            let sig = rsa_sha1_sign(&out, key_pem)?;
            out.extend_from_slice(&sig);
            out.extend_from_slice(&u32::try_from(sig.len()).ok()?.to_le_bytes());
            out.extend_from_slice(&PHAR_OPENSSL_SIGNATURE_TYPE.to_le_bytes());
            out.extend_from_slice(b"GBMB");
            write_path(path, &out)
        }
    }
}

/// Re-signs the phar at `path` with a hash-based signature (MD5/SHA1/SHA256/SHA512
/// per `algo` 1..=4). Native PHARs append `digest ++ LE32(algo) ++ "GBMB"`; tar/zip
/// phars gain a `.phar/signature.bin` entry.
pub(super) fn sign_archive_hash(path: &[u8], algo: u32) -> Option<()> {
    let data = read_path(path)?;
    match signing_format(&data)? {
        ArchiveFormat::Zip => {
            let signed = sign_zip_archive(&parse_zip_archive(&data)?, algo, None)?;
            write_path(path, &signed)
        }
        ArchiveFormat::Tar => {
            let signed = sign_tar_archive(&parse_tar_archive(&data)?, algo, None)?;
            write_path(path, &signed)
        }
        ArchiveFormat::NativePhar => {
            let mut out = strip_signature_trailer(&data).to_vec();
            let digest = compute_signature(algo, None, &out)?;
            out.extend_from_slice(&digest);
            out.extend_from_slice(&algo.to_le_bytes());
            out.extend_from_slice(b"GBMB");
            write_path(path, &out)
        }
    }
}

/// Decodes a tar/zip `.phar/signature.bin` payload into its flag and signature
/// bytes (`LE32(flag) ++ LE32(len) ++ signature`).
pub(super) fn parse_signature_bin(payload: &[u8]) -> Option<(u32, Vec<u8>)> {
    let flag = le32(payload, 0)?;
    let len = le32(payload, 4)? as usize;
    Some((flag, payload.get(8..8usize.checked_add(len)?)?.to_vec()))
}

/// Returns the raw `.phar/signature.bin` payload from a tar phar, if present.
pub(super) fn read_tar_signature(data: &[u8]) -> Option<Vec<u8>> {
    let mut p = 0usize;
    while p.checked_add(512)? <= data.len() {
        let header = &data[p..p + 512];
        if header.iter().all(|&b| b == 0) {
            break;
        }
        let size = parse_tar_octal(&header[124..136])?;
        let payload_start = p.checked_add(512)?;
        let typeflag = header[156];
        if (typeflag == 0 || typeflag == b'0') && tar_entry_name(header)? == PHAR_SIGNATURE_ENTRY {
            return data
                .get(payload_start..payload_start.checked_add(size)?)
                .map(<[u8]>::to_vec);
        }
        p = payload_start.checked_add(round_up_to_512(size)?)?;
    }
    None
}

/// Returns the raw `.phar/signature.bin` payload from a zip phar, if present.
pub(super) fn read_zip_signature(data: &[u8]) -> Option<Vec<u8>> {
    let (entry_count, central_dir_offset) = zip_eocd_info(data)?;
    let mut p = central_dir_offset;
    for _ in 0..entry_count {
        if le32(data, p)? != 0x0201_4b50 {
            return None;
        }
        let method = le16(data, p + 10)?;
        let mut compressed_size = le32(data, p + 20)? as usize;
        let mut uncompressed_size = le32(data, p + 24)? as usize;
        let name_len = le16(data, p + 28)? as usize;
        let extra_len = le16(data, p + 30)? as usize;
        let comment_len = le16(data, p + 32)? as usize;
        let mut local_offset = le32(data, p + 42)? as usize;
        let name_start = p + 46;
        let name = data.get(name_start..name_start.checked_add(name_len)?)?;
        if name == PHAR_SIGNATURE_ENTRY {
            apply_zip64_central_extra(
                data,
                name_start.checked_add(name_len)?,
                extra_len,
                &mut uncompressed_size,
                &mut compressed_size,
                &mut local_offset,
            )?;
            // The reserved signature entry is never encrypted.
            return decode_zip_local_entry(
                data,
                local_offset,
                method,
                compressed_size,
                uncompressed_size,
                false,
                0,
            );
        }
        p = name_start
            .checked_add(name_len)?
            .checked_add(extra_len)?
            .checked_add(comment_len)?;
    }
    None
}

/// Reads the signature of the phar at `path`, returning the flag and the raw
/// signature/digest bytes. Native PHARs use the `GBMB` trailer; tar/zip phars use
/// the `.phar/signature.bin` entry.
pub(super) fn read_signature_info(path: &[u8]) -> Option<(u32, Vec<u8>)> {
    let data = read_path(path)?;
    match signing_format(&data)? {
        ArchiveFormat::Zip => parse_signature_bin(&read_zip_signature(&data)?),
        ArchiveFormat::Tar => parse_signature_bin(&read_tar_signature(&data)?),
        ArchiveFormat::NativePhar => {
            let n = data.len();
            if n < 8 || &data[n - 4..] != b"GBMB" {
                return None;
            }
            let flags = u32::from_le_bytes(data[n - 8..n - 4].try_into().unwrap());
            if flags == PHAR_OPENSSL_SIGNATURE_TYPE {
                let sig_len =
                    u32::from_le_bytes(data.get(n - 12..n - 8)?.try_into().unwrap()) as usize;
                let start = n.checked_sub(12)?.checked_sub(sig_len)?;
                Some((flags, data.get(start..n - 12)?.to_vec()))
            } else {
                let dlen = signature_digest_len(flags)?;
                let start = n.checked_sub(8)?.checked_sub(dlen)?;
                Some((flags, data.get(start..n - 8)?.to_vec()))
            }
        }
    }
}

/// Returns the uppercase hex of the PHAR's signature/digest bytes (PHP
/// `Phar::getSignature()['hash']`).
pub(super) fn signature_hash_hex(path: &[u8]) -> Option<Vec<u8>> {
    let (_, bytes) = read_signature_info(path)?;
    let mut hex = Vec::with_capacity(bytes.len() * 2);
    for byte in bytes {
        hex.extend_from_slice(format!("{byte:02X}").as_bytes());
    }
    Some(hex)
}

/// Returns the PHP signature type name for the PHAR (`getSignature()['hash_type']`).
pub(super) fn signature_type_name(path: &[u8]) -> Option<Vec<u8>> {
    let (flags, _) = read_signature_info(path)?;
    let name: &[u8] = match flags {
        1 => b"MD5",
        2 => b"SHA-1",
        3 => b"SHA-256",
        4 => b"SHA-512",
        PHAR_OPENSSL_SIGNATURE_TYPE => b"OpenSSL",
        _ => return None,
    };
    Some(name.to_vec())
}
