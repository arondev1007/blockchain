use std::str::FromStr;

use ethers::core::rand::rngs::OsRng;
use ethers::signers::{LocalWallet, Signer};
use ethers::types::H160;
pub use ethers::utils::to_checksum;

use crate::network::*;
use crate::util::hex::*;

#[derive(Debug)]
pub struct WalletEth {
    wallet: LocalWallet,
}

impl WalletEth {
    pub fn new() -> Self {
        let mut rng = OsRng;
        let wallet = LocalWallet::new(&mut rng);
        WalletEth { wallet }
    }

    pub fn from_privkey(s: &str) -> Result<Self, WalletError> {
        let privkey =
            Hex::decode(s).map_err(|e| WalletError::EthNewFromPrivateKeyHexDecodeFail(e))?;

        Self::from_bytes(&privkey)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WalletError> {
        let wallet = LocalWallet::from_bytes(&bytes)
            .map_err(|e| WalletError::EthNewFromPrivateKeyWalletImportFail(e.to_string()))?;

        Ok(WalletEth { wallet })
    }

    pub fn privkey_to_address(privkey: &[u8]) -> Result<String, WalletError> {
        let wallet_eth = Self::from_bytes(privkey)?;
        let wallet = wallet_eth.export()?;

        Ok(wallet.address)
    }

    pub fn validate_address(s: &str) -> bool {
        // length validation
        if s.len() != 42 {
            return false;
        }

        // start address 0x validation
        if s.starts_with("0x") {
            return false;
        }

        true
    }

    pub fn export(&self) -> Result<Wallet, WalletError> {
        // extract private key from wallet
        let privkey = self.wallet.signer().to_bytes();

        // extract public key
        let verifying_key = self.wallet.signer().verifying_key();
        let compressed_point = verifying_key.to_encoded_point(true);
        let pubkey = compressed_point.as_bytes();

        // extract address
        let address = self.wallet.address();
        let checksum_address = to_checksum(&address, None);

        Ok(Wallet::new(
            Hex::encode(&privkey),
            Hex::encode(&pubkey),
            checksum_address,
        ))
    }

    pub fn checksum_address(s: &str) -> Result<String, WalletError> {
        let h160_address = H160::from_str(s)
            .map_err(|e| WalletError::EthChecksumAddressFromStrFail(e.to_string()))?;

        let checksum_address = to_checksum(&h160_address, None);
        Ok(checksum_address)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wallet_random() {
        // 1) random wallet creation
        let wallet_eth = WalletEth::new();
        let wallet = wallet_eth.export().unwrap();
        println!("(Random) TEthereumWallet = {:?}", wallet);
    }

    #[test]
    fn test_import_private_key() {
        let privkey = [
            235, 208, 246, 59, 19, 147, 175, 38, 187, 107, 183, 122, 61, 181, 207, 56, 217, 161,
            94, 122, 1, 176, 197, 176, 221, 64, 231, 228, 87, 89, 168, 179,
        ];

        // 1) create wallet from existing private key
        let wallet_eth =
            WalletEth::from_bytes(&privkey).expect("Failed to create wallet from private key");

        // 2) Export
        let wallet = wallet_eth.export().expect("Failed to export wallet");
        println!("(Imported) TEthereumWallet = {:?}", wallet);
    }

    #[test]
    fn test_import_hex_private_key() {
        let hex_privkey = "ebd0f63b1393af26bb6bb77a3db5cf38d9a15e7a01b0c5b0dd40e7e45759a8b3";

        // 1) create wallet from existing private key
        let wallet_eth =
            WalletEth::from_privkey(hex_privkey).expect("Failed to create wallet from private key");

        // 2) Export
        let wallet = wallet_eth.export().expect("Failed to export wallet");
        println!("(Imported) TEthereumWallet = {:?}", wallet);
    }

    #[test]
    fn test_private_key_to_address() {
        let privkey = [
            235, 208, 246, 59, 19, 147, 175, 38, 187, 107, 183, 122, 61, 181, 207, 56, 217, 161,
            94, 122, 1, 176, 197, 176, 221, 64, 231, 228, 87, 89, 168, 179,
        ];

        let address = WalletEth::privkey_to_address(&privkey)
            .expect("Failed to create wallet from private key");

        println!("(Imported) TEthereumWallet = {:?}", address);
    }

    #[test]
    fn test_checksum_address() {
        let address = "0x92664edbddccad08df691f4409973444e66266ed";
        let checksum_address =
            WalletEth::checksum_address(address).expect("Failed to create wallet from private key");

        println!("(Imported) TEthereumWallet = {:?}", checksum_address);
    }
}
