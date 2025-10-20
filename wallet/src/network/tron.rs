use ::bitcoin::{
    base58,
    key::rand::thread_rng,
    secp256k1::{PublicKey, Secp256k1, SecretKey},
};

use ethers::core::k256::sha2::Digest;
use ethers::core::k256::sha2::Sha256;
use tiny_keccak::{Hasher, Keccak};

use crate::network::*;
use crate::util::hex::*;

#[derive(Debug)]
pub struct WalletTron {
    privkey: [u8; 32],
    pubkey: [u8; 65],
}

impl WalletTron {
    pub fn new() -> Self {
        let mut rng = thread_rng();

        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut rng);

        Self {
            privkey: secret_key.secret_bytes(),
            pubkey: public_key.serialize_uncompressed(),
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WalletError> {
        let sk = SecretKey::from_slice(bytes)
            .map_err(|e| WalletError::TronNewFromPrivateKeyImportFail(e.to_string()))?;
        let secp = Secp256k1::new();
        let pubkey = PublicKey::from_secret_key(&secp, &sk);

        Ok(Self {
            privkey: sk.secret_bytes(),
            pubkey: pubkey.serialize_uncompressed(),
        })
    }

    pub fn from_privkey(s: &str) -> Result<Self, WalletError> {
        let privkey =
            Hex::decode(s).map_err(|e| WalletError::TronNewFromPrivateKeyHexDecodeFail(e))?;

        Self::from_bytes(&privkey)
    }

    pub fn base58_encode(address: &[u8]) -> String {
        let checksum1 = Sha256::digest(address);
        let checksum2 = Sha256::digest(&checksum1);
        let checksum = &checksum2[0..4];

        let mut extended_payload = Vec::from(address);
        extended_payload.extend_from_slice(checksum);

        base58::encode(&extended_payload)
    }

    pub fn base58_decode(b58_address: &str) -> Result<Vec<u8>, WalletError> {
        // Base58 decode
        let decoded = base58::decode(b58_address)
            .map_err(|e| WalletError::TronBase58DecodeFail(e.to_string()))?;

        // decoded data is minimum 5 bytes (1 byte or more data + 4 bytes checksum)
        if decoded.len() < 5 {
            return Err(WalletError::TronBase58DecodeTooShort(
                decoded.len().to_string(),
            ));
        }

        // payload and checksum separate
        let payload = &decoded[..decoded.len() - 4];
        let checksum_received = &decoded[decoded.len() - 4..];

        // recalculate checksum
        let checksum1 = Sha256::digest(payload);
        let checksum2 = Sha256::digest(&checksum1);
        let checksum_calculated = &checksum2[..4];

        // checksum validation
        if checksum_received != checksum_calculated {
            return Err(WalletError::TronBase58ChecksumMismatch);
        }

        // valid payload return
        Ok(payload.to_vec())
    }

    pub fn privkey_to_address(privkey: &[u8]) -> Result<String, WalletError> {
        let wallet_tron = Self::from_bytes(privkey)?;
        let wallet = wallet_tron.export()?;

        Ok(wallet.address)
    }

    pub fn export(&self) -> Result<Wallet, WalletError> {
        let privkey = Hex::encode(&self.privkey);
        let pubkey = Hex::encode(&self.pubkey);

        // uncompressed public key except 0x04 and calculate Keccak256 hash
        let pubkey_bytes = &self.pubkey[1..];
        let mut keccak = Keccak::v256();
        keccak.update(pubkey_bytes);
        let mut pubkey_hash = [0u8; 32];
        keccak.finalize(&mut pubkey_hash);

        // get last 20 bytes and append 0x41 (Tron address)
        let mut address = vec![0x41];
        address.extend_from_slice(&pubkey_hash[12..]);

        // Base58Check encoding
        let b58_address = Self::base58_encode(&address);
        Ok(Wallet::new(privkey, pubkey, b58_address))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wallet_random() {
        // 1) random wallet creation
        let wallet_tron = WalletTron::new();
        let wallet = wallet_tron.export().unwrap();
        println!("(Random) TTronWallet = {:?}", wallet);
    }

    #[test]
    fn test_import_private_key() {
        let privkey = [
            75, 179, 78, 228, 62, 194, 210, 11, 203, 80, 83, 242, 170, 100, 76, 244, 194, 115, 155,
            162, 60, 119, 134, 132, 207, 177, 99, 33, 227, 167, 193, 82,
        ];

        // 1) create wallet from existing private key
        let wallet_tron =
            WalletTron::from_bytes(&privkey).expect("Failed to create wallet from private key");

        // 2) Export
        let wallet = wallet_tron.export().expect("Failed to export wallet");
        println!("(Imported) TTronWallet = {:?}", wallet);
    }

    #[test]
    fn test_import_hex_private_key() {
        let hex_privkey = "4BB34EE43EC2D20BCB5053F2AA644CF4C2739BA23C778684CFB16321E3A7C152";

        // 1) create wallet from existing private key
        let wallet_tron = WalletTron::from_privkey(hex_privkey)
            .expect("Failed to create wallet from private key");

        // 2) Export
        let wallet = wallet_tron.export().expect("Failed to export wallet");
        println!("(Imported) TTronWallet = {:?}", wallet);
    }

    #[test]
    fn test_private_key_to_address() {
        let privkey = [
            123, 113, 61, 75, 104, 51, 36, 160, 223, 85, 247, 167, 53, 244, 66, 242, 44, 92, 233,
            120, 72, 230, 198, 54, 72, 30, 74, 16, 217, 61, 146, 208,
        ];

        let address = WalletTron::privkey_to_address(&privkey)
            .expect("Failed to create wallet from private key");

        println!("(Imported) TTronWallet = {:?}", address);
    }

    #[test]
    fn test_base58_encode_decode() {
        let def_address = "TNvKoz95a756fRpZkj31QJFWj7WzwESccG";
        let address = WalletTron::base58_decode(def_address).unwrap();
        let b58_address = WalletTron::base58_encode(&address);

        assert_eq!(def_address, b58_address);
    }
}
