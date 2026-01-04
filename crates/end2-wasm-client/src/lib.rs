use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::Serializer;
use std::str::FromStr;
use uuid::Uuid;
use vodozemac::{
    Curve25519PublicKey,
    olm::{Account, AccountPickle, OlmMessage, Session, SessionConfig, SessionPickle},
};
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

#[derive(Deserialize, Serialize)]
pub struct State {
    account_pickle: String,
    sessions: HashMap<String, String>,
}

impl State {
    fn from_client(client: &End2ClientSession) -> Self {
        let account_pickle = client.account.pickle().encrypt(&[0u8; 32]);

        Self {
            account_pickle,
            sessions: client.sessions.clone(),
        }
    }
}

#[wasm_bindgen]
pub struct End2ClientSession {
    account: Account,
    sessions: HashMap<String, String>,
}

impl TryFrom<State> for End2ClientSession {
    type Error = JsError;

    fn try_from(state: State) -> Result<Self, JsError> {
        let pickle = AccountPickle::from_encrypted(&state.account_pickle, &[0u8; 32])
            .map_err(|e| JsError::new(&e.to_string()))?;
        let account = Account::from_pickle(pickle);

        Ok(Self {
            account,
            sessions: state.sessions,
        })
    }
}

#[wasm_bindgen]
impl End2ClientSession {
    pub fn new() -> Self {
        Self {
            account: Account::new(),
            sessions: HashMap::default(),
        }
    }

    pub fn export_state(&self) -> Result<JsValue, JsError> {
        Ok(serde_wasm_bindgen::to_value(&State::from_client(self))?)
    }

    pub fn from_state(state: &str) -> Result<End2ClientSession, JsError> {
        let state =
            serde_json::from_str::<State>(state).map_err(|e| JsError::new(&e.to_string()))?;
        Self::try_from(state)
    }

    pub fn get_identity_keys(&self) -> Result<JsValue, JsError> {
        let keys = self.account.identity_keys();
        Ok(serde_wasm_bindgen::to_value(&keys)?)
    }

    pub fn generate_otks(&mut self, count: usize) -> Result<JsValue, JsError> {
        self.account.generate_one_time_keys(count);
        let otks = self.account.one_time_keys();
        self.account.mark_keys_as_published();

        let otks: Vec<String> = otks.values().map(|otk| otk.to_base64()).collect();
        // let serializer = Serializer::json_compatible();
        // Ok(otks.serialize(&serializer)?)
        Ok(serde_wasm_bindgen::to_value(&otks)?)
    }

    pub fn create_outbound_session(
        &mut self,
        channel_id: &str,
        identity_key: &str,
        one_time_key: &str,
    ) -> Result<(), JsError> {
        let channel_id = Uuid::from_str(channel_id).map_err(|e| JsError::new(&e.to_string()))?;
        let identity_key = Curve25519PublicKey::from_base64(identity_key)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let one_time_key = Curve25519PublicKey::from_base64(one_time_key)
            .map_err(|e| JsError::new(&e.to_string()))?;
        let session = self.account.create_outbound_session(
            SessionConfig::version_2(),
            identity_key,
            one_time_key,
        );

        let pickle = session.pickle().encrypt(&[0u8; 32]);
        self.sessions.insert(channel_id.to_string(), pickle);
        Ok(())
    }

    pub fn encrypt(&mut self, channel_id: &str, message: &str) -> Result<JsValue, JsError> {
        let channel_id = Uuid::from_str(channel_id).map_err(|e| JsError::new(&e.to_string()))?;
        let session = self
            .sessions
            .get(&channel_id.to_string())
            .ok_or(JsError::new("Missing session"))?;
        let pickle = SessionPickle::from_encrypted(session, &[0u8; 32])
            .map_err(|e| JsError::new(&e.to_string()))?;
        let mut session = Session::from_pickle(pickle);

        let olm_message = session.encrypt(message);

        let pickle = session.pickle().encrypt(&[0u8; 32]);
        self.sessions.insert(channel_id.to_string(), pickle);

        Ok(serde_wasm_bindgen::to_value(&olm_message)?)
    }

    pub fn decrypt(
        &mut self,
        channel_id: &str,
        identity_key: &str,
        message: &str,
    ) -> Result<String, JsError> {
        let channel_id = Uuid::from_str(channel_id).map_err(|e| JsError::new(&e.to_string()))?;
        let identity_key = Curve25519PublicKey::from_base64(identity_key)
            .map_err(|e| JsError::new(&e.to_string()))?;

        let message = serde_json::from_str::<OlmMessage>(message)
            .map_err(|e| JsError::new(&e.to_string()))?;

        let plaintext = if let OlmMessage::PreKey(pkm) = message {
            let result = self
                .account
                .create_inbound_session(identity_key, &pkm)
                .map_err(|e| JsError::new(&e.to_string()))?;

            self.sessions.insert(
                channel_id.to_string(),
                result.session.pickle().encrypt(&[0u8; 32]),
            );

            String::from_utf8(result.plaintext).map_err(|e| JsError::new(&e.to_string()))?
        } else {
            let session = self
                .sessions
                .get(&channel_id.to_string())
                .ok_or(JsError::new("Missing session"))?;
            let session_pickle = SessionPickle::from_encrypted(session, &[0u8; 32])
                .map_err(|e| JsError::new(&e.to_string()))?;
            let mut session = Session::from_pickle(session_pickle);

            let plaintext = session
                .decrypt(&message)
                .map_err(|e| JsError::new(&e.to_string()))?;

            self.sessions
                .insert(channel_id.to_string(), session.pickle().encrypt(&[0u8; 32]));
            String::from_utf8(plaintext).map_err(|e| JsError::new(&e.to_string()))?
        };

        Ok(plaintext)
    }

    pub fn register(&self, username: String) -> Result<JsValue, JsError> {
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
