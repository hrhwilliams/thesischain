use async_trait::async_trait;
use base64::Engine;
use base64::prelude::BASE64_STANDARD_NO_PAD;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper, r2d2::ConnectionManager,
};
use ed25519_dalek::Signature;
use r2d2::Pool;
use uuid::Uuid;
use vodozemac::Curve25519PublicKey;

use crate::schema::{device, one_time_key};
use crate::{AppError, Device, InboundDevice, InboundOtks, NewDevice, NewOtk, Otk, User};

#[async_trait]
pub trait KeyExchangeService: Send + Sync {
    async fn new_device_for(&self, user: &User) -> Result<Device, AppError>;
    async fn get_device(&self, user: &User, device_id: Uuid) -> Result<Device, AppError>;
    async fn get_all_devices(&self, user: &User) -> Result<Vec<Device>, AppError>;
    async fn set_device_keys(
        &self,
        user: &User,
        device_id: Uuid,
        keys: InboundDevice,
    ) -> Result<Device, AppError>;
    async fn get_otks(&self, device_id: Uuid) -> Result<Vec<Otk>, AppError>;
    async fn upload_otks(
        &self,
        user: &User,
        device_id: Uuid,
        otks: InboundOtks,
    ) -> Result<(), AppError>;
    async fn get_user_otk(&self, user: &User, device_id: Uuid) -> Result<Otk, AppError>;
}

pub struct DbKeyExchangeService {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl DbKeyExchangeService {
    #[must_use]
    pub const fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        Self { pool }
    }

    fn get_conn(
        &self,
    ) -> Result<r2d2::PooledConnection<ConnectionManager<PgConnection>>, AppError> {
        self.pool
            .get()
            .map_err(|e| AppError::PoolError(e.to_string()))
    }
}

#[async_trait]
impl KeyExchangeService for DbKeyExchangeService {
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
    async fn get_device(&self, user: &User, device_id: Uuid) -> Result<Device, AppError> {
        let mut conn = self.get_conn()?;
        let user_id = user.id;

        tracing::debug!("querying for device");

        let device = tokio::task::spawn_blocking(move || {
            device::table
                .filter(device::id.eq(device_id).and(device::user_id.eq(user_id)))
                .select(Device::as_select())
                .first(&mut conn)
        })
        .await??;

        Ok(device)
    }

    #[tracing::instrument(skip(self))]
    async fn get_all_devices(&self, user: &User) -> Result<Vec<Device>, AppError> {
        let mut conn = self.get_conn()?;

        device::table
            .filter(device::user_id.eq(user.id))
            .select(Device::as_select())
            .load(&mut conn)
            .map_err(AppError::from)
    }

    async fn set_device_keys(
        &self,
        user: &User,
        device_id: Uuid,
        device_keys: InboundDevice,
    ) -> Result<Device, AppError> {
        let mut conn = self.get_conn()?;
        let user_id = user.id;
        let new_device = NewDevice::from_network(user_id, &device_keys)?;

        let device = tokio::task::spawn_blocking(move || {
            diesel::update(device::table)
                .filter(device::id.eq(device_id).and(device::user_id.eq(user_id)))
                .set((
                    device::x25519.eq(new_device.x25519),
                    device::ed25519.eq(new_device.ed25519),
                ))
                .get_result(&mut conn)
        })
        .await??;

        Ok(device)
    }

    async fn get_otks(&self, device_id: Uuid) -> Result<Vec<Otk>, AppError> {
        let mut conn = self.get_conn()?;

        let otks = tokio::task::spawn_blocking(move || {
            one_time_key::table
                .filter(one_time_key::device_id.eq(device_id))
                .select(Otk::as_select())
                .load(&mut conn)
        })
        .await??;

        Ok(otks)
    }

    #[tracing::instrument(skip(self, otks))]
    async fn upload_otks(
        &self,
        user: &User,
        device_id: Uuid,
        otks: InboundOtks,
    ) -> Result<(), AppError> {
        let mut conn = self.get_conn()?;

        let created_signature = BASE64_STANDARD_NO_PAD.decode(&otks.created_signature)?;
        let created_signature = Signature::from_bytes(
            created_signature
                .as_slice()
                .try_into()
                .map_err(|_| AppError::InvalidSignature)?,
        );

        let device = device::table
            .filter(device::id.eq(device_id).and(device::user_id.eq(user.id)))
            .select(Device::as_select())
            .first(&mut conn)?;

        let created_otks: Vec<Curve25519PublicKey> = otks
            .created
            .iter()
            .map(|k| {
                Curve25519PublicKey::from_base64(k).map_err(|e| AppError::InvalidKey(e.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let message = created_otks
            .iter()
            .map(|k| k.as_bytes() as &[u8])
            .collect::<Vec<&[u8]>>()
            .concat();

        let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(
            device
                .ed25519
                .ok_or(AppError::InvalidSignature)?
                .as_slice()
                .try_into()
                .map_err(|_| AppError::InvalidKeySize)?,
        )
        .map_err(|e| AppError::InvalidKey(e.to_string()))?;

        verifying_key
            .verify_strict(&message, &created_signature)
            .map_err(|e| AppError::ChallengeFailed(e.to_string()))?;

        let new_otks = created_otks
            .into_iter()
            .map(|k| NewOtk {
                device_id,
                otk: k.to_bytes(),
            })
            .collect::<Vec<NewOtk>>();

        diesel::insert_into(one_time_key::table)
            .values(&new_otks)
            .execute(&mut conn)?;

        if let Some(removed_signature) = otks.removed_signature {
            tracing::info!("removing {} keys", otks.removed.len());

            let removed_signature = BASE64_STANDARD_NO_PAD.decode(&removed_signature)?;
            let removed_signature = Signature::from_bytes(
                removed_signature
                    .as_slice()
                    .try_into()
                    .map_err(|_| AppError::InvalidSignature)?,
            );

            let removed_otks: Vec<Curve25519PublicKey> = otks
                .removed
                .iter()
                .map(|k| {
                    Curve25519PublicKey::from_base64(k)
                        .map_err(|e| AppError::InvalidKey(e.to_string()))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let removed_otks = removed_otks
                .iter()
                .map(|k| k.as_bytes() as &[u8])
                .collect::<Vec<&[u8]>>();

            verifying_key
                .verify_strict(&removed_otks.concat(), &removed_signature)
                .map_err(|e| AppError::ChallengeFailed(e.to_string()))?;

            diesel::delete(one_time_key::table)
                .filter(one_time_key::otk.eq_any(removed_otks))
                .execute(&mut conn)?;
        }

        Ok(())
    }

    async fn get_user_otk(&self, user: &User, device_id: Uuid) -> Result<Otk, AppError> {
        let mut conn = self.get_conn()?;
        let user_id = user.id;

        let otk = tokio::task::spawn_blocking(move || {
            let otk = one_time_key::table
                .inner_join(device::table.on(one_time_key::device_id.eq(device::id)))
                .filter(
                    one_time_key::device_id
                        .eq(device_id)
                        .and(device::user_id.eq(user_id)),
                )
                .select(Otk::as_select())
                .first(&mut conn)
                .map_err(|e| AppError::QueryFailed(e.to_string()))?;

            diesel::delete(one_time_key::table.find(otk.id))
                .execute(&mut conn)
                .map_err(|e| AppError::QueryFailed(e.to_string()))?;

            Ok::<_, AppError>(otk)
        })
        .await??;

        Ok(otk)
    }
}
