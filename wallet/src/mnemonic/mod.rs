use bip39::{Language, Mnemonic as Bip39Mnemonic, MnemonicType, Seed};

#[derive(Debug, PartialEq)]
pub enum MnemonicError {
    FromBytesEntropyEmpty,
    FromBytesEntropyLenNotSame16,
    FromStrMnemonicEmpty,
    ExportSeedFail(String),
    ConvertMnemonicToEntropyFail(String),
    ConvertEntropyToMnemonicFail(String),
}

#[derive(Debug)]
pub struct Mnemonic {
    entropy: Vec<u8>,
}

impl Mnemonic {
    pub fn gen(&self) -> String {
        let bip39_mnemonic = Bip39Mnemonic::new(MnemonicType::Words12, Language::English);
        bip39_mnemonic.phrase().to_string()
    }

    pub fn from_bytes(entropy: Vec<u8>) -> Result<Self, MnemonicError> {
        if entropy.is_empty() {
            return Err(MnemonicError::FromBytesEntropyEmpty);
        }

        if entropy.len() != 16 {
            return Err(MnemonicError::FromBytesEntropyLenNotSame16);
        }

        Ok(Self { entropy })
    }

    pub fn from_str(mnemonic: &str) -> Result<Self, MnemonicError> {
        if mnemonic.is_empty() || mnemonic == "" {
            return Err(MnemonicError::FromStrMnemonicEmpty);
        }

        let entropy = Self::convert_mnemonic_to_entropy(mnemonic)?;
        Ok(Self { entropy })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.entropy.clone()
    }

    pub fn to_str(&self) -> Result<String, MnemonicError> {
        let mnemonic = Self::convert_entropy_to_mnemonic(&self.entropy)?;
        Ok(mnemonic)
    }

    pub fn export_seed(&self, pwd: &str) -> Result<Vec<u8>, MnemonicError> {
        let bip39_mnemonic = Bip39Mnemonic::from_entropy(&self.entropy, Language::English)
            .map_err(|e| MnemonicError::ExportSeedFail(e.to_string()))?;

        let seed = Seed::new(&bip39_mnemonic, pwd);
        Ok(seed.as_bytes().to_vec())
    }

    pub fn is_validate(mnemonic: &str) -> bool {
        let result = Bip39Mnemonic::validate(mnemonic, Language::English);
        result.is_ok()
    }

    fn convert_entropy_to_mnemonic(entropy: &Vec<u8>) -> Result<String, MnemonicError> {
        let bip39_mnemonic = Bip39Mnemonic::from_entropy(entropy, Language::English)
            .map_err(|e| MnemonicError::ConvertEntropyToMnemonicFail(e.to_string()))?;

        Ok(bip39_mnemonic.phrase().to_string())
    }

    fn convert_mnemonic_to_entropy(mnemonic: &str) -> Result<Vec<u8>, MnemonicError> {
        let bip39_mnemonic = Bip39Mnemonic::from_phrase(mnemonic, Language::English)
            .map_err(|e| MnemonicError::ConvertMnemonicToEntropyFail(e.to_string()))?;

        Ok(bip39_mnemonic.entropy().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use crate::mnemonic::*;

    #[test]
    fn from_bytes() {
        // ok
        {
            let entropy = vec![
                234, 97, 165, 255, 4, 230, 146, 17, 184, 3, 203, 181, 91, 48, 185, 55,
            ];

            let result = Mnemonic::from_bytes(entropy);
            assert_eq!(result.is_ok(), true);
        }

        // fail - length entropy
        {
            let entropy = vec![
                234, 97, 165, 255, 4, 230, 146, 17, 184, 3, 203, 181, 91, 48, 185,
            ];

            let result = Mnemonic::from_bytes(entropy);
            assert_eq!(
                result.unwrap_err(),
                MnemonicError::FromBytesEntropyLenNotSame16
            );
        }

        // fail - empty binary
        {
            let entropy: Vec<u8> = vec![];
            let result = Mnemonic::from_bytes(entropy);
            assert_eq!(result.unwrap_err(), MnemonicError::FromBytesEntropyEmpty);
        }
    }

    #[test]
    fn from_str() {
        // ok
        {
            let mnemonic =
                "tuna artwork lemon antenna hard angle theme just relief sunset comic huge";

            let is_ok = Mnemonic::is_validate(mnemonic);
            assert_eq!(is_ok, true);

            let result = Mnemonic::from_str(mnemonic);
            assert_eq!(result.is_ok(), true);
        }

        // fail - mnemonic length
        {
            let mnemonic = "tuna artwork lemon antenna hard angle theme just relief sunset comic";

            let result = Mnemonic::from_str(mnemonic);
            assert_eq!(
                result.unwrap_err(),
                MnemonicError::ConvertMnemonicToEntropyFail(
                    "invalid number of words in phrase: 11".to_string()
                )
            );
        }

        // fail - mnemonic empty
        {
            let mnemonic = "";

            let result = Mnemonic::from_str(mnemonic);
            assert_eq!(result.unwrap_err(), MnemonicError::FromStrMnemonicEmpty);
        }
    }

    #[test]
    fn to_bytes() {
        let def_entropy = vec![
            234, 97, 165, 255, 4, 230, 146, 17, 184, 3, 203, 181, 91, 48, 185, 55,
        ];

        // import - mnemonic
        let mnemonic = "tuna artwork lemon antenna hard angle theme just relief sunset comic huge";

        let result = Mnemonic::from_str(mnemonic);
        assert_eq!(result.is_ok(), true);
        let mnemonic = result.unwrap();

        // export - entropy
        let entropy = mnemonic.to_bytes();
        assert_eq!(entropy, def_entropy);
    }

    #[test]
    fn to_str() {
        let def_mnemonic =
            "tuna artwork lemon antenna hard angle theme just relief sunset comic huge";

        // import - mnemonic
        let entropy = vec![
            234, 97, 165, 255, 4, 230, 146, 17, 184, 3, 203, 181, 91, 48, 185, 55,
        ];

        let result = Mnemonic::from_bytes(entropy);
        assert_eq!(result.is_ok(), true);
        let mnemonic = result.unwrap();

        // export - mnemonic string
        let ret_mnemonic = mnemonic.to_str();
        assert_eq!(ret_mnemonic.is_ok(), true);
        assert_eq!(ret_mnemonic.unwrap(), def_mnemonic);
    }

    #[test]
    fn export_seed() {
        let def_seed = vec![
            142, 204, 157, 53, 198, 157, 49, 170, 6, 161, 204, 205, 239, 182, 27, 169, 85, 201,
            237, 60, 30, 196, 95, 179, 165, 92, 88, 82, 88, 222, 56, 108, 182, 136, 132, 246, 235,
            75, 201, 15, 134, 138, 76, 76, 27, 143, 43, 116, 125, 83, 110, 207, 215, 34, 159, 181,
            52, 46, 148, 249, 45, 140, 59, 228,
        ];

        // import - entropy
        let entropy = vec![
            234, 97, 165, 255, 4, 230, 146, 17, 184, 3, 203, 181, 91, 48, 185, 55,
        ];

        let result = Mnemonic::from_bytes(entropy);
        assert_eq!(result.is_ok(), true);
        let mnemonic = result.unwrap();

        // export - seed
        let pwd = "test";

        let ret_seed = mnemonic.export_seed(pwd);
        assert_eq!(ret_seed.is_ok(), true);
        assert_eq!(ret_seed.unwrap(), def_seed);
    }
}
