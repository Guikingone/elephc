//! Purpose:
//! Tests hash and OpenSSL signatures for native, tar, and ZIP PHAR archives.
//!
//! Called from:
//! - `cargo test -p elephc-phar` through Rust's test harness.
//!
//! Key details:
//! - Tests verify both signature metadata and the exact signed byte ranges.

use super::*;

const TEST_RSA_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----\n\
MIICdgIBADANBgkqhkiG9w0BAQEFAASCAmAwggJcAgEAAoGBAOuAP7xZaVfhwn9l\n\
BaMgxKPU1ODBpuT7Ybu6Fav03TJp1BKc1wUMiXnUPraUUI2R2JxoattDe7R/LcGk\n\
jVoPiBGGPoxxTaByd5LJZJk6MJAiGBhzQT7bkK3OMDHLQqhziefqDFfnDLt/TN7+\n\
umuMCPtLmuF6UUXiebMzyH21x7jvAgMBAAECgYBBhL+2rgVxzrxm5vsnhEFQ9zB2\n\
i0ncYNey+7V1zr0PfoPi3cGwhOlmfJcqAp9ak534/c/kyqSK9esL+bTdvn5zIQqC\n\
Swt2znffaW9nC6lM/pkZcvGLETt2m0L71n6pZVkMewsGBm9YrBQFA1krC7BV674U\n\
mlOmmYpM3LPgzmRLwQJBAPm/G7O4Stmzu5xV5qtvYX1dNZ2gydkVyfK/AwCYpfbK\n\
8ZXntKeWCt1BER1hNBSMPacHKb0LotK3j3LNNteLHCECQQDxZdNsXNLTHylWKA/X\n\
dyM3SH9mM6ESZP07cU7Ifq6t9zJdTfGdiyxsAjaaXxDmShL+bAjU16iwaTAGcYTB\n\
NrMPAkEAoUGwVV7Nlbvji5I7mr4UKKoikGDdc/oJp1+GRMBLiQqI6s3ta7gJ08rL\n\
jjjRM+NJe6u4W4RD4eL8EJhIrOv5gQJAK4Tm+8c0PtmEU0L/sCGLWMEaLquqIy3P\n\
tXK0+FJWXYiOLOILaBKaHJK9k1EGM+4wxGtnoC+M+tjLzq2SeF7LIwJAPdLUn2Qq\n\
eGMK12chOVcx41RxYctqsOlEKCIt011yGsV2/Mdm9ljTXeyXvNXCVOVcnHaf1v5w\n\
rNiobfy8sSb6iw==\n\
-----END PRIVATE KEY-----\n";

/// OpenSSL signing replaces the native PHAR's SHA1 trailer with an RSA-SHA1
/// signature trailer, the signature is deterministic and verifies against the
/// derived public key, and the signature metadata reads back as OpenSSL.
#[test]
pub(super) fn native_phar_openssl_signature_round_trip() {
    use rsa::pkcs8::DecodePrivateKey;
    use rsa::{Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};
    use sha1::{Digest, Sha1};

    let path = std::env::temp_dir().join(format!("elephc_phar_sig_{}.phar", std::process::id()));
    let pb = path.to_string_lossy();
    assert_eq!(put_entry_bytes(pb.as_bytes(), b"a.txt", b"alpha"), Some(5));
    assert_eq!(
        sign_archive_openssl(pb.as_bytes(), TEST_RSA_KEY_PEM.as_bytes()),
        Some(())
    );

    let signed = std::fs::read(&path).unwrap();
    let n = signed.len();
    assert_eq!(&signed[n - 4..], b"GBMB");
    assert_eq!(
        u32::from_le_bytes(signed[n - 8..n - 4].try_into().unwrap()),
        PHAR_OPENSSL_SIGNATURE_TYPE
    );

    // The signature reads back as OpenSSL with a 1024-bit (128-byte) RSA signature.
    let (flags, sig) = read_signature_info(pb.as_bytes()).unwrap();
    assert_eq!(flags, PHAR_OPENSSL_SIGNATURE_TYPE);
    assert_eq!(sig.len(), 128, "1024-bit RSA signature is 128 bytes");
    assert_eq!(
        u32::from_le_bytes(signed[n - 12..n - 8].try_into().unwrap()) as usize,
        sig.len()
    );
    assert_eq!(signature_type_name(pb.as_bytes()).as_deref(), Some(&b"OpenSSL"[..]));

    // Re-signing is deterministic (PKCS#1 v1.5).
    assert_eq!(
        sign_archive_openssl(pb.as_bytes(), TEST_RSA_KEY_PEM.as_bytes()),
        Some(())
    );
    let (_, sig2) = read_signature_info(pb.as_bytes()).unwrap();
    assert_eq!(sig, sig2, "PKCS#1 v1.5 signature is deterministic");

    // The signature verifies against the public key over the signed data.
    let key = RsaPrivateKey::from_pkcs8_pem(TEST_RSA_KEY_PEM).unwrap();
    let pubkey = RsaPublicKey::from(&key);
    let data = strip_signature_trailer(&std::fs::read(&path).unwrap()).to_vec();
    let hashed = Sha1::digest(&data);
    pubkey
        .verify(Pkcs1v15Sign::new::<Sha1>(), &hashed, &sig)
        .expect("signature verifies");
    std::fs::remove_file(&path).ok();
}

/// Hash-based signing rewrites the native PHAR trailer with the requested digest
/// algorithm, readable back via the signature metadata.
#[test]
pub(super) fn native_phar_hash_signature_round_trip() {
    let path = std::env::temp_dir().join(format!("elephc_phar_hsig_{}.phar", std::process::id()));
    let pb = path.to_string_lossy();
    assert_eq!(put_entry_bytes(pb.as_bytes(), b"a.txt", b"alpha"), Some(5));
    // SHA-256 (algo 3): 32-byte digest, type "SHA-256".
    assert_eq!(sign_archive_hash(pb.as_bytes(), 3), Some(()));
    let (flags, digest) = read_signature_info(pb.as_bytes()).unwrap();
    assert_eq!(flags, 3);
    assert_eq!(digest.len(), 32);
    assert_eq!(signature_type_name(pb.as_bytes()).as_deref(), Some(&b"SHA-256"[..]));
    std::fs::remove_file(&path).ok();
}

/// Reconstructs the byte range a tar/zip phar signature is computed over from a
/// parsed archive: the tar data records, or the zip locals + central + comment.
pub(super) fn tar_zip_signed_range(arch: &Archive) -> Vec<u8> {
    match arch.format {
        ArchiveFormat::Tar => {
            let mut body = Vec::new();
            write_tar_body(&mut body, &arch.entries, &arch.metadata, &arch.stub).unwrap();
            body
        }
        ArchiveFormat::Zip => {
            let mut out = Vec::new();
            let mut central = Vec::new();
            write_zip_body(&mut out, &mut central, &arch.entries, &arch.stub).unwrap();
            out.extend_from_slice(&central);
            out.extend_from_slice(&arch.metadata);
            out
        }
        ArchiveFormat::NativePhar => unreachable!("native phars sign with a trailer"),
    }
}

/// Hash signing a tar/zip phar writes a hidden `.phar/signature.bin` entry
/// (`LE32(flag) ++ LE32(len) ++ digest`) computed over the signed range, leaving
/// real entries readable and reporting the right algorithm.
pub(super) fn check_tar_zip_hash_signature(ext: &str) {
    let path =
        std::env::temp_dir().join(format!("elephc_phar_sig_{}_{ext}.{ext}", std::process::id()));
    let pb = path.to_string_lossy();
    assert_eq!(
        put_entry_bytes(pb.as_bytes(), b"doc.txt", b"bundled document\n"),
        Some(17)
    );
    // SHA-256 (algo 3).
    assert_eq!(sign_archive_hash(pb.as_bytes(), 3), Some(()));
    let data = std::fs::read(&path).unwrap();
    // The signature entry is hidden; the real entry still reads back.
    assert_eq!(
        extract_entry_bytes(&data, b"doc.txt").as_deref(),
        Some(&b"bundled document\n"[..])
    );
    let (flag, digest) = read_signature_info(pb.as_bytes()).unwrap();
    assert_eq!(flag, 3);
    assert_eq!(digest.len(), 32);
    assert_eq!(signature_type_name(pb.as_bytes()).as_deref(), Some(&b"SHA-256"[..]));
    // The digest covers exactly the reconstructed signed range.
    let arch = parse_archive(&data).unwrap();
    assert_eq!(digest, compute_signature(3, None, &tar_zip_signed_range(&arch)).unwrap());
    std::fs::remove_file(&path).ok();
}

/// SHA-256 signing a tar phar round-trips through `.phar/signature.bin`.
#[test]
pub(super) fn tar_phar_hash_signature_round_trip() {
    check_tar_zip_hash_signature("tar");
}

/// SHA-256 signing a zip phar round-trips through `.phar/signature.bin`.
#[test]
pub(super) fn zip_phar_hash_signature_round_trip() {
    check_tar_zip_hash_signature("zip");
}

/// OpenSSL signing a tar/zip phar writes an RSA-SHA1 `.phar/signature.bin` that
/// verifies against the derived public key over the archive's signed range.
pub(super) fn check_tar_zip_openssl_signature(ext: &str) {
    use rsa::pkcs8::DecodePrivateKey;
    use rsa::{Pkcs1v15Sign, RsaPrivateKey, RsaPublicKey};
    use sha1::{Digest, Sha1};

    let path =
        std::env::temp_dir().join(format!("elephc_phar_osig_{}_{ext}.{ext}", std::process::id()));
    let pb = path.to_string_lossy();
    assert_eq!(
        put_entry_bytes(pb.as_bytes(), b"doc.txt", b"bundled document\n"),
        Some(17)
    );
    assert_eq!(
        sign_archive_openssl(pb.as_bytes(), TEST_RSA_KEY_PEM.as_bytes()),
        Some(())
    );
    let data = std::fs::read(&path).unwrap();
    let (flag, sig) = read_signature_info(pb.as_bytes()).unwrap();
    assert_eq!(flag, PHAR_OPENSSL_SIGNATURE_TYPE);
    assert_eq!(sig.len(), 128, "1024-bit RSA signature is 128 bytes");
    assert_eq!(signature_type_name(pb.as_bytes()).as_deref(), Some(&b"OpenSSL"[..]));
    // The signature verifies against the public key over the signed range.
    let arch = parse_archive(&data).unwrap();
    let key = RsaPrivateKey::from_pkcs8_pem(TEST_RSA_KEY_PEM).unwrap();
    let pubkey = RsaPublicKey::from(&key);
    let hashed = Sha1::digest(tar_zip_signed_range(&arch));
    pubkey
        .verify(Pkcs1v15Sign::new::<Sha1>(), &hashed, &sig)
        .expect("tar/zip OpenSSL signature verifies");
    std::fs::remove_file(&path).ok();
}

/// OpenSSL signing a tar phar verifies against the derived public key.
#[test]
pub(super) fn tar_phar_openssl_signature_round_trip() {
    check_tar_zip_openssl_signature("tar");
}

/// OpenSSL signing a zip phar verifies against the derived public key.
#[test]
pub(super) fn zip_phar_openssl_signature_round_trip() {
    check_tar_zip_openssl_signature("zip");
}
