use std::mem;

use nalgebra::{Isometry2, Vector2};
use native_physics::physics::Movement;
use nphysics2d::algebra::Velocity2;
use protobuf::Message;
use uuid::Uuid;

use conf::CONF;
use phoenix_proto::send_channel_message;
use protos::channel_messages::Event;
use protos::client_messages::{ClientMessage, ClientMessage_oneof_payload as ClientMessageContent};
use protos::message_common::{MovementDirection, Uuid as ProtoUuid};
pub use protos::server_messages::ServerMessage_Payload_oneof_payload as ServerMessageContent;
use protos::server_messages::{
    MovementUpdate, ServerMessage, ServerMessage_Payload as ServerMessagePayload,
};
use util::warn;

pub struct InnerServerMessage {
    pub id: Uuid,
    pub content: ServerMessageContent,
}

impl Into<Option<InnerServerMessage>> for ServerMessagePayload {
    fn into(mut self: ServerMessagePayload) -> Option<InnerServerMessage> {
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

impl<'a> Into<(Isometry2<f32>, Velocity2<f32>)> for &'a MovementUpdate {
    fn into(self) -> (Isometry2<f32>, Velocity2<f32>) {
        let pos = Isometry2::new(Vector2::new(self.pos_x, self.pos_y), self.rotation);
        let velocity = Velocity2::new(
            Vector2::new(self.velocity_x, self.velocity_y),
            self.angular_velocity,
        );

        (pos, velocity)
    }
}

impl Into<Movement> for MovementDirection {
    fn into(self) -> Movement {
        match self {
            MovementDirection::DOWN => Movement::Down,
            MovementDirection::DOWN_LEFT => Movement::DownLeft,
            MovementDirection::DOWN_RIGHT => Movement::DownRight,
            MovementDirection::LEFT => Movement::Left,
            MovementDirection::RIGHT => Movement::Right,
            MovementDirection::STOP => Movement::Stop,
            MovementDirection::UP => Movement::Up,
            MovementDirection::UP_LEFT => Movement::UpLeft,
            MovementDirection::UP_RIGHT => Movement::UpRight,
        }
    }
}

pub fn parse_server_msg_payload(msg: ServerMessage) -> Vec<InnerServerMessage> {
    let mut inner_messages = Vec::with_capacity(msg.payload.len());
    for msg in msg.payload.into_iter() {
        if let Some(inner_msg) = msg.into() {
            inner_messages.push(inner_msg);
        }
    }
    inner_messages
}

pub fn msg_to_bytes<M: Message>(msg: M) -> Vec<u8> {
    msg.write_to_bytes().unwrap_or_else(|err| {
        panic!(format!(
            "Error while writing created `ServerMessage` to bytes: {:?}",
            err
        ))
    })
}

/// Creates a `ClientMessage` with the given payload, converts it to binary, encodes it into
/// binary, and sends it over the WebSocket to the backend.
pub fn send_user_message(payload: ClientMessageContent) {
    let mut client_msg = ClientMessage::new();
    client_msg.payload = Some(payload);

    let mut event = Event::new();
    event.set_custom_event(CONF.network.custom_event_name.into());
    send_channel_message(event, Some(client_msg));
}
