use std::str::FromStr;

use elements::bitcoin::bip32::DerivationPath;
use thiserror::Error;

use crate::Signer;

// TODO impl error handling
pub fn singlesig_desc<S: Signer>(
    signer: &S,
    script_variant: Singlesig,
    blinding_variant: DescriptorBlindingKey,
) -> Result<String, String> {
    let (prefix, path, suffix) = match script_variant {
        Singlesig::Wpkh => ("elwpkh", "84h/1h/0h", ""),
        Singlesig::ShWpkh => ("elsh(wpkh", "49h/1h/0h", ")"),
    };

    let master = signer
        .derive_xpub(&DerivationPath::master())
        .map_err(|e| format!("{:?}", e))?;
    let fingerprint = master.fingerprint();

    let xpub = signer
        .derive_xpub(
            &DerivationPath::from_str(&format!("m/{path}")).map_err(|e| format!("{:?}", e))?,
        )
        .map_err(|e| format!("{:?}", e))?;

    let blinding_key = match blinding_variant {
        DescriptorBlindingKey::Slip77 => format!(
            "slip77({})",
            signer
                .slip77_master_blinding_key()
                .map_err(|e| format!("{:?}", e))?
        ),
    };

    // m / purpose' / coin_type' / account' / change / address_index
    Ok(format!(
        "ct({blinding_key},{prefix}([{fingerprint}/{path}]{xpub}/<0;1>/*){suffix})"
    ))
}

pub enum Singlesig {
    /// as defined by bip84
    Wpkh,

    // as defined by bip49
    ShWpkh,
}

#[derive(Error, Debug)]
#[error("Invalid singlesig variant '{0}' supported variant are: 'wpkh', 'shwpkh'")]
pub struct InvalidSinglesigVariant(String);

impl FromStr for Singlesig {
    type Err = InvalidSinglesigVariant;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "wpkh" => Singlesig::Wpkh,
            "shwpkh" => Singlesig::ShWpkh,
            v => return Err(InvalidSinglesigVariant(v.to_string())),
        })
    }
}

pub enum DescriptorBlindingKey {
    Slip77,
}

#[derive(Error, Debug)]
#[error("Invalid blinding key variant '{0}' supported variant are: 'slip77'")]
pub struct InvalidBlindingKeyVariant(String);

impl FromStr for DescriptorBlindingKey {
    type Err = InvalidBlindingKeyVariant;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "slip77" => DescriptorBlindingKey::Slip77,
            v => return Err(InvalidBlindingKeyVariant(v.to_string())),
        })
    }
}

pub enum Bip {
    Bip84,
}

#[derive(Error, Debug)]
#[error("Invalid bip  variant '{0}' supported variant are: 'bip84'")]
pub struct InvalidBipVariant(String);

impl FromStr for Bip {
    type Err = InvalidBipVariant;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "bip84" => Bip::Bip84,
            v => return Err(InvalidBipVariant(v.to_string())),
        })
    }
}
