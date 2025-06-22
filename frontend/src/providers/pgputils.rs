use leptos::tachys::view::keyed::SerializableKey;
use pgp::{
    armor,
    composed::{ArmorOptions, Deserializable, MessageBuilder, SignedPublicKey, SignedPublicSubKey},
    crypto::sym::SymmetricKeyAlgorithm,
    types::{KeyDetails, PublicKeyTrait},
};

fn get_encryption_key(pub_key_armor: String) -> anyhow::Result<SignedPublicSubKey> {
    let (public_key, _headers_public) = SignedPublicKey::from_string(&pub_key_armor)?;
    public_key.verify()?;

    for key in public_key.public_subkeys {
        if key.is_encryption_key() {
            return Ok(key);
        }
    }

    anyhow::bail!("No encryption key in pub key")
}

pub fn encrypt(
    pub_key_armor: String,
    description: String,
    message: String,
) -> anyhow::Result<String> {
    let mut rng = rand::thread_rng();

    let enc_key = get_encryption_key(pub_key_armor)?;

    let mut builder =
        MessageBuilder::from_bytes("encrypted_message", Vec::from(message.as_bytes()))
            // .seipd_v2(
            //     &mut rng,
            //     SymmetricKeyAlgorithm::AES256,
            //     pgp::crypto::aead::AeadAlgorithm::Gcm,
            //     pgp::crypto::aead::ChunkSize::C128B,
            // );
            .seipd_v1(&mut rng, SymmetricKeyAlgorithm::AES128);

    builder.compression(pgp::types::CompressionAlgorithm::ZLIB);

    builder.encrypt_to_key(&mut rng, &enc_key)?;

    let headers = {
        let mut map = armor::Headers::new();
        map.insert("Comment".into(), vec![description]);
        map
    };
    let enc_message = builder.to_armored_string(
        &mut rng,
        ArmorOptions {
            headers: Some(&headers),
            ..Default::default()
        },
    )?;

    Ok(enc_message)
}

fn extract_email_from_user_id(user_id: &[u8]) -> anyhow::Result<String> {
    let mut user_id_iter = user_id.into_iter().cloned().skip_while(|c| *c != b'<');
    user_id_iter.next(); // skip '<' char
    let email_bytes: Vec<u8> = user_id_iter.take_while(|c| *c != b'>').collect();

    let email = String::from_utf8(email_bytes)?;
    if email.is_empty() {
        anyhow::bail!("No email in uid");
    }

    Ok(email)
}

#[derive(Default, Debug, Clone)]
pub struct PublicKeyInfo {
    pub armored_key: String,
    pub emails: Vec<String>,
    pub encryption_keys: Vec<(String, Option<chrono::DateTime<chrono::Utc>>)>,
}

pub fn get_public_key_info(armored_key: String) -> anyhow::Result<PublicKeyInfo> {
    let mut public_key_info = PublicKeyInfo {
        armored_key,
        ..Default::default()
    };

    let (public_key, _headers_public) = SignedPublicKey::from_string(&public_key_info.armored_key)?;
    public_key.verify()?;

    for user in public_key.details.users {
        let email = match extract_email_from_user_id(user.id.id()) {
            Ok(email) => email,
            Err(e) => {
                log::error!("Failed to extract email from user id. {:?}", e);
                continue;
            }
        };

        public_key_info.emails.push(email);
    }

    for key in public_key.public_subkeys {
        if !key.is_encryption_key() {
            continue;
        }
        let fingerprint = format!("{}", key.fingerprint());

        let mut expires_at = None;
        for sig in &key.signatures {
            if let Some(sig_expr) = sig.key_expiration_time() {
                let sig_expires_at_datetime = (*key.created_at()) + (*sig_expr);

                match expires_at {
                    None => expires_at = Some(sig_expires_at_datetime),
                    Some(current_exp) => {
                        expires_at = if current_exp > sig_expires_at_datetime {
                            Some(current_exp)
                        } else {
                            Some(sig_expires_at_datetime)
                        };
                    }
                }
            }
            //
        }

        public_key_info
            .encryption_keys
            .push((fingerprint, expires_at));
    }

    Ok(public_key_info)
}
