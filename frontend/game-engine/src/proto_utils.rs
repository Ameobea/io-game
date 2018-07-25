use std::mem;

use protobuf::{parse_from_bytes, Message};
use uuid::Uuid;

use super::send_message;
use protos::client_messages::{ClientMessage, ClientMessage_oneof_payload as ClientMessageContent};
use protos::message_common::Uuid as ProtoUuid;
use protos::server_messages::ServerMessage;
pub use protos::server_messages::ServerMessage_oneof_payload as ServerMessageContent;
use util::{error, log, warn};

pub struct InnerServerMessage {
    pub id: Uuid,
    pub content: ServerMessageContent,
}

impl Into<Option<InnerServerMessage>> for ServerMessage {
    fn into(mut self: ServerMessage) -> Option<InnerServerMessage> {
        if cfg!(debug_assertions) {
            if let Some(ref fields) = self.get_unknown_fields().fields {
                let field_names = fields.iter().collect::<Vec<_>>();
                warn(format!(
                    "Unknown fields provided to message: {:?}",
                    field_names
                ));
            }
        }

        if !self.has_id() {
            warn("Issue while parsing server message: `id` was not provided!");
            return None;
        } else if self.payload.is_none() {
            warn("Issue while parsing server message: `payload` as not provided!");
            return None;
        }

        let inner_msg = InnerServerMessage {
            id: self.take_id().into(),
            content: self.payload.unwrap(),
        };

        Some(inner_msg)
    }
}

impl Into<Uuid> for ProtoUuid {
    fn into(self: ProtoUuid) -> Uuid {
        let data: u128 = unsafe { mem::transmute([self.get_data_1(), self.get_data_2()]) };
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

/// Creates a `ClientMessage` with the given payload, converts it to binary, encodes it into
/// binary, and sends it over the WebSocket to the backend.
pub fn send_user_message(payload: ClientMessageContent) {
    let mut client_msg = ClientMessage::new();
    client_msg.payload = Some(payload);

    let bytes = msg_to_bytes(client_msg);
    log(format!("Sending user message: {:?}", bytes));
    send_message(bytes);
}
