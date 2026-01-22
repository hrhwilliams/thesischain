use serde::{Deserialize, Serialize};
use vodozemac::{
    Curve25519PublicKey,
    olm::{Account, Session},
};

#[derive(Serialize)]
pub struct IdentityKeys {
    x25519: String,
    ed25519: String,
    signature: String,
}

#[derive(Serialize)]
pub struct InboundOtks {
    otks: Vec<String>,
    signature: String,
}

#[derive(Deserialize)]
pub struct Otk {
    pub id: String,
    pub device_id: String,
    pub otk: String,
}

#[derive(Serialize)]
pub struct Otks2 {
    created: Vec<String>,
    removed: Vec<String>,
    created_signature: String,
    removed_signature: String,
}

#[derive(Debug, Deserialize)]
pub struct DeviceInfo {
    pub device_id: String,
    pub user_id: String,
    pub x25519: Option<String>,
    pub ed25519: Option<String>,
}

pub struct Device {
    account: Account,
    device_id: String,
}

impl Device {
    pub fn new(device_id: &str) -> Self {
        Self {
            account: Account::new(),
            device_id: device_id.to_string(),
        }
    }

    pub fn id(&self) -> &str {
        &self.device_id
    }

    pub fn get_identity_keys(&self) -> IdentityKeys {
        let keys = self.account.identity_keys();
        let message = [keys.curve25519.as_bytes() as &[u8], keys.ed25519.as_bytes()].concat();

        let signature = self.account.sign(&message);

        IdentityKeys {
            x25519: keys.curve25519.to_base64(),
            ed25519: keys.ed25519.to_base64(),
            signature: signature.to_base64(),
        }
    }

    pub fn get_otks(&mut self, count: usize) -> InboundOtks {
        let keys = if count > self.account.max_number_of_one_time_keys() {
            self.account
                .generate_one_time_keys(self.account.max_number_of_one_time_keys())
        } else {
            self.account.generate_one_time_keys(count)
        };

        self.account.mark_keys_as_published();

        let message = keys
            .created
            .iter()
            .map(|k| k.as_bytes() as &[u8])
            .collect::<Vec<&[u8]>>()
            .concat();
        let signature = self.account.sign(&message);

        // send removed keys as well so server can remove them

        InboundOtks {
            otks: keys.created.iter().map(|k| k.to_base64()).collect(),
            signature: signature.to_base64(),
        }
    }
}
