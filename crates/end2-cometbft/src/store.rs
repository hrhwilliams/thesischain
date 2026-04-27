use std::sync::Arc;

use jmt::storage::{Node, NodeKey, TreeReader, TreeUpdateBatch};
use rocksdb::{
    BlockBasedOptions, ColumnFamily, ColumnFamilyDescriptor, DB, Direction, IteratorMode, Options,
    WriteBatchWithTransaction,
};

// Key layout in a single column family.
// d/<user_hash_hex>/<device_id>  -> JSON(DeviceKeys)
// m/height                       -> u64 LE
// m/app_hash                     -> 32 bytes
pub const CF_DEVICE: &str = "device";
pub const CF_JMT: &str = "jmt";

pub const META_HEIGHT: &[u8] = b"m/height";
pub const META_APP_HASH: &[u8] = b"m/app_hash";

pub struct Store {
    pub db: Arc<DB>,
}

impl Store {
    pub fn open(path: &str) -> Result<Self, rocksdb::Error> {
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);

        let default_opts = Options::default();

        let mut device_opts = Options::default();
        device_opts.optimize_for_point_lookup(256);

        let mut jmt_opts = Options::default();
        jmt_opts.set_max_background_jobs(4);
        jmt_opts.optimize_level_style_compaction(512 * 1024 * 1024);

        let mut jmt_block_opts = BlockBasedOptions::default();
        jmt_block_opts.set_block_cache(&rocksdb::Cache::new_lru_cache(256 * 1024 * 1024));
        jmt_block_opts.set_bloom_filter(10f64, false);
        jmt_opts.set_block_based_table_factory(&jmt_block_opts);

        let cfs = vec![
            ColumnFamilyDescriptor::new("default", default_opts),
            ColumnFamilyDescriptor::new(CF_DEVICE, device_opts),
            ColumnFamilyDescriptor::new(CF_JMT, jmt_opts),
        ];

        let db = DB::open_cf_descriptors(&db_opts, path, cfs)?;

        Ok(Self { db: Arc::new(db) })
    }

    pub fn cf_device(&self) -> &ColumnFamily {
        self.db.cf_handle(CF_DEVICE).expect("device cf missing")
    }

    pub fn cf_jmt(&self) -> &ColumnFamily {
        self.db.cf_handle(CF_JMT).expect("jmt cf missing")
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
        // k.push(P_DEVICE);
        // k.push(b'/');
        k.extend_from_slice(user_hash_hex.as_bytes());
        k.push(b'/');
        k.extend_from_slice(device_id.as_bytes());
        k
    }

    pub fn device_prefix(user_hash_hex: &str) -> Vec<u8> {
        let mut k = Vec::with_capacity(2 + user_hash_hex.len() + 1);
        // k.push(P_DEVICE);
        // k.push(b'/');
        k.extend_from_slice(user_hash_hex.as_bytes());
        k.push(b'/');
        k
    }

    pub fn get_device(&self, user_hash_hex: &str, device_id: &str) -> Option<Vec<u8>> {
        self.db
            .get_cf(
                &self.cf_device(),
                Self::device_key(user_hash_hex, device_id),
            )
            .expect("rocksdb get device")
    }

    pub fn iter_user_devices(&self, user_hash_hex: &str) -> Vec<(String, Vec<u8>)> {
        let prefix = Self::device_prefix(user_hash_hex);
        let mut out = Vec::new();
        let iter = self.db.iterator_cf(
            self.cf_device(),
            IteratorMode::From(&prefix, Direction::Forward),
        );
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

    pub fn jmt_node_key(node_key: &NodeKey) -> Vec<u8> {
        let key_bytes = borsh::to_vec(node_key).expect("failed to serialize jmt NodeKey");
        key_bytes
    }

    pub fn write_tree_update(
        &self,
        batch: &mut WriteBatchWithTransaction<false>,
        tree_update: TreeUpdateBatch,
    ) {
        for (node_key, node) in tree_update.node_batch.nodes() {
            let encoded_node = borsh::to_vec(node).expect("jmt encode node");
            batch.put(Self::jmt_node_key(node_key), encoded_node);
        }

        // TODO prune stale nodes
        // for stale_node in tree_update.stale_node_index_batch {

        // }
    }
}

impl TreeReader for Store {
    fn get_node_option(&self, node_key: &NodeKey) -> anyhow::Result<Option<Node>> {
        if let Some(bytes) = self
            .db
            .get_cf(&self.cf_jmt(), Self::jmt_node_key(node_key))?
        {
            let node = borsh::from_slice(&bytes)?;
            Ok(Some(node))
        } else {
            Ok(None)
        }
    }

    #[allow(unused_variables)]
    fn get_value_option(
        &self,
        max_version: jmt::Version,
        key_hash: jmt::KeyHash,
    ) -> anyhow::Result<Option<jmt::OwnedValue>> {
        unimplemented!("get_value_option unimplemented")
    }

    fn get_rightmost_leaf(&self) -> anyhow::Result<Option<(NodeKey, jmt::storage::LeafNode)>> {
        unimplemented!("get_rightmost_leaf unimplemented")
    }
}
