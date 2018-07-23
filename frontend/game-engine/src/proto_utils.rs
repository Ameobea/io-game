use std::mem;

use protobuf::{parse_from_bytes, Message};
use uuid::Uuid;

use protos::message_common::Uuid as ProtoUuid;
use protos::server_messages::ServerMessage;
pub use protos::server_messages::ServerMessage_oneof_payload as ServerMessageContent;
use util::{error, log};

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

impl Into<ProtoUuid> for Uuid {
    fn into(self: Uuid) -> ProtoUuid {
        let (data_1, data_2): (u64, u64) = unsafe { mem::transmute(self) };
        let mut id = ProtoUuid::new();
        id.set_data_1(data_1);
        id.set_data_2(data_2);

        id
    }
}

pub fn parse_server_message(bytes: &[u8]) -> Option<InnerServerMessage> {
    log("Parsing server message...");
    let msg: ServerMessage = match parse_from_bytes(bytes) {
        Ok(msg) => msg,
        Err(err) => {
            error("ERROR");
            error(format!("Error parsing message from server: {:?}", err));
            return None;
        }
    };

    msg.into()
}

pub fn msg_to_bytes<M: Message>(msg: M) -> Vec<u8> {
    msg.write_to_bytes().unwrap_or_else(|err| {
        error(format!(
            "Error while writing created `ServerMessage` to bytes: {:?}",
            err
        ));
        panic!()
    })
}
