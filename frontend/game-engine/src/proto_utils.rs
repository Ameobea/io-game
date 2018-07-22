use std::mem;

use protobuf::parse_from_bytes;
use uuid::Uuid;

use protos::message_common::Uuid as ProtoUuid;
use protos::server_messages::ServerMessage;
pub use protos::server_messages::ServerMessage_oneof_payload as ServerMessageContent;
use util::error;

pub struct InnerServerMessage {
    pub id: Uuid,
    pub content: ServerMessageContent,
}

impl Into<Option<InnerServerMessage>> for ServerMessage {
    fn into(self: ServerMessage) -> Option<InnerServerMessage> {
        if self.id.is_none() || self.payload.is_none() {
            return None;
        }

        let inner_msg = InnerServerMessage {
            id: self.id.unwrap().into(),
            content: self.payload.unwrap(),
        };

        Some(inner_msg)
    }
}

impl Into<Uuid> for ProtoUuid {
    fn into(self: ProtoUuid) -> Uuid {
        let data: u128 = unsafe { mem::transmute([self.data_1, self.data_2]) };
        data.into()
    }
}

pub fn parse_server_message(bytes: &[u8]) -> Option<InnerServerMessage> {
    let msg: ServerMessage = match parse_from_bytes(bytes) {
        Ok(msg) => msg,
        Err(err) => {
            error(format!("Error parsing message from server: {:?}", err));
            return None;
        }
    };

    msg.into()
}
