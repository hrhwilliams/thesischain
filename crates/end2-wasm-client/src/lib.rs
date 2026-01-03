use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use uuid::Uuid;
use vodozemac::olm::Account;
use wasm_bindgen::prelude::*;

#[derive(Deserialize, Serialize)]
pub struct NewUser {
    pub username: String,
    pub ed25519: String,
    pub curve25519: String,
    pub signature: String,
}

#[derive(Deserialize)]
struct ChallengeInput {
    id: String,
    user_id: String,
}

#[wasm_bindgen]
pub struct End2ClientSession {
    account: Account,
}

#[wasm_bindgen]
impl End2ClientSession {
    pub fn new() -> Self {
        Self {
            account: Account::new(),
        }
    }

    pub fn get_identity_keys(&self) -> Result<JsValue, JsError> {
        let keys = self.account.identity_keys();
        Ok(serde_wasm_bindgen::to_value(&keys)?)
    }

    pub fn generate_otks(&mut self, count: usize) -> Result<JsValue, JsError> {
        self.account.generate_one_time_keys(count);
        let otks = self.account.one_time_keys();

        let otks_clean: HashMap<String, _> = otks
            .iter()
            .map(|(k, v)| (k.to_base64(), v)) 
            .collect();

        let serializer = Serializer::json_compatible();
        Ok(otks_clean.serialize(&serializer)?)
    }

    pub fn get_registration_payload(&self, username: String) -> Result<JsValue, JsError> {
        let identity_keys = self.account.identity_keys();
        let ed25519 = identity_keys.ed25519;
        let curve25519 = identity_keys.curve25519;

        let message = [
            username.as_bytes(),
            curve25519.as_bytes(),
            ed25519.as_bytes(),
        ]
        .concat();

        let signature = self.account.sign(&message);

        let new_user = NewUser {
            username,
            ed25519: ed25519.to_base64(),
            curve25519: curve25519.to_base64(),
            signature: signature.to_base64(),
        };

        Ok(serde_wasm_bindgen::to_value(&new_user)?)
    }

    pub fn sign_challenge(&self, challenge: JsValue) -> Result<JsValue, JsError> {
        let input: ChallengeInput = serde_wasm_bindgen::from_value(challenge)?;
        let id = Uuid::try_parse(&input.id).unwrap();
        let user_id = Uuid::try_parse(&input.user_id).unwrap();
        let message = [id.into_bytes(), user_id.into_bytes()].concat();

        let signature_bytes = self.account.sign(&message);

        Ok(serde_wasm_bindgen::to_value(&signature_bytes.to_base64())?)
    }
}

#[wasm_bindgen]
pub fn add_rs(a: i32, b: i32) -> i32 {
    a + b
}
