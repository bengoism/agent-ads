use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn appsecret_proof(access_token: &str, app_secret: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(app_secret.as_bytes())
        .expect("HMAC can be initialized from any key size");
    mac.update(access_token.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

#[cfg(test)]
mod tests {
    use super::appsecret_proof;

    #[test]
    fn computes_proof() {
        let proof = appsecret_proof("abc123", "secret");
        assert_eq!(
            proof,
            "5ae5ac802a1a5c94fb683e1bfa121f9f700a26995213ff2fc1c503eb43ec71c6"
        );
    }
}
