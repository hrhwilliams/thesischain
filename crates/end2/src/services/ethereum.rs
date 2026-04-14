use std::sync::Arc;

use alloy::{
    network::Ethereum,
    primitives::{FixedBytes, U256, keccak256},
    providers::Provider,
};
use async_trait::async_trait;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper, r2d2::ConnectionManager,
};
use r2d2::Pool;
use vodozemac::{Curve25519PublicKey, Ed25519PublicKey};

use crate::{
    AppError, Device, DeviceId, DeviceKeyService, InboundDevice, NewDevice, User,
    schema::{device, user as user_table},
};

alloy::sol! {
    #[sol(rpc)]
    contract KeyDirectory {
            struct Device {
                uint128 device_id;
                uint128 flags;
                bytes32 x25519;
                bytes32 ed25519;
            }

        function add_first_device(bytes32 userHash, uint128 deviceId, bytes32 x25519, bytes32 ed25519) public;
        function add_device(bytes32 userHash, uint128 deviceId, bytes32 x25519, bytes32 ed25519, uint256 nonce) public;
        function get_device(bytes32 user_hash, uint128 device_id) public view returns (Device memory);
        function get_all_devices(bytes32 user_hash) public view returns (Device[] memory);
        function get_nonce(bytes32 userHash) public view returns (uint256);
    }
}

#[derive(Clone)]
pub struct EthDeviceKeyService<P> {
    provider: Arc<P>,
    contract_address: alloy::primitives::Address,
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl<P> EthDeviceKeyService<P>
where
    P: Provider<Ethereum> + Clone + 'static,
{
    #[must_use]
    pub fn new(
        provider: Arc<P>,
        contract_address: alloy::primitives::Address,
        pool: Pool<ConnectionManager<PgConnection>>,
    ) -> Self {
        Self {
            provider,
            contract_address,
            pool,
        }
    }

    fn get_conn(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, AppError> {
        self.pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))
    }

    fn user_hash(user: &User) -> FixedBytes<32> {
        keccak256(format!("{}", user.id).as_bytes())
    }
}

#[async_trait]
impl<P> DeviceKeyService for EthDeviceKeyService<P>
where
    P: Provider<Ethereum> + Clone + Send + Sync + 'static,
{
    #[tracing::instrument(skip(self))]
    async fn new_device_for(&self, user: &User) -> Result<Device, AppError> {
        let mut conn = self.get_conn()?;
        let new_device = NewDevice {
            user_id: user.id,
            x25519: None,
            ed25519: None,
        };

        let device = diesel::insert_into(device::table)
            .values(&new_device)
            .returning(Device::as_returning())
            .get_result(&mut conn)?;

        Ok(device)
    }

    #[tracing::instrument(skip(self))]
    async fn get_device(&self, user: &User, device_id: DeviceId) -> Result<Device, AppError> {
        let contract = KeyDirectory::new(self.contract_address, self.provider.clone());
        let user_hash = Self::user_hash(user);

        let device = contract
            .get_device(user_hash, device_id.into_inner().as_u128())
            .call()
            .await
            .map(|d| Device {
                id: DeviceId::from(uuid::Uuid::from_u128(d.device_id)),
                user_id: user.id,
                x25519: Some(d.x25519.to_vec()),
                ed25519: Some(d.ed25519.to_vec()),
            })
            .map_err(|e| AppError::ValueError(e.to_string()))?;

        Ok(device)
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_devices(&self, user: &User) -> Result<Vec<Device>, AppError> {
        let contract = KeyDirectory::new(self.contract_address, self.provider.clone());
        let user_hash = Self::user_hash(user);

        let devices = contract
            .get_all_devices(user_hash)
            .call()
            .await
            .map_err(|e| AppError::ValueError(e.to_string()))?
            .into_iter()
            .map(|d| Device {
                id: DeviceId::from(uuid::Uuid::from_u128(d.device_id)),
                user_id: user.id,
                x25519: Some(d.x25519.to_vec()),
                ed25519: Some(d.ed25519.to_vec()),
            })
            .collect();

        Ok(devices)
    }

    /// Upload device keys (x25519, ed25519) to smart contract
    #[tracing::instrument(skip(self))]
    async fn set_device_keys(
        &self,
        user: &User,
        device_id: DeviceId,
        device_keys: InboundDevice,
    ) -> Result<Device, AppError> {
        let x25519 = Curve25519PublicKey::from_base64(&device_keys.x25519)
            .map_err(|e| AppError::InvalidKey(e.to_string()))?;
        let ed25519 = Ed25519PublicKey::from_base64(&device_keys.ed25519)
            .map_err(|e| AppError::InvalidKey(e.to_string()))?;

        let x25519_bytes: FixedBytes<32> = FixedBytes::from_slice(x25519.as_bytes());
        let ed25519_bytes: FixedBytes<32> = FixedBytes::from_slice(ed25519.as_bytes());

        let user_hash = Self::user_hash(user);
        let device_id_u128 = device_id.into_inner().as_u128();

        let contract = KeyDirectory::new(self.contract_address, self.provider.clone());

        let nonce = contract
            .get_nonce(user_hash)
            .call()
            .await
            .map_err(|e| AppError::ValueError(e.to_string()))?;

        let send = if nonce == U256::ZERO {
            contract
                .add_first_device(user_hash, device_id_u128, x25519_bytes, ed25519_bytes)
                .send()
                .await
                .map_err(|e| AppError::ValueError(e.to_string()))?
        } else {
            contract
                .add_device(
                    user_hash,
                    device_id_u128,
                    x25519_bytes,
                    ed25519_bytes,
                    nonce,
                )
                .send()
                .await
                .map_err(|e| AppError::ValueError(e.to_string()))?
        };

        let receipt = send
            .get_receipt()
            .await
            .map_err(|e| AppError::ValueError(e.to_string()))?;

        if !receipt.status() {
            tracing::error!(%user.username, "key directory contract transaction reverted");
            return Err(AppError::ValueError("contract transaction reverted".into()));
        }

        let x25519_vec = x25519.as_bytes().to_vec();
        let ed25519_vec = ed25519.as_bytes().to_vec();

        let mut conn = self.get_conn()?;
        let x25519_db = x25519_vec.clone();
        let ed25519_db = ed25519_vec.clone();
        let user = user.clone();
        let device = tokio::task::spawn_blocking(move || {
            diesel::update(device::table)
                .filter(device::id.eq(device_id).and(device::user_id.eq(user.id)))
                .set((device::x25519.eq(x25519_db), device::ed25519.eq(ed25519_db)))
                .returning(Device::as_returning())
                .get_result(&mut conn)
        })
        .await??;

        Ok(device)
    }

    async fn get_valid_users(&self) -> Result<usize, AppError> {
        let mut have_devices = 0;

        let mut conn = self.get_conn()?;
        let users = tokio::task::spawn_blocking(move || {
            user_table::table.select(User::as_select()).load(&mut conn)
        })
        .await??;

        for user in &users {
            if self
                .get_all_devices(user)
                .await
                .is_ok_and(|d| !d.is_empty())
            {
                have_devices += 1;
            }
        }

        Ok(have_devices)
    }
}
