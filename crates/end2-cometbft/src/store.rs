use std::sync::Arc;

use rocksdb::{DB, Direction, IteratorMode};

// Key layout in a single column family.
// d/<user_hash_hex>/<device_id>  -> JSON(DeviceKeys)
// m/height                       -> u64 LE
// m/app_hash                     -> 32 bytes
const P_DEVICE: u8 = b'd';

pub const META_HEIGHT: &[u8] = b"m/height";
pub const META_APP_HASH: &[u8] = b"m/app_hash";

pub struct Store {
    pub db: Arc<DB>,
}

impl Store {
    pub fn open(path: &str) -> Result<Self, rocksdb::Error> {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(Self { db: Arc::new(db) })
    }

    pub fn last_height(&self) -> u64 {
        match self.db.get(META_HEIGHT).expect("rocksdb get height") {
            Some(b) if b.len() == 8 => {
                let mut a = [0u8; 8];
                a.copy_from_slice(&b);
                u64::from_le_bytes(a)
            }
            _ => 0,
        }
    }

    pub fn last_app_hash(&self) -> Vec<u8> {
        self.db
            .get(META_APP_HASH)
            .expect("rocksdb get app_hash")
            .unwrap_or_default()
    }

    pub fn device_key(user_hash_hex: &str, device_id: &str) -> Vec<u8> {
        let mut k = Vec::with_capacity(2 + user_hash_hex.len() + 1 + device_id.len());
        k.push(P_DEVICE);
        k.push(b'/');
        k.extend_from_slice(user_hash_hex.as_bytes());
        k.push(b'/');
        k.extend_from_slice(device_id.as_bytes());
        k
    }

    pub fn device_prefix(user_hash_hex: &str) -> Vec<u8> {
        let mut k = Vec::with_capacity(2 + user_hash_hex.len() + 1);
        k.push(P_DEVICE);
        k.push(b'/');
        k.extend_from_slice(user_hash_hex.as_bytes());
        k.push(b'/');
        k
    }

    pub fn all_devices_prefix() -> Vec<u8> {
        vec![P_DEVICE, b'/']
    }

    pub fn get_device(&self, user_hash_hex: &str, device_id: &str) -> Option<Vec<u8>> {
        self.db
            .get(Self::device_key(user_hash_hex, device_id))
            .expect("rocksdb get device")
    }

    pub fn iter_user_devices(&self, user_hash_hex: &str) -> Vec<(String, Vec<u8>)> {
        let prefix = Self::device_prefix(user_hash_hex);
        let mut out = Vec::new();
        let iter = self
            .db
            .iterator(IteratorMode::From(&prefix, Direction::Forward));
        for item in iter {
            let (k, v) = item.expect("rocksdb iterate user devices");
            if !k.starts_with(&prefix) {
                break;
            }
            let device_id = std::str::from_utf8(&k[prefix.len()..])
                .expect("non-utf8 device id in rocksdb")
                .to_owned();
            out.push((device_id, v.to_vec()));
        }
        out
    }

    // Every device entry in sorted rocksdb-key order. Used by app_hash computation.
    pub fn all_devices(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        let prefix = Self::all_devices_prefix();
        let mut out = Vec::new();
        let iter = self
            .db
            .iterator(IteratorMode::From(&prefix, Direction::Forward));
        for item in iter {
            let (k, v) = item.expect("rocksdb iterate all devices");
            if !k.starts_with(&prefix) {
                break;
            }
            out.push((k.to_vec(), v.to_vec()));
        }
        out
    }
}
