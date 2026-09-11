//! ZATCA (Saudi Arabia) & FTA (UAE) E-Invoicing TLV QR Code Generator.
//!
//! Provides compliance with Phase 1 and Phase 2 electronic invoicing requirements
//! established by the Saudi Zakat, Tax and Customs Authority (ZATCA / FATOORA)
//! and the United Arab Emirates Federal Tax Authority (FTA).
//!
//! Generates standard TLV (Tag-Length-Value) structures packed into RFC 4648 Base64
//! strings ready for printing as 2D QR codes on thermal receipt slips.

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Encodes arbitrary binary data into a standard RFC 4648 Base64 string with `=` padding.
#[must_use]
pub fn base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    let (chunks, remainder) = data.as_chunks::<3>();

    for &[b0, b1, b2] in chunks {
        out.push(BASE64_ALPHABET[(b0 >> 2) as usize] as char);
        out.push(BASE64_ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(BASE64_ALPHABET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        out.push(BASE64_ALPHABET[(b2 & 0x3f) as usize] as char);
    }

    match remainder.len() {
        1 => {
            let b0 = remainder[0];
            out.push(BASE64_ALPHABET[(b0 >> 2) as usize] as char);
            out.push(BASE64_ALPHABET[((b0 & 0x03) << 4) as usize] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let b0 = remainder[0];
            let b1 = remainder[1];
            out.push(BASE64_ALPHABET[(b0 >> 2) as usize] as char);
            out.push(BASE64_ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
            out.push(BASE64_ALPHABET[((b1 & 0x0f) << 2) as usize] as char);
            out.push('=');
        }
        _ => {}
    }

    out
}

/// Decodes an RFC 4648 Base64 string back into binary bytes.
///
/// Returns `None` if the input contains invalid characters or malformed padding.
#[must_use]
pub fn base64_decode(input: &str) -> Option<Vec<u8>> {
    fn decode_char(c: u8) -> Option<u8> {
        match c {
            b'A'..=b'Z' => Some(c - b'A'),
            b'a'..=b'z' => Some(c - b'a' + 26),
            b'0'..=b'9' => Some(c - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        }
    }

    let bytes = input.trim().as_bytes();
    if bytes.is_empty() {
        return Some(Vec::new());
    }

    let unpadded = match bytes.strip_suffix(b"==") {
        Some(s) => s,
        None => bytes.strip_suffix(b"=").unwrap_or(bytes),
    };

    let mut out = Vec::with_capacity(unpadded.len() * 3 / 4);
    let (chunks, remainder) = unpadded.as_chunks::<4>();

    for &[c0, c1, c2, c3] in chunks {
        let n0 = decode_char(c0)?;
        let n1 = decode_char(c1)?;
        let n2 = decode_char(c2)?;
        let n3 = decode_char(c3)?;

        out.push((n0 << 2) | (n1 >> 4));
        out.push((n1 << 4) | (n2 >> 2));
        out.push((n2 << 6) | n3);
    }

    match remainder.len() {
        2 => {
            let n0 = decode_char(remainder[0])?;
            let n1 = decode_char(remainder[1])?;
            out.push((n0 << 2) | (n1 >> 4));
        }
        3 => {
            let n0 = decode_char(remainder[0])?;
            let n1 = decode_char(remainder[1])?;
            let n2 = decode_char(remainder[2])?;
            out.push((n0 << 2) | (n1 >> 4));
            out.push((n1 << 4) | (n2 >> 2));
        }
        0 => {}
        _ => return None,
    }

    Some(out)
}

/// Parsed TLV tag and value entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlvEntry {
    /// Tag identifier (1 to 9).
    pub tag: u8,
    /// Raw byte payload of the entry.
    pub value: Vec<u8>,
}

/// Decodes binary TLV bytes into individual tag entries.
#[must_use]
pub fn decode_tlv(bytes: &[u8]) -> Vec<TlvEntry> {
    let mut entries = Vec::new();
    let mut idx = 0;
    while idx + 2 <= bytes.len() {
        let tag = bytes[idx];
        let len = bytes[idx + 1] as usize;
        idx += 2;
        if idx + len <= bytes.len() {
            entries.push(TlvEntry {
                tag,
                value: bytes[idx..idx + len].to_vec(),
            });
            idx += len;
        } else {
            break;
        }
    }
    entries
}

fn append_tlv(buf: &mut Vec<u8>, tag: u8, value: &[u8]) {
    buf.push(tag);
    let len = value.len().min(255) as u8;
    buf.push(len);
    buf.extend_from_slice(&value[..len as usize]);
}

/// A structured invoice definition for generating ZATCA (Saudi Arabia) & FTA (UAE) E-Invoicing QR codes.
///
/// In compliance with ZATCA Phase 1 (Generation) and Phase 2 (Integration) specifications.
///
/// # Example
///
/// ```rust
/// use papermint::tax::ZatcaInvoice;
///
/// let invoice = ZatcaInvoice::new(
///     "Bob's Fashions",
///     "310122393500003",
///     "2022-04-25T15:30:00Z",
///     "1000.00",
///     "150.00",
/// );
///
/// let qr_base64 = invoice.to_qr_base64();
/// assert!(!qr_base64.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZatcaInvoice {
    /// Tag 1 (0x01): Seller's legal or commercial trading name.
    pub seller_name: String,
    /// Tag 2 (0x02): Tax Registration Number (TRN / VAT number, typically 15 digits).
    pub vat_number: String,
    /// Tag 3 (0x03): Invoice timestamp in ISO 8601 format (`YYYY-MM-DDTHH:MM:SSZ`).
    pub timestamp: String,
    /// Tag 4 (0x04): Invoice total amount including VAT (in decimal format, e.g. "115.00").
    pub total_amount: String,
    /// Tag 5 (0x05): Total VAT amount (in decimal format, e.g. "15.00").
    pub vat_amount: String,
    /// Tag 6 (0x06 - Optional): Cryptographic SHA-256 hash of the XML invoice (Phase 2).
    pub invoice_hash: Option<Vec<u8>>,
    /// Tag 7 (0x07 - Optional): Digital signature using ECDSA algorithm (Phase 2).
    pub signature: Option<Vec<u8>>,
    /// Tag 8 (0x08 - Optional): ECDSA public key (Phase 2).
    pub public_key: Option<Vec<u8>>,
    /// Tag 9 (0x09 - Optional): Cryptographic stamp of ZATCA (Phase 2).
    pub stamp: Option<Vec<u8>>,
}

impl ZatcaInvoice {
    /// Creates a new Phase 1 compliant [`ZatcaInvoice`] with mandatory fields.
    #[must_use]
    pub fn new(
        seller_name: impl Into<String>,
        vat_number: impl Into<String>,
        timestamp: impl Into<String>,
        total_amount: impl Into<String>,
        vat_amount: impl Into<String>,
    ) -> Self {
        Self {
            seller_name: seller_name.into(),
            vat_number: vat_number.into(),
            timestamp: timestamp.into(),
            total_amount: total_amount.into(),
            vat_amount: vat_amount.into(),
            invoice_hash: None,
            signature: None,
            public_key: None,
            stamp: None,
        }
    }

    /// Sets the Phase 2 cryptographic SHA-256 invoice hash (Tag 6).
    #[must_use]
    pub fn with_invoice_hash(mut self, hash: impl Into<Vec<u8>>) -> Self {
        self.invoice_hash = Some(hash.into());
        self
    }

    /// Sets the Phase 2 digital ECDSA signature (Tag 7).
    #[must_use]
    pub fn with_signature(mut self, signature: impl Into<Vec<u8>>) -> Self {
        self.signature = Some(signature.into());
        self
    }

    /// Sets the Phase 2 ECDSA public key (Tag 8).
    #[must_use]
    pub fn with_public_key(mut self, public_key: impl Into<Vec<u8>>) -> Self {
        self.public_key = Some(public_key.into());
        self
    }

    /// Sets the Phase 2 ZATCA cryptographic stamp (Tag 9).
    #[must_use]
    pub fn with_stamp(mut self, stamp: impl Into<Vec<u8>>) -> Self {
        self.stamp = Some(stamp.into());
        self
    }

    /// Encodes invoice metadata into raw binary TLV (Tag-Length-Value) bytes.
    #[must_use]
    pub fn to_tlv_bytes(&self) -> Vec<u8> {
        encode_zatca_tlv(self)
    }

    /// Encodes invoice metadata into an RFC 4648 Base64 string ready for QR code rendering.
    #[must_use]
    pub fn to_qr_base64(&self) -> String {
        zatca_qr_base64(self)
    }
}

/// Encodes a [`ZatcaInvoice`] into binary TLV (Tag-Length-Value) representation.
#[must_use]
pub fn encode_zatca_tlv(invoice: &ZatcaInvoice) -> Vec<u8> {
    let mut buf = Vec::with_capacity(128);

    // Tag 1: Seller's Name
    append_tlv(&mut buf, 1, invoice.seller_name.as_bytes());

    // Tag 2: VAT Registration Number
    append_tlv(&mut buf, 2, invoice.vat_number.as_bytes());

    // Tag 3: Time stamp of the invoice
    append_tlv(&mut buf, 3, invoice.timestamp.as_bytes());

    // Tag 4: Invoice total amount (with VAT)
    append_tlv(&mut buf, 4, invoice.total_amount.as_bytes());

    // Tag 5: Total VAT amount
    append_tlv(&mut buf, 5, invoice.vat_amount.as_bytes());

    // Tag 6: SHA-256 Hash of XML invoice (Optional)
    if let Some(hash) = &invoice.invoice_hash {
        append_tlv(&mut buf, 6, hash);
    }

    // Tag 7: ECDSA signature (Optional)
    if let Some(sig) = &invoice.signature {
        append_tlv(&mut buf, 7, sig);
    }

    // Tag 8: ECDSA Public Key (Optional)
    if let Some(key) = &invoice.public_key {
        append_tlv(&mut buf, 8, key);
    }

    // Tag 9: Cryptographic Stamp (Optional)
    if let Some(stamp) = &invoice.stamp {
        append_tlv(&mut buf, 9, stamp);
    }

    buf
}

/// Generates a compliant Base64 QR payload string for a [`ZatcaInvoice`].
#[must_use]
pub fn zatca_qr_base64(invoice: &ZatcaInvoice) -> String {
    let tlv_bytes = encode_zatca_tlv(invoice);
    base64_encode(&tlv_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_encode_decode() {
        let cases = [
            ("", ""),
            ("f", "Zg=="),
            ("fo", "Zm8="),
            ("foo", "Zm9v"),
            ("foob", "Zm9vYg=="),
            ("fooba", "Zm9vYmE="),
            ("foobar", "Zm9vYmFy"),
            ("Hello, World!", "SGVsbG8sIFdvcmxkIQ=="),
        ];

        for (plain, expected_b64) in cases {
            let encoded = base64_encode(plain.as_bytes());
            assert_eq!(encoded, expected_b64, "Encoding mismatch for '{plain}'");

            let decoded = base64_decode(&encoded).expect("Decode should succeed");
            assert_eq!(
                String::from_utf8(decoded).unwrap(),
                plain,
                "Decoded mismatch for '{plain}'"
            );
        }
    }

    #[test]
    fn test_zatca_tlv_roundtrip() {
        let invoice = ZatcaInvoice::new(
            "Bob's Fashions",
            "310122393500003",
            "2022-04-25T15:30:00Z",
            "1000.00",
            "150.00",
        );

        let tlv = invoice.to_tlv_bytes();
        let entries = decode_tlv(&tlv);

        assert_eq!(entries.len(), 5);
        assert_eq!(entries[0].tag, 1);
        assert_eq!(
            std::str::from_utf8(&entries[0].value).unwrap(),
            "Bob's Fashions"
        );
        assert_eq!(entries[1].tag, 2);
        assert_eq!(
            std::str::from_utf8(&entries[1].value).unwrap(),
            "310122393500003"
        );
        assert_eq!(entries[2].tag, 3);
        assert_eq!(
            std::str::from_utf8(&entries[2].value).unwrap(),
            "2022-04-25T15:30:00Z"
        );
        assert_eq!(entries[3].tag, 4);
        assert_eq!(std::str::from_utf8(&entries[3].value).unwrap(), "1000.00");
        assert_eq!(entries[4].tag, 5);
        assert_eq!(std::str::from_utf8(&entries[4].value).unwrap(), "150.00");

        let b64 = invoice.to_qr_base64();
        let decoded_b64 = base64_decode(&b64).unwrap();
        assert_eq!(decoded_b64, tlv);
    }

    #[test]
    fn test_zatca_phase_2_fields() {
        let hash = vec![0xAB; 32];
        let sig = vec![0xCD; 64];
        let invoice = ZatcaInvoice::new(
            "Saudi Tech Co",
            "300000000000003",
            "2026-09-10T12:00:00Z",
            "500.00",
            "75.00",
        )
        .with_invoice_hash(hash.clone())
        .with_signature(sig.clone());

        let tlv = invoice.to_tlv_bytes();
        let entries = decode_tlv(&tlv);

        assert_eq!(entries.len(), 7);
        assert_eq!(entries[5].tag, 6);
        assert_eq!(entries[5].value, hash);
        assert_eq!(entries[6].tag, 7);
        assert_eq!(entries[6].value, sig);
    }
}
