use crate::auth::provider::{
    AuthProviderChangeset, AuthProviderInsert, AuthProviderType, IntoAuthProviderInsert,
};
use crate::user::{User, UserInsert};
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::prelude::*;
use openssl::{hash::MessageDigest, sign::Verifier, x509::X509};
use reqwest::StatusCode;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct IdentitySignature {
    pub public_key_url: String,
    pub signature: String,
    pub salt: String,
    pub timestamp: u64,
    pub player_id: String,
    pub user_name: Option<String>,
    pub bundle_id: String,
}

impl From<&IdentitySignature> for AuthProviderChangeset {
    fn from(value: &IdentitySignature) -> Self {
        AuthProviderChangeset {
            order: 0,
            email: None,
            email_verified: false,
            display_name: None,
            user_name: value.user_name.clone(),
            picture_url: None,
            locale: None,
        }
    }
}

impl From<&IdentitySignature> for UserInsert {
    fn from(value: &IdentitySignature) -> Self {
        UserInsert {
            name: value.user_name.clone(),
        }
    }
}

impl IntoAuthProviderInsert for IdentitySignature {
    fn into_provider_insert(&self, user: &User) -> AuthProviderInsert {
        AuthProviderInsert {
            user_id: user.id,
            order: 0,
            provider_type: AuthProviderType::AppleGameCenter,
            provider_id: self.player_id.clone(),
            email: None,
            email_verified: false,
            display_name: None,
            user_name: self.user_name.clone(),
            picture_url: None,
            locale: None,
        }
    }
}

#[async_trait::async_trait]
pub trait GameCenterIdValidationService {
    async fn is_validated(
        &self,
        identity: &IdentitySignature,
    ) -> Result<bool, Box<dyn std::error::Error>>;
}

pub struct RealGameCenterIdValidationService;

#[async_trait::async_trait]
impl GameCenterIdValidationService for RealGameCenterIdValidationService {
    async fn is_validated(
        &self,
        identity: &IdentitySignature,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let cert = get_certificate(&identity.public_key_url).await?;

        let cert_verified = verify_certificate(&cert)?;
        if !cert_verified {
            return Err("Failed to verify Apple's public key certificate".into());
        }

        let identity_data = get_identity_data(&identity)?;

        let public_key = cert.public_key()?;
        let mut verifier = Verifier::new(MessageDigest::sha256(), &public_key)?;
        verifier.update(&identity_data)?;

        let signature = BASE64_STANDARD.decode(&identity.signature)?;
        Ok(verifier.verify(&signature)?)
    }
}

async fn get_certificate(public_key_url: &str) -> Result<X509, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let resp = client.get(public_key_url).send().await?;
    match resp.status() {
        StatusCode::OK => {
            let bytes = resp.bytes().await?;
            Ok(X509::from_der(&bytes)?)
        }
        _ => {
            let text = resp.text().await?;
            Err(text.into())
        }
    }
}

/// This is a stub implementation until we actually figure out how to load all
/// the certificates required in the verification chain.
fn verify_certificate(_cert: &X509) -> Result<bool, Box<dyn std::error::Error>> {
    Ok(true)
}

fn get_identity_data(identity: &IdentitySignature) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let mut data = Vec::new();
    data.extend_from_slice(identity.player_id.as_bytes());
    data.extend_from_slice(identity.bundle_id.as_bytes());
    data.extend_from_slice(&identity.timestamp.to_be_bytes());
    data.extend_from_slice(&BASE64_STANDARD.decode(&identity.salt)?);
    Ok(data)
}
