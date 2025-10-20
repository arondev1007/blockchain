use bitcoin::address::NetworkUnchecked;
use bitcoin::key::rand::rngs::OsRng;
use bitcoin::key::PrivateKey;
use bitcoin::secp256k1::{Keypair, PublicKey, Secp256k1, SecretKey, XOnlyPublicKey};
use bitcoin::{Address, CompressedPublicKey};

pub use bitcoin::network::Network;
pub use bitcoin::AddressType;

use crate::network::{Wallet, WalletError};
use crate::util::hex::*;

#[derive(Debug)]
pub enum AddrTypeBtc {
    Legacy,  // P2PKH
    P2SH,    // P2SH (예: p2sh-wpkh)
    Bech32,  // P2WPKH
    Taproot, // P2TR
}

#[derive(Debug)]
pub struct WalletBitcoin {
    network: Network,
    sk: SecretKey,
    pubkey: PublicKey,
}

impl WalletBitcoin {
    pub fn new(network: Network) -> Self {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

        WalletBitcoin {
            network,
            sk: secret_key,
            pubkey: public_key,
        }
    }

    pub fn from_privkey(network: Network, hex: &str) -> Result<Self, WalletError> {
        let privkey = Hex::decode(hex)
            .map_err(|e| WalletError::BtcNewFromHexPrivateKeyHexDecodeFail(e))?;

        Self::from_bytes(network, &privkey)
    }

    pub fn from_wif(network: Network, wif: &str) -> Result<Self, WalletError> {
        let private_key = PrivateKey::from_wif(wif)
            .map_err(|e| WalletError::BtcNewFromHexPrivateKeyWifImportFail(e.to_string()))?;

        let secret_key = private_key.inner;
        Self::from_bytes(network, &secret_key.secret_bytes())
    }

    pub fn from_bytes(network: Network, bytes: &[u8]) -> Result<Self, WalletError> {
        let privkey = PrivateKey::from_slice(&bytes, network)
            .map_err(|e| WalletError::BtcNewFromPrivateKeyImportFail(e.to_string()))?;

        let secp = Secp256k1::new();
        let secret_key = privkey.inner;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);

        Ok(WalletBitcoin {
            network,
            sk: secret_key,
            pubkey: public_key,
        })
    }

    pub fn export(&self, addr_type: AddrTypeBtc) -> Result<Wallet, WalletError> {
        // private key ( WIF )
        let private_key = PrivateKey::new(self.sk, self.network);
        let wif_private_key = private_key.to_string();

        // public key ( Hex )
        let pubkey = self.pubkey.serialize();
        let hex_pubkey = Hex::encode(&pubkey);

        // wallet address
        let address = self.gen_address(addr_type)?;

        Ok(Wallet::new(wif_private_key, hex_pubkey, address))
    }

    pub fn private_key_to_address(
        network: Network,
        privkey: &[u8],
        addr_type: AddrTypeBtc,
    ) -> Result<String, WalletError> {
        let wallet = WalletBitcoin::from_bytes(network, privkey)?;
        let address = wallet.gen_address(addr_type)?;

        Ok(address)
    }

    pub fn validate_address(network: Network, address: &str) -> bool {
        match address.parse::<Address<NetworkUnchecked>>() {
            Ok(address) => address.require_network(network).is_ok(),
            Err(_) => false,
        }
    }

    fn gen_address(&self, addr_type: AddrTypeBtc) -> Result<String, WalletError> {
        let secp = Secp256k1::new();

        // compress public key and convert to bitcoin public key object
        let serialized = self.pubkey.serialize(); // [u8; 33]
        let pubkey = CompressedPublicKey::from_slice(&serialized)
            .map_err(|e| WalletError::BtcGenerateAddressPubkeyCompressFail(e.to_string()))?;

        // create bitcoin address by type
        let address = match addr_type {
            // P2PKH
            AddrTypeBtc::Legacy => Address::p2pkh(&pubkey, self.network),
            // P2SH-WPKH
            AddrTypeBtc::P2SH => Address::p2shwpkh(&pubkey, self.network),
            // P2WPKH
            AddrTypeBtc::Bech32 => Address::p2wpkh(&pubkey, self.network),
            // P2TR
            AddrTypeBtc::Taproot => {
                // create key pair
                let key_pair = Keypair::from_secret_key(&secp, &self.sk);
                let (xonly_pub, _parity) = XOnlyPublicKey::from_keypair(&key_pair);

                // extract address
                Address::p2tr(&secp, xonly_pub, None, self.network)
            }
        };

        Ok(address.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_address() {
        // mainnet (bitcoin)
        {
            let wallet_btc = WalletBitcoin::new(Network::Bitcoin);
            let wallet = wallet_btc
                .export(AddrTypeBtc::Legacy)
                .expect("Failed to export legacy address (mainnet)");

            // P2PKH mainnet: '1' start
            assert!(
                wallet.address.starts_with('1'),
                "Legacy P2PKH on mainnet should start with '1'"
            );
        }

        // testnet
        {
            let wallet_btc = WalletBitcoin::new(Network::Testnet);
            let wallet = wallet_btc
                .export(AddrTypeBtc::Legacy)
                .expect("Failed to export legacy address (testnet)");

            // P2PKH testnet: 'm' or 'n' start
            assert!(
                wallet.address.starts_with('m') || wallet.address.starts_with('n'),
                "Legacy P2PKH on testnet should start with 'm' or 'n'"
            );
        }
    }

    #[test]
    fn test_p2sh_address() {
        // mainnet (bitcoin)
        {
            let wallet_btc = WalletBitcoin::new(Network::Bitcoin);
            let wallet = wallet_btc
                .export(AddrTypeBtc::P2SH)
                .expect("Failed to export p2sh-wpkh address (mainnet)");

            // P2SH mainnet: '3' start
            assert!(
                wallet.address.starts_with('3'),
                "P2SH on mainnet should start with '3'"
            );
        }

        // testnet
        {
            let wallet_btc = WalletBitcoin::new(Network::Testnet);
            let wallet = wallet_btc
                .export(AddrTypeBtc::P2SH)
                .expect("Failed to export p2sh-wpkh address (testnet)");

            // P2SH testnet: '2' start
            assert!(
                wallet.address.starts_with('2'),
                "P2SH on testnet should start with '2'"
            );
        }
    }

    #[test]
    fn test_bech32_address() {
        // mainnet (bitcoin)
        {
            let wallet_btc = WalletBitcoin::new(Network::Bitcoin);
            let wallet = wallet_btc
                .export(AddrTypeBtc::Bech32)
                .expect("Failed to export bech32 address (mainnet)");

            // Bech32 P2WPKH mainnet: 'bc1q' start
            assert!(
                wallet.address.starts_with("bc1q"),
                "Bech32 (P2WPKH) on mainnet should start with 'bc1q'"
            );
        }

        // testnet
        {
            let wallet_btc = WalletBitcoin::new(Network::Testnet);
            let wallet = wallet_btc
                .export(AddrTypeBtc::Bech32)
                .expect("Failed to export bech32 address (testnet)");

            // Bech32 P2WPKH testnet: 'tb1q' start
            assert!(
                wallet.address.starts_with("tb1q"),
                "Bech32 (P2WPKH) on testnet should start with 'tb1q'"
            );
        }
    }

    #[test]
    fn test_taproot_address() {
        // mainnet (bitcoin)
        {
            let wallet_btc = WalletBitcoin::new(Network::Bitcoin);
            let wallet = wallet_btc
                .export(AddrTypeBtc::Taproot)
                .expect("Failed to export taproot address (mainnet)");

            // Taproot(P2TR) mainnet: 'bc1p' start
            assert!(
                wallet.address.starts_with("bc1p"),
                "Taproot on mainnet should start with 'bc1p'"
            );
        }

        // testnet
        {
            let wallet_btc = WalletBitcoin::new(Network::Testnet);
            let wallet = wallet_btc
                .export(AddrTypeBtc::Taproot)
                .expect("Failed to export taproot address (testnet)");

            // Taproot(P2TR) testnet: 'tb1p' start
            assert!(
                wallet.address.starts_with("tb1p"),
                "Taproot on testnet should start with 'tb1p'"
            );
        }
    }

    #[test]
    fn test_import_private_key() {
        let privkey = [
            81, 254, 180, 169, 150, 242, 22, 213, 65, 134, 43, 55, 37, 53, 33, 116, 93, 238, 52,
            252, 230, 198, 74, 228, 193, 176, 125, 9, 70, 76, 205, 203,
        ];

        let wallet_btc = WalletBitcoin::from_bytes(Network::Testnet, &privkey)
            .expect("Failed to create wallet from private key");

        let wallet = wallet_btc
            .export(AddrTypeBtc::Bech32)
            .expect("Failed to export legacy address (testnet)");
        println!("(Legacy Testnet) TWallet = {:?}", wallet);
    }

    #[test]
    fn test_import_hex_private_key() {
        let privkey = [
            81, 254, 180, 169, 150, 242, 22, 213, 65, 134, 43, 55, 37, 53, 33, 116, 93, 238, 52,
            252, 230, 198, 74, 228, 193, 176, 125, 9, 70, 76, 205, 203,
        ];
        let hex_privkey = Hex::encode(&privkey);

        let wallet_btc = WalletBitcoin::from_privkey(Network::Testnet, &hex_privkey)
            .expect("Failed to create wallet from private key");

        let wallet = wallet_btc
            .export(AddrTypeBtc::Bech32)
            .expect("Failed to export legacy address (testnet)");
        println!("(Legacy Testnet) TWallet = {:?}", wallet);
    }

    #[test]
    fn test_import_wif_private_key() {
        let wallet_btc = WalletBitcoin::from_wif(
            Network::Testnet,
            "cQL67ycAaLqiUP4qh4kZ753DWmf37ijZaKSBdVFeqfxTsZWEdiLz",
        )
        .expect("Failed to create wallet from private key");

        let wallet = wallet_btc
            .export(AddrTypeBtc::Bech32)
            .expect("Failed to export legacy address (testnet)");
        println!("(Legacy Testnet) TWallet = {:?}", wallet);
    }

    #[test]
    fn test_private_key_to_address() {
        let privkey = [
            81, 254, 180, 169, 150, 242, 22, 213, 65, 134, 43, 55, 37, 53, 33, 116, 93, 238, 52,
            252, 230, 198, 74, 228, 193, 176, 125, 9, 70, 76, 205, 203,
        ];

        let address =
            WalletBitcoin::private_key_to_address(Network::Testnet, &privkey, AddrTypeBtc::Bech32)
                .expect("Failed to create wallet from private key");

        println!("(Legacy Testnet) TWallet = {:?}", address);
    }
}
