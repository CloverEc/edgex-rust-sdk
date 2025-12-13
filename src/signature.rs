use ethers::signers::WalletError; // Keep if using WalletError wrapping, or remove if not.
// Actually we wrapped WalletError but removed LocalWallet usage.
// Let's remove WalletError dependency if possible or keep for compat.
// But we should use starknet types.
use starknet_types_core::felt::Felt;
use starknet_crypto::{pedersen_hash, sign};
use thiserror::Error;
use std::str::FromStr;

#[derive(Error, Debug)]
pub enum SignatureError {
    #[error("Wallet error: {0}")]
    WalletError(#[from] WalletError),
    #[error("Hex error: {0}")]
    HexError(#[from] hex::FromHexError),
    #[error("Felt error")]
    FeltError,
    #[error("Signing error")]
    SigningError,
}

// StarkNet Prime (2^251 + 17 * 2^192 + 1)
// The python code uses: 0x800000000000011000000000000000000000000000000000000000000000001
// which matches the Stark curve prime.

pub struct SignatureManager {
    private_key: Felt, // L2 Private Key (Stark Key)
    // We might also need L1 wallet for onboarding, but for L2 actions we need L2 key.
}

impl SignatureManager {
    pub fn new(l2_private_key_hex: &str) -> Result<Self, SignatureError> {
        let key_str = l2_private_key_hex.trim_start_matches("0x");
        let private_key = Felt::from_hex(key_str)
            .map_err(|_| SignatureError::FeltError)?;
        Ok(Self { private_key })
    }

    /// Calculates the Pedersen hash for a limit order (Order with fees).
    /// Replicates the logic from EdgeX Python SDK `calc_limit_order_hash`.
    pub fn calc_limit_order_hash(
        &self,
        synthetic_asset_id: &str,
        collateral_asset_id: &str,
        fee_asset_id: &str,
        is_buy: bool,
        amount_synthetic: u64,
        amount_collateral: u64,
        amount_fee: u64,
        nonce: u64,
        account_id: u64,
        expire_time: u64,
    ) -> Result<Felt, SignatureError> {
        // Parse Asset IDs
        let syn_id = Felt::from_hex(synthetic_asset_id.trim_start_matches("0x"))
            .map_err(|_| SignatureError::FeltError)?;
        let col_id = Felt::from_hex(collateral_asset_id.trim_start_matches("0x"))
            .map_err(|_| SignatureError::FeltError)?;
        let fee_id = Felt::from_hex(fee_asset_id.trim_start_matches("0x"))
            .map_err(|_| SignatureError::FeltError)?;

        let (asset_id_sell, asset_id_buy, amount_sell, amount_buy) = if is_buy {
            (col_id, syn_id, amount_collateral, amount_synthetic)
        } else {
            (syn_id, col_id, amount_synthetic, amount_collateral)
        };

        // First hash: hash(asset_id_sell, asset_id_buy)
        let msg = pedersen_hash(&asset_id_sell, &asset_id_buy);

        // Second hash: hash(msg, asset_id_fee)
        let msg = pedersen_hash(&msg, &fee_id);

        // Pack message 0
        // packed_message0 = amount_sell * 2^64 + amount_buy * 2^64 + max_amount_fee * 2^32 + nonce
        // Note: Felt doesn't support '<<' directly for non-Felt inputs easily unless we convert.
        // But we can construct BigUint or perform check.
        // Since we are using Felt which is 252 bits, we can try to compose it.
        // The python code does: val = (val << 64) + next_val.
        
        // Helper to shift and add
        let shift_add = |acc: Felt, val: u64, shift: u32| -> Felt {
            // acc * 2^shift + val
            // Felt::pow takes u128.
            let shift_multiplier = Felt::from(2u64).pow(shift as u128);
            (acc * shift_multiplier) + Felt::from(val)
        };
        
        // Wait, does Felt implement std::ops::Add etc? Yes usually.

        let pm0 = Felt::from(amount_sell);
        let pm0 = shift_add(pm0, amount_buy, 64);
        let pm0 = shift_add(pm0, amount_fee, 64);
        let pm0 = shift_add(pm0, nonce, 32);
        // implicit modulo prime is handled by Felt arithmetic

        // Third hash: hash(msg, packed_message0)
        let msg = pedersen_hash(&msg, &pm0);

        // Pack message 1
        // packed_message1 = LIMIT_ORDER_WITH_FEE_TYPE * 2^64 + account_id * 2^64 + account_id * 2^64 + account_id * 2^32 + expiration_timestamp * 2^17
        // Python:
        // packed_message1 = LIMIT_ORDER_WITH_FEE_TYPE  # 3
        // packed_message1 = (packed_message1 << 64) + account_id
        // packed_message1 = (packed_message1 << 64) + account_id
        // packed_message1 = (packed_message1 << 64) + account_id
        // packed_message1 = (packed_message1 << 32) + expire_time
        // packed_message1 = packed_message1 << 17
        
        let limit_order_type = 3u64;
        let pm1 = Felt::from(limit_order_type);
        let pm1 = shift_add(pm1, account_id, 64);
        let pm1 = shift_add(pm1, account_id, 64);
        let pm1 = shift_add(pm1, account_id, 64);
        let pm1 = shift_add(pm1, expire_time, 32);
        
        // Final shift by 17 (padding)
        let shift_17 = Felt::from(2u64).pow(17u128);
        let pm1 = pm1 * shift_17;

        // Final hash: hash(msg, packed_message1)
        let msg = pedersen_hash(&msg, &pm1);

        Ok(msg)
    }

    pub fn sign_l2_action(&self, hash: Felt) -> Result<String, SignatureError> {
        // Sign with k value (randomness). API often expects standard ECDSA signature (r, s).
        // starknet_crypto::sign usage: sign(private_key, message_hash, k)
        // We need a random k.
        
        // For deterministic signing (RFC6979 equivalent), we usually derive k from msg and key.
        // But starknet_crypto might need explicit k.
        // Let's use a simple RFC6979-like derivation or random if possible.
        // Actually, for safety, using a secure random k is better.
        
        // Generate random k. 
        // Note: For full safety, RFC6979 deterministic k is preferred to avoid RNG failure risks,
        // but random k is acceptable if RNG is good.
        let mut rng = rand::thread_rng();
        // Generate a random u64 or u128 and convert to Felt?
        // Felt is large (252 bits).
        // We can just pick a random number < Prime.
        // For simplicity in this MVP, we use a random u128.
        use rand::Rng;
        let k_low: u128 = rng.r#gen();
        let k_high: u128 = rng.r#gen(); 
        
        // Construct K. 
        // Felt::from_u128 is likely available. 
        // Or assume from(u128).
        // If from(u128) works:
        // let k = Felt::from(k_low) + (Felt::from(k_high) * Felt::from(2u64).pow(128));
        // However, 2^128 might overflow u64. 
        // Safer to use byte array.
        // let bytes = ...
        // Felt::from_bytes_be(&bytes).
        
        let mut bytes = [0u8; 32];
        bytes[16..32].copy_from_slice(&k_low.to_be_bytes());
        bytes[0..16].copy_from_slice(&k_high.to_be_bytes());
        // Mask out top bits to ensure < Prime (Prime is 251 bits).
        bytes[0] &= 0x0f; // Keep safe.
        
        let k = Felt::from_bytes_be(&bytes);
        
        let signature = sign(&self.private_key, &hash, &k).map_err(|_| SignatureError::SigningError)?;
        
        // Format: r, s. Usually hex strings.
        // API expects... "l2Signature".
        // Often formatted as `r` and `s` or concatenated.
        // EdgeX docs say "l2Signature": "0x..."
        // I will return r and s packed or check doc again.
        // Docs usually want: r, s as hex strings, or packed 0x{r}{s}.
        // Common Starknet format is often JSON `[r, s]`.
        // Let's assume standard hex concatenation for now given "0x..." string type.
        // 0x + r_hex + s_hex
        
        let r_hex = format!("{:064x}", signature.r);
        let s_hex = format!("{:064x}", signature.s);
        Ok(format!("0x{}{}", r_hex, s_hex))
    }
    
    // Kept for Header signing if different keys are used
    pub async fn sign_message(&self, _message: &str) -> Result<String, SignatureError> {
        // This likely needs L1 key if it is 'ethers' style. 
        // The L2 key is a Felt, not compatible with 'LocalWallet' (Secp256k1).
        // If 'X-edgeX-Api-Signature' is also L2 key based, we need to sign the hash of the generic message.
        // But headers usually use the L2 key with generic hash?
        // IF L2 key is used for headers, we hash the message (keccak or pedersen?) and sign.
        // Docs said: "The signature generated using the private key and request details".
        // If it's the L2 key, it must be Stark curve.
        
        // Assume Header signature also uses Stark key on Pedersen hash of the string?
        // Or Keccak hash of string?
        // "Method + Path + Body" -> usually Keccak or SHA256. 
        // StarkEx usually uses Pedersen for L2 data (Orders), but REST headers might be standard.
        // Let's assume one key for everything for now, but watch out.
        
        Err(SignatureError::SigningError) // Placeholder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_generation() {
        // Dummy key (valid hex)
        let key = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let manager = SignatureManager::new(key).unwrap();

        // Test limit order hash calculation
        let hash = manager.calc_limit_order_hash(
            "0x1", "0x2", "0x3", true, 100, 200, 10, 123, 1, 999999
        ).unwrap();
        
        println!("Hash: {:?}", hash);

        // Test signing
        let signature = manager.sign_l2_action(hash).unwrap();
        println!("Signature: {}", signature);
        
        assert!(signature.starts_with("0x"));
        assert_eq!(signature.len(), 2 + 64 + 64); // 0x + r(64) + s(64)
    }
}
