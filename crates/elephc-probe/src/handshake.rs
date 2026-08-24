//! Purpose:
//! The build-key mutual authentication for the production probe endpoint. Both
//! the compiled server (which links this crate) and the `--probe-host` client
//! (the compiler binary, which depends on this crate's rlib) use these exact
//! functions, so the two sides can never drift.
//!
//! Protocol (challenge-response, no secret on the wire, no replay):
//!   client → server : nonce_c
//!   server → client : nonce_s, HMAC(key, "S" || nonce_c || nonce_s)   (binary proves identity)
//!   client → server : HMAC(key, "C" || nonce_s || nonce_c)            (operator proves authority)
//!
//! The 32-byte key is embedded in the binary and written to a `.probe-key`
//! sidecar at compile time. Possession of the key is the credential — the same
//! property a binary hash would give, but insensitive to signing/strip/layers
//! and never transmitted.

/// HMAC-SHA256 tag length and the key length the probe embeds.
pub const KEY_LEN: usize = 32;
/// Nonce length each side contributes to the challenge.
pub const NONCE_LEN: usize = 32;
/// HMAC-SHA256 output length.
pub const TAG_LEN: usize = 32;

// --- SHA-256 (FIPS 180-4), dependency-free so the crate stays lean ---
//
// Not async-signal-safe, and the wording used to imply otherwise: the digests
// allocate, and `malloc` inside a signal handler deadlocks if the interrupted
// thread was already in the allocator. Everything below runs in ordinary
// context — the handshake, the per-request header check, the startup fd probe —
// and nothing here may be called from the SIGPROF handler.

const SHA256_H: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

const SHA256_K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// Computes the SHA-256 digest of `message`.
pub fn sha256(message: &[u8]) -> [u8; 32] {
    let mut h = SHA256_H;
    let bit_len = (message.len() as u64) * 8;
    let mut padded = message.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in padded.chunks_exact(64) {
        let mut w = [0u32; 64];
        for (i, word) in chunk.chunks_exact(4).enumerate() {
            w[i] = u32::from_be_bytes([word[0], word[1], word[2], word[3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = h;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let t1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(SHA256_K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(maj);
            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (slot, value) in h.iter_mut().zip([a, b, c, d, e, f, g, hh]) {
            *slot = slot.wrapping_add(value);
        }
    }
    let mut digest = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        digest[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    digest
}

/// Computes HMAC-SHA256 of `message` under `key` (RFC 2104).
pub fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; TAG_LEN] {
    let mut block = [0u8; 64];
    if key.len() > 64 {
        block[..32].copy_from_slice(&sha256(key));
    } else {
        block[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0x36u8; 64];
    let mut opad = [0x5cu8; 64];
    for i in 0..64 {
        ipad[i] ^= block[i];
        opad[i] ^= block[i];
    }
    let mut inner = ipad.to_vec();
    inner.extend_from_slice(message);
    let inner_digest = sha256(&inner);
    let mut outer = opad.to_vec();
    outer.extend_from_slice(&inner_digest);
    sha256(&outer)
}

/// The public build fingerprint: first 8 hex chars of SHA-256(key). Safe to
/// print on both sides so an operator can confirm they reached the intended
/// build without exposing the key.
pub fn fingerprint(key: &[u8]) -> String {
    let digest = sha256(key);
    digest[..4].iter().map(|b| format!("{b:02x}")).collect()
}

/// Server proof: HMAC over the domain-separated `nonce_c || nonce_s`, proving
/// the binary holds the key.
pub fn server_tag(key: &[u8], nonce_c: &[u8], nonce_s: &[u8]) -> [u8; TAG_LEN] {
    let mut message = Vec::with_capacity(1 + nonce_c.len() + nonce_s.len());
    message.push(b'S');
    message.extend_from_slice(nonce_c);
    message.extend_from_slice(nonce_s);
    hmac_sha256(key, &message)
}

/// Client proof: HMAC over the domain-separated `nonce_s || nonce_c`, proving
/// the operator holds the key. Distinct domain byte from the server tag so a
/// server response can never be replayed as a client proof.
pub fn client_tag(key: &[u8], nonce_s: &[u8], nonce_c: &[u8]) -> [u8; TAG_LEN] {
    let mut message = Vec::with_capacity(1 + nonce_s.len() + nonce_c.len());
    message.push(b'C');
    message.extend_from_slice(nonce_s);
    message.extend_from_slice(nonce_c);
    hmac_sha256(key, &message)
}

/// Derives this connection's keys from the build key and both nonces.
///
/// Binding to BOTH nonces is what makes the derived material specific to this
/// connection: a recorded exchange cannot be replayed against a later one, and a
/// relay that forwards someone else's handshake ends up with keys for a session
/// it cannot produce plaintext for. The version string is in the input so a
/// future change of construction cannot be confused with this one.
///
/// Two subkeys rather than one, because a key used both to generate a keystream
/// and to authenticate it is one mistake away from a reused mask.
pub fn session_keys(key: &[u8], nonce_c: &[u8], nonce_s: &[u8]) -> ([u8; TAG_LEN], [u8; TAG_LEN]) {
    let mut message = Vec::with_capacity(23 + nonce_c.len() + nonce_s.len());
    message.extend_from_slice(b"elephc-probe-session-v1");
    message.extend_from_slice(nonce_c);
    message.extend_from_slice(nonce_s);
    let session = hmac_sha256(key, &message);
    (hmac_sha256(&session, b"enc"), hmac_sha256(&session, b"mac"))
}

/// XORs `data` with the keystream `HMAC(k_enc, counter)`, block by block.
///
/// Symmetric: sealing and opening are the same operation. The counter is the
/// only thing that varies, and the subkey is derived per connection, so no two
/// connections share a keystream.
fn apply_keystream(k_enc: &[u8; TAG_LEN], data: &mut [u8]) {
    for (index, chunk) in data.chunks_mut(TAG_LEN).enumerate() {
        let block = hmac_sha256(k_enc, &(index as u64).to_be_bytes());
        for (byte, mask) in chunk.iter_mut().zip(block.iter()) {
            *byte ^= mask;
        }
    }
}

/// Encrypts `plaintext` and returns `(ciphertext, tag)`.
///
/// Encrypt-then-MAC: the tag covers the ciphertext and its length, so a receiver
/// can reject a forgery before decrypting anything, and a tampered length is
/// caught rather than turning into a short read.
pub fn seal(
    k_enc: &[u8; TAG_LEN],
    k_mac: &[u8; TAG_LEN],
    plaintext: &[u8],
) -> (Vec<u8>, [u8; TAG_LEN]) {
    let mut ciphertext = plaintext.to_vec();
    apply_keystream(k_enc, &mut ciphertext);
    let tag = payload_tag(k_mac, &ciphertext);
    (ciphertext, tag)
}

/// Verifies `tag` and decrypts, or `None` if the payload was not produced by a
/// holder of these keys. Verification comes first and is constant time.
pub fn open(
    k_enc: &[u8; TAG_LEN],
    k_mac: &[u8; TAG_LEN],
    ciphertext: &[u8],
    tag: &[u8],
) -> Option<Vec<u8>> {
    if !tags_equal(&payload_tag(k_mac, ciphertext), tag) {
        return None;
    }
    let mut plaintext = ciphertext.to_vec();
    apply_keystream(k_enc, &mut plaintext);
    Some(plaintext)
}

/// The authentication tag over a framed payload: its length, then its bytes.
fn payload_tag(k_mac: &[u8; TAG_LEN], ciphertext: &[u8]) -> [u8; TAG_LEN] {
    let mut message = Vec::with_capacity(4 + ciphertext.len());
    message.extend_from_slice(&(ciphertext.len() as u32).to_be_bytes());
    message.extend_from_slice(ciphertext);
    hmac_sha256(k_mac, &message)
}

/// Constant-time tag comparison, so a rejected handshake leaks no timing.
pub fn tags_equal(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    /// A sealed profile round-trips, and does not appear on the wire.
    ///
    /// The second half is the one that matters: a `seal` that returned its input
    /// would pass a round-trip test and leak exactly what this exists to hide.
    #[test]
    fn a_sealed_profile_round_trips_and_is_not_readable_in_transit() {
        let key = [7u8; KEY_LEN];
        let nonce_c = [1u8; NONCE_LEN];
        let nonce_s = [2u8; NONCE_LEN];
        let profile = "elephc-probe: {main};hot 42\nelephc-probe-samples: 42\n";

        let (k_enc, k_mac) = session_keys(&key, &nonce_c, &nonce_s);
        let (sealed, tag) = seal(&k_enc, &k_mac, profile.as_bytes());

        assert_ne!(sealed.as_slice(), profile.as_bytes(), "the payload is in the clear");
        assert!(
            !sealed.windows(6).any(|w| w == b"{main}"),
            "a recognizable fragment of the profile survived on the wire"
        );
        assert_eq!(
            open(&k_enc, &k_mac, &sealed, &tag).as_deref(),
            Some(profile.as_bytes()),
            "the intended reader must get it back"
        );
    }

    /// Holding the build key is not enough: the keys are bound to BOTH nonces,
    /// so a recorded exchange cannot be replayed against a later connection.
    #[test]
    fn a_payload_from_another_session_does_not_open() {
        let key = [7u8; KEY_LEN];
        let (k_enc, k_mac) = session_keys(&key, &[1u8; NONCE_LEN], &[2u8; NONCE_LEN]);
        let (sealed, tag) = seal(&k_enc, &k_mac, b"elephc-probe: {main} 1\n");

        for (nonce_c, nonce_s) in [
            ([9u8; NONCE_LEN], [2u8; NONCE_LEN]),
            ([1u8; NONCE_LEN], [9u8; NONCE_LEN]),
        ] {
            let (other_enc, other_mac) = session_keys(&key, &nonce_c, &nonce_s);
            assert!(
                open(&other_enc, &other_mac, &sealed, &tag).is_none(),
                "a different session opened this payload"
            );
        }

        let (wrong_enc, wrong_mac) =
            session_keys(&[8u8; KEY_LEN], &[1u8; NONCE_LEN], &[2u8; NONCE_LEN]);
        assert!(
            open(&wrong_enc, &wrong_mac, &sealed, &tag).is_none(),
            "a different build key opened this payload"
        );
    }

    /// Anything changed in flight is refused rather than decrypted into
    /// plausible-looking garbage: the tag is checked before the keystream runs.
    #[test]
    fn a_tampered_payload_is_refused() {
        let key = [7u8; KEY_LEN];
        let (k_enc, k_mac) = session_keys(&key, &[1u8; NONCE_LEN], &[2u8; NONCE_LEN]);
        let (sealed, tag) = seal(&k_enc, &k_mac, b"elephc-probe: {main};hot 10\n");

        let mut flipped = sealed.clone();
        flipped[0] ^= 1;
        assert!(open(&k_enc, &k_mac, &flipped, &tag).is_none(), "a flipped byte opened");

        let mut bad_tag = tag;
        bad_tag[TAG_LEN - 1] ^= 1;
        assert!(open(&k_enc, &k_mac, &sealed, &bad_tag).is_none(), "a flipped tag opened");

        assert!(
            open(&k_enc, &k_mac, &sealed[..sealed.len() - 1], &tag).is_none(),
            "a truncated payload opened"
        );
    }

    /// An empty profile is a legitimate answer — a process that has sampled
    /// nothing yet — and must survive the frame like any other.
    #[test]
    fn an_empty_profile_seals_and_opens() {
        let (k_enc, k_mac) = session_keys(&[7u8; KEY_LEN], &[1u8; NONCE_LEN], &[2u8; NONCE_LEN]);
        let (sealed, tag) = seal(&k_enc, &k_mac, b"");
        assert!(sealed.is_empty());
        assert_eq!(open(&k_enc, &k_mac, &sealed, &tag).as_deref(), Some(&b""[..]));
    }

    /// The keystream must not repeat across blocks, or a payload longer than one
    /// digest would XOR two of its parts against the same mask.
    #[test]
    fn the_keystream_differs_between_blocks() {
        let (k_enc, k_mac) = session_keys(&[7u8; KEY_LEN], &[1u8; NONCE_LEN], &[2u8; NONCE_LEN]);
        let plaintext = vec![0u8; TAG_LEN * 3];
        let (sealed, _) = seal(&k_enc, &k_mac, &plaintext);
        // Sealing zeroes yields the keystream itself, so the blocks are visible.
        assert_ne!(sealed[..TAG_LEN], sealed[TAG_LEN..TAG_LEN * 2], "block 0 == block 1");
        assert_ne!(
            sealed[TAG_LEN..TAG_LEN * 2],
            sealed[TAG_LEN * 2..],
            "block 1 == block 2"
        );
    }

    use super::*;

    // NIST SHA-256 known-answer vectors.
    #[test]
    fn sha256_matches_known_vectors() {
        assert_eq!(
            hex(&sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex(&sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    // RFC 4231 HMAC-SHA256 test case 2.
    #[test]
    fn hmac_matches_rfc4231() {
        let tag = hmac_sha256(b"Jefe", b"what do ya want for nothing?");
        assert_eq!(
            hex(&tag),
            "5bdcc146bf60754e6a042426089575c75a003f089d2739839dec58b964ec3843"
        );
    }

    #[test]
    fn mutual_handshake_accepts_the_matching_key_and_rejects_others() {
        let key = [7u8; KEY_LEN];
        let nonce_c = [1u8; NONCE_LEN];
        let nonce_s = [2u8; NONCE_LEN];
        // Server proves identity; client verifies with the same key.
        let s = server_tag(&key, &nonce_c, &nonce_s);
        assert!(tags_equal(&s, &server_tag(&key, &nonce_c, &nonce_s)));
        // A different key produces a different server tag → client rejects.
        let wrong = [8u8; KEY_LEN];
        assert!(!tags_equal(&s, &server_tag(&wrong, &nonce_c, &nonce_s)));
        // Client proves authority; server verifies.
        let c = client_tag(&key, &nonce_s, &nonce_c);
        assert!(tags_equal(&c, &client_tag(&key, &nonce_s, &nonce_c)));
        // Server tag and client tag differ (domain separation) — no replay.
        assert!(!tags_equal(&s, &c));
    }

    #[test]
    fn fingerprint_is_stable_and_key_dependent() {
        let a = fingerprint(&[1u8; KEY_LEN]);
        assert_eq!(a.len(), 8);
        assert_eq!(a, fingerprint(&[1u8; KEY_LEN]));
        assert_ne!(a, fingerprint(&[2u8; KEY_LEN]));
    }

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }
}
