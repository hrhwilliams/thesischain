use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper,
};
use uuid::Uuid;

use crate::schema::{channel, channel_participant, device, message, message_payload, one_time_key, user};
use crate::{
    AppError, Channel, ChannelInfo, ChannelParticipant, Device, Otk, OutboundChatMessage, User,
};

use super::AppState;

impl AppState {
    pub(crate) async fn get_channel_participants(
        &self,
        channel_id: Uuid,
    ) -> Result<Vec<User>, AppError> {
        let mut conn = self.get_conn()?;

        let users = tokio::task::spawn_blocking(move || {
            channel_participant::table
                .filter(channel_participant::channel_id.eq(channel_id))
                .inner_join(user::table.on(user::id.eq(channel_participant::user_id)))
                .select(User::as_select())
                .load(&mut conn)
        })
        .await??;

        Ok(users)
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_channel_info(
        &self,
        user: &User,
        channel_id: Uuid,
    ) -> Result<ChannelInfo, AppError> {
        let mut conn = self.get_conn()?;

        let participants = self.get_channel_participants(channel_id).await?;

        if !participants.contains(user) {
            return Err(AppError::Unauthorized);
        }

        let devices = device::table
            .inner_join(user::table.on(device::user_id.eq(user::id)))
            .filter(user::id.eq_any(participants.iter().map(|u| u.id)))
            .distinct()
            .select(Device::as_select())
            .load(&mut conn)?;

        Ok(ChannelInfo {
            channel_id,
            participants,
            devices,
        })
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_user_channels(&self, user: &User) -> Result<Vec<Channel>, AppError> {
        let mut conn = self.get_conn()?;

        let user_id = user.id;
        let channels = tokio::task::spawn_blocking(move || {
            channel_participant::table
                .filter(channel_participant::user_id.eq(user_id))
                .inner_join(channel::table.on(channel::id.eq(channel_participant::channel_id)))
                .select(Channel::as_select())
                .load(&mut conn)
        })
        .await??;

        Ok(channels)
    }

    pub async fn get_channel_history(
        &self,
        _user: &User,
        channel_id: Uuid,
        device_id: Uuid,
        after: Option<Uuid>,
    ) -> Result<Vec<OutboundChatMessage>, AppError> {
        let mut conn = self.get_conn()?;

        let mut query = message::table
            .inner_join(
                message_payload::table.on(message::id
                    .eq(message_payload::message_id)
                    .and(message_payload::recipient_device_id.eq(device_id))),
            )
            .filter(message::channel_id.eq(channel_id))
            .select((
                message::id,
                message::sender_device_id,
                message::channel_id,
                message::sender_id,
                message_payload::ciphertext,
                message::created,
                message_payload::is_pre_key,
            ))
            .order(message::id.asc())
            .into_boxed();

        if let Some(after) = after {
            query = query.filter(message::id.gt(after))
        }

        let history = query.load::<OutboundChatMessage>(&mut conn)?;
        Ok(history)
    }

    pub async fn create_channel_between(
        &self,
        sender: &User,
        recipient: &User,
    ) -> Result<ChannelInfo, AppError> {
        let mut conn = self.get_conn()?;

        if sender == recipient {
            return Err(AppError::UserError(
                "can't make chat with yourself".to_string(),
            ));
        }

        let channel = diesel::insert_into(channel::table)
            .default_values()
            .returning(Channel::as_returning())
            .get_result(&mut conn)?;

        let participant1 = ChannelParticipant {
            channel_id: channel.id,
            user_id: sender.id,
        };

        let participant2 = ChannelParticipant {
            channel_id: channel.id,
            user_id: recipient.id,
        };

        diesel::insert_into(channel_participant::table)
            .values(&[participant1, participant2])
            .execute(&mut conn)?;

        let channel_info = self.get_channel_info(sender, channel.id).await?;
        Ok(channel_info)
    }

    pub async fn get_user_otk(&self, user: &User, device_id: Uuid) -> Result<Otk, AppError> {
        let mut conn = self.get_conn()?;

        let otk = one_time_key::table
            .inner_join(device::table.on(one_time_key::device_id.eq(device::id)))
            .filter(
                one_time_key::device_id
                    .eq(device_id)
                    .and(device::user_id.eq(user.id)),
            )
            .select(Otk::as_select())
            .first(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        diesel::delete(one_time_key::table.find(otk.id))
            .execute(&mut conn)
            .map_err(|e| AppError::QueryFailed(e.to_string()))?;

        Ok(otk)
    }
}
