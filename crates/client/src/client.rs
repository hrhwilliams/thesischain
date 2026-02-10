use defs::Identity;

pub struct Client;

impl Client {
    pub fn new(identity: Identity) -> Self {
        Self
    }

    pub async fn try_connect(self, urls: &[&'static str]) -> Self {
        Self
    }

    pub async fn run(self) {}
}
