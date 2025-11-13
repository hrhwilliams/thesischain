use core::{identity::Identity, proto::IdentityTx};
use prost::{Message, bytes::Bytes};
use std::{collections::HashMap, sync::Arc};

use tendermint_abci::Application;
use tendermint_proto::{
    abci::ResponseCommit,
    v0_38::abci::{RequestCheckTx, RequestQuery, ResponseCheckTx, ResponseQuery},
};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AbciApp {
    users: Arc<Mutex<HashMap<String, Identity>>>,
}

impl AbciApp {
    pub fn new(users: Arc<Mutex<HashMap<String, Identity>>>) -> Self {
        Self { users }
    }
}

impl Application for AbciApp {
    fn check_tx(&self, request: RequestCheckTx) -> ResponseCheckTx {
        let mut response = ResponseCheckTx::default();

        let mut tx_bytes: &[u8] = request.tx.as_ref();
        match IdentityTx::decode(&mut tx_bytes) {
            Ok(proto_identity) => match Identity::try_from_proto(proto_identity) {
                Ok(identity) => {
                    if identity.verify() {
                        response.code = 0; // OK

                        let mut write = self.users.blocking_lock();
                        write.insert(identity.name.clone(), identity);
                    } else {
                        response.code = 1;
                        response.log = "identity signature is invalid".into();
                    }
                }
                Err(err) => {
                    response.code = 1;
                    response.log = format!("invalid identity data: {err}");
                }
            },
            Err(err) => {
                response.code = 1;
                response.log = format!("failed to decode identity tx: {err}");
            }
        }

        response
    }

    fn commit(&self) -> ResponseCommit {
        println!("commit: persisting new application state...");

        ResponseCommit { retain_height: 0 }
    }

    fn query(&self, request: RequestQuery) -> ResponseQuery {
        let mut response = ResponseQuery::default();

        if request.path != "/users" {
            response.code = 1;
            response.log = format!("unsupported query path: {}", request.path);
            return response;
        }

        let username = match std::str::from_utf8(&request.data) {
            Ok(value) => value,
            Err(_) => {
                response.code = 1;
                response.log = "query data must be valid UTF-8".into();
                return response;
            }
        };

        let users = self.users.blocking_lock();
        match users.get(username) {
            Some(identity) => {
                response.code = 0;
                response.key = Bytes::copy_from_slice(username.as_bytes());
                response.value =
                    Bytes::from(serde_json::to_vec(identity).expect("serialize identity"));
            }
            None => {
                response.code = 1;
                response.log = format!("user '{}' not found", username);
            }
        }

        response
    }
}
