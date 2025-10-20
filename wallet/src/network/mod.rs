use crate::util::hex::HexError;

pub mod bitcoin;
pub mod ethereum;
pub mod tron;

#[derive(Debug)]
pub struct Wallet {
    pub privkey: String,
    pub pubkey: String,
    pub address: String,
}

impl Wallet {
    pub fn new(privkey: String, pubkey: String, address: String) -> Self {
        Self {
            privkey,
            pubkey,
            address,
        }
    }
}

#[derive(Debug)]
pub enum WalletError {
    // Bitcoin
    BtcNewFromPrivateKeyImportFail(String),
    BtcNewFromHexPrivateKeyHexDecodeFail(HexError),
    BtcNewFromHexPrivateKeyWifImportFail(String),
    BtcGenerateAddressPubkeyCompressFail(String),

    // Ethereum
    EthNewFromPrivateKeyHexDecodeFail(HexError),
    EthNewFromPrivateKeyWalletImportFail(String),
    EthChecksumAddressFromStrFail(String),

    // Tron
    TronBase58DecodeTooShort(String),
    TronBase58DecodeFail(String),
    TronBase58ChecksumMismatch,
    TronNewFromPrivateKeyImportFail(String),
    TronNewFromPrivateKeyHexDecodeFail(HexError),
}
