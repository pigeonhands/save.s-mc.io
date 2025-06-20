use pgp::{
    armor,
    composed::{
        ArmorOptions, Deserializable, KeyType, Message, MessageBuilder, PublicKey, SignedPublicKey,
        SignedPublicSubKey,
    },
    crypto::{ecc_curve::ECCCurve, sym::SymmetricKeyAlgorithm},
    types::{Password, PublicKeyTrait, StringToKey},
};

fn get_encryption_key(pub_key_armor: String) -> Result<SignedPublicSubKey, String> {
    let (public_key, _headers_public) =
        SignedPublicKey::from_string(&pub_key_armor).map_err(|_| String::from("bad pub key"))?;

    for key in public_key.public_subkeys {
        if key.is_encryption_key() {
            return Ok(key);
        }
    }

    Err(String::from("No encryptin key in pub key"))
}

pub fn encrypt(
    pub_key_armor: String,
    description: String,
    message: String,
) -> Result<String, String> {
    let mut rng = rand::thread_rng();

    let enc_key = get_encryption_key(pub_key_armor)?;

    let mut builder =
        MessageBuilder::from_bytes("encrypted_message", Vec::from(message.as_bytes()))
            .seipd_v1(&mut rng, SymmetricKeyAlgorithm::AES128);

    builder.compression(pgp::types::CompressionAlgorithm::ZLIB);

    builder
        .encrypt_to_key(&mut rng, &enc_key)
        .map_err(|e| format!("failed to encrypt. {}", e.to_string()))?;

    let headers = {
        let mut map = armor::Headers::new();
        map.insert("description".into(), vec![description]);
        map
    };
    let enc_message = builder
        .to_armored_string(
            &mut rng,
            ArmorOptions {
                headers: Some(&headers),
                ..Default::default()
            },
        )
        .map_err(|_| String::from("failed to create armor"))?;

    Ok(enc_message)
}
