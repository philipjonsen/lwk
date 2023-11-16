mod software;

pub use crate::software::{NewError, SignError, SwSigner};

use common::Signer;
use elements_miniscript::bitcoin::bip32::DerivationPath;
use elements_miniscript::elements;
use elements_miniscript::elements::bitcoin::bip32::{ExtendedPubKey, Fingerprint};
use elements_miniscript::elements::bitcoin::hash_types::XpubIdentifier;
use elements_miniscript::elements::pset::PartiallySignedTransaction;
use jade::mutex_jade::MutexJade;

#[derive(thiserror::Error, Debug)]
pub enum SignerError {
    #[error(transparent)]
    Software(#[from] SignError),

    #[error(transparent)]
    Jade(#[from] jade::sign_pset::Error),

    #[error(transparent)]
    JadeError(#[from] jade::error::Error),

    #[error(transparent)]
    Bip32Error(#[from] elements::bitcoin::bip32::Error),
}

pub enum AnySigner {
    Software(SwSigner),
    Jade(MutexJade),
}

impl AnySigner {
    pub fn xpub(&self) -> Result<ExtendedPubKey, SignerError> {
        match self {
            AnySigner::Software(s) => Ok(s.xpub()),
            AnySigner::Jade(s) => {
                let params = jade::protocol::GetXpubParams {
                    network: jade::Network::LocaltestLiquid,
                    path: vec![],
                };
                Ok(s.get_xpub(params)?)
            }
        }
    }

    pub fn id(&self) -> Result<XpubIdentifier, SignerError> {
        Ok(self.xpub()?.identifier())
    }

    pub fn fingerprint(&self) -> Result<Fingerprint, SignerError> {
        Ok(self.xpub()?.fingerprint())
    }
}

impl Signer for SwSigner {
    type Error = SignError;

    fn sign(&self, pset: &mut PartiallySignedTransaction) -> Result<u32, Self::Error> {
        self.sign_pset(pset)
    }

    fn derive_xpub(&self, path: &DerivationPath) -> Result<ExtendedPubKey, Self::Error> {
        let derived = self.xprv.derive_priv(&self.secp, path)?;
        Ok(ExtendedPubKey::from_priv(&self.secp, &derived))
    }

    fn slip77(&self) -> Result<elements_miniscript::slip77::MasterBlindingKey, Self::Error> {
        Ok(self.slip77())
    }
}

impl Signer for AnySigner {
    type Error = SignerError;

    fn sign(&self, pset: &mut PartiallySignedTransaction) -> Result<u32, Self::Error> {
        Signer::sign(&self, pset)
    }

    fn derive_xpub(&self, path: &DerivationPath) -> Result<ExtendedPubKey, Self::Error> {
        Signer::derive_xpub(&self, path)
    }

    fn slip77(&self) -> Result<elements_miniscript::slip77::MasterBlindingKey, Self::Error> {
        Signer::slip77(&self)
    }
}

impl Signer for &AnySigner {
    type Error = SignerError;

    fn sign(&self, pset: &mut PartiallySignedTransaction) -> Result<u32, Self::Error> {
        Ok(match self {
            AnySigner::Software(signer) => signer.sign_pset(pset)?,
            AnySigner::Jade(signer) => signer.sign_pset(pset)?,
        })
    }

    fn derive_xpub(&self, path: &DerivationPath) -> Result<ExtendedPubKey, Self::Error> {
        Ok(match self {
            AnySigner::Software(s) => s.derive_xpub(path)?,
            AnySigner::Jade(s) => s.derive_xpub(path)?,
        })
    }

    fn slip77(&self) -> Result<elements_miniscript::slip77::MasterBlindingKey, Self::Error> {
        Ok(match self {
            AnySigner::Software(s) => s.slip77(),
            AnySigner::Jade(s) => s.slip77()?,
        })
    }
}
