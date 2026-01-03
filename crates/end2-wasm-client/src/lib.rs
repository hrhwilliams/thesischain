use base64::{Engine, prelude::BASE64_URL_SAFE};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use vodozemac::olm::Account;

#[derive(Deserialize, Serialize)]
pub struct NewUser {
    pub username: String,
    pub ed25519: Vec<u8>,
    pub curve25519: Vec<u8>,
    pub signature: Vec<u8>,
}

#[wasm_bindgen]
pub struct End2ClientSession {
    account: Account,
}

#[wasm_bindgen]
impl End2ClientSession {
    pub fn new() -> Self {
        Self {
            account: Account::new()
        }
    }

    pub fn get_identity_keys(&self) -> String {
        let keys = self.account.identity_keys();
        serde_json::to_string(&keys).unwrap()
    }

    pub fn generate_otks(&mut self, count: usize) -> String {
        self.account.generate_one_time_keys(count);
        let otks = self.account.one_time_keys();
        serde_json::to_string(&otks).unwrap()
    }

    pub fn get_registration_payload(&self, username: String) -> Result<String, JsValue> {
        let identity_keys = self.account.identity_keys();
        let ed25519 = identity_keys.ed25519;
        let curve25519 = identity_keys.curve25519;

        let mut message = Vec::new();
        message.extend_from_slice(curve25519.as_bytes());
        message.extend_from_slice(ed25519.as_bytes());
        
        let signature = self.account.sign(&message);

        let new_user = NewUser {
            username,
            ed25519: ed25519.as_bytes().to_vec(),
            curve25519: curve25519.to_vec(),
            signature: signature.to_bytes().to_vec(),
        };

        // let payload = serde_json::json!({
        //     "username": username,
        //     "ed25519": BASE64_URL_SAFE.encode(ed25519.as_bytes()),
        //     "curve25519": BASE64_URL_SAFE.encode(curve25519.as_bytes()),
        //     "signature": BASE64_URL_SAFE.encode(signature.to_bytes()),
        // });

        Ok(serde_json::to_string(&new_user).unwrap())
    }
}

#[wasm_bindgen]
pub fn add_rs(a: i32, b: i32) -> i32 {
    a + b
}