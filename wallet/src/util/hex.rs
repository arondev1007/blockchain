#[derive(Debug, PartialEq)]
pub enum HexError {
    DecodeFail(String),
}

pub struct Hex;

impl Hex {
    pub fn encode(bytes: &[u8]) -> String {
        hex::encode(bytes)
    }

    pub fn decode(s: &str) -> Result<Vec<u8>, HexError> {
        hex::decode(s).map_err(|e| HexError::DecodeFail(e.to_string()))
    }
}

#[test]
fn test_encode_decode() {
    let data: Vec<u8> = vec![0, 250, 1, 2, 3, 4, 5, 10, 100];

    // encode
    let encoded = Hex::encode(&data);
    assert_eq!(encoded, "00fa01020304050a64");

    // decode
    let decoded = Hex::decode(&encoded).unwrap();
    assert_eq!(data, decoded);
}
