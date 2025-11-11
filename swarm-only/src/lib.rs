use p256::{PublicKey, SecretKey, elliptic_curve::rand_core::OsRng};

pub struct Me {
    pub username: String,
    private_key: SecretKey,
}

impl Me {
    pub fn new(username: String) -> Self {
        Self {
            username,
            private_key: SecretKey::random(&mut OsRng),
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.private_key.public_key()
    }
}

pub struct User {
    pub username: String,
    pub public_key: PublicKey,
}