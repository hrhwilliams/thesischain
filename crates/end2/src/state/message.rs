use diesel::{RunQueryDsl, SelectableHelper};

use crate::schema::{message, message_payload};
use crate::{
    AppError, ChatMessage, InboundChatMessage, MessagePayload, NewChatMessage, NewMessagePayload,
    User,
};

use super::AppState;

impl AppState {
    pub async fn save_message(
        &self,
        user: &User,
        message: InboundChatMessage,
    ) -> Result<(ChatMessage, Vec<MessagePayload>), AppError> {
        let mut conn = self.get_conn()?;

        let users = self.get_channel_participants(message.channel_id).await?;

        if !users.contains(user) {
            return Err(AppError::Unauthorized);
        }

        let new_message = NewChatMessage::from_inbound(user, &message);
        let payloads = message
            .payloads
            .into_iter()
            .map(|m| m.into_new_message(message.message_id))
            .collect::<Result<Vec<NewMessagePayload>, _>>()?;

        let message = diesel::insert_into(message::table)
            .values(&new_message)
            .returning(ChatMessage::as_returning())
            .get_result(&mut conn)?;

        let payloads = diesel::insert_into(message_payload::table)
            .values(&payloads)
            .returning(MessagePayload::as_returning())
            .load(&mut conn)?;

        Ok((message, payloads))
    }
}
