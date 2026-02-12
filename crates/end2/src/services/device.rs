use async_trait::async_trait;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper, r2d2::ConnectionManager,
};
use r2d2::Pool;
use uuid::Uuid;

use crate::schema::device;
use crate::{AppError, Device, InboundDevice, NewDevice, User};

#[async_trait]
pub trait DeviceKeyService: Send + Sync {
    async fn new_device_for(&self, user: &User) -> Result<Device, AppError>;
    async fn get_device(&self, user: &User, device_id: Uuid) -> Result<Device, AppError>;
    async fn get_all_devices(&self, user: &User) -> Result<Vec<Device>, AppError>;
    async fn set_device_keys(
        &self,
        user: &User,
        device_id: Uuid,
        keys: InboundDevice,
    ) -> Result<Device, AppError>;
}

pub struct DbDeviceKeyService {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl DbDeviceKeyService {
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
impl DeviceKeyService for DbDeviceKeyService {
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
}
