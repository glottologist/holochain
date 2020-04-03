use crate::*;

/// create a new insecure byte buffer
pub fn crypto_insecure_buffer(size: usize) -> CryptoResult<DynCryptoBytes> {
    Ok(InsecureBytes::new(size))
}

/// create an insecure buffer from bytes
pub fn crypto_insecure_buffer_from_bytes(o: &[u8]) -> CryptoResult<DynCryptoBytes> {
    let mut out = crypto_insecure_buffer(o.len())?;
    out.copy_from(0, o)?;
    Ok(out)
}

/// create a new secure byte buffer (i.e. for use with private keys)
pub fn crypto_secure_buffer(size: usize) -> CryptoResult<DynCryptoBytes> {
    plugin::get_global_crypto_plugin()?.secure_buffer(size)
}

/// DANGER - create a secure buffer from bytes.
/// This is dangerous, because if your data is in a `&[u8]` reference,
/// it's probably already insecure.
pub fn danger_crypto_secure_buffer_from_bytes(o: &[u8]) -> CryptoResult<DynCryptoBytes> {
    let mut out = crypto_secure_buffer(o.len())?;
    out.copy_from(0, o)?;
    Ok(out)
}

/// randomize a byte buffer
pub async fn crypto_randombytes_buf(buf: &mut DynCryptoBytes) -> CryptoResult<()> {
    plugin::get_global_crypto_plugin()?
        .randombytes_buf(buf)
        .await
}

/// minimum size of output generic (blake2b) hash
pub fn crypto_generic_hash_min_bytes() -> CryptoResult<usize> {
    Ok(plugin::get_global_crypto_plugin()?.generic_hash_min_bytes())
}

/// maximum size of output generic (blake2b) hash
pub fn crypto_generic_hash_max_bytes() -> CryptoResult<usize> {
    Ok(plugin::get_global_crypto_plugin()?.generic_hash_max_bytes())
}

/// minimum size of generic hash key
pub fn crypto_generic_hash_key_min_bytes() -> CryptoResult<usize> {
    Ok(plugin::get_global_crypto_plugin()?.generic_hash_key_min_bytes())
}

/// maximum size of generic hash key
pub fn crypto_generic_hash_key_max_bytes() -> CryptoResult<usize> {
    Ok(plugin::get_global_crypto_plugin()?.generic_hash_key_max_bytes())
}

/// calculate the generic (blake2b) hash for the given data
/// with the optional blake2b key
pub async fn crypto_generic_hash(
    size: usize,
    data: &mut DynCryptoBytes,
    key: Option<&mut DynCryptoBytes>,
) -> CryptoResult<DynCryptoBytes> {
    plugin::get_global_crypto_plugin()?
        .generic_hash(size, data, key)
        .await
}

/// size of seed needed for signature keys
pub fn crypto_sign_seed_bytes() -> CryptoResult<usize> {
    Ok(plugin::get_global_crypto_plugin()?.sign_seed_bytes())
}

/// size of signature public key
pub fn crypto_sign_public_key_bytes() -> CryptoResult<usize> {
    Ok(plugin::get_global_crypto_plugin()?.sign_public_key_bytes())
}

/// size of signature secret key
pub fn crypto_sign_secret_key_bytes() -> CryptoResult<usize> {
    Ok(plugin::get_global_crypto_plugin()?.sign_secret_key_bytes())
}

/// size of an actual signature
pub fn crypto_sign_bytes() -> CryptoResult<usize> {
    Ok(plugin::get_global_crypto_plugin()?.sign_bytes())
}

/// generate a signature keypair optionally based off a seed
pub async fn crypto_sign_keypair(
    seed: Option<&mut DynCryptoBytes>,
) -> CryptoResult<(DynCryptoBytes, DynCryptoBytes)> {
    plugin::get_global_crypto_plugin()?.sign_keypair(seed).await
}

/// generate a signature from message data
pub async fn crypto_sign(
    message: &mut DynCryptoBytes,
    secret_key: &mut DynCryptoBytes,
) -> CryptoResult<DynCryptoBytes> {
    plugin::get_global_crypto_plugin()?
        .sign(message, secret_key)
        .await
}

/// generate a signature from message data
pub async fn crypto_sign_verify(
    signature: &mut DynCryptoBytes,
    message: &mut DynCryptoBytes,
    public_key: &mut DynCryptoBytes,
) -> CryptoResult<bool> {
    plugin::get_global_crypto_plugin()?
        .sign_verify(signature, message, public_key)
        .await
}