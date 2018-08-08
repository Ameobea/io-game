use super::send_message;
use conf::CONF;
use game_state::get_state;
use protos::channel_messages::{
    ClientChannelMessage, Event, Event_oneof_payload as EventPayload, PhoenixEvent,
    ServerChannelMessage,
};
use protos::client_messages::{ClientMessage, ConnectMessage};
use protos::server_messages::ServerMessage;
use util::{error, warn};

use protobuf::{parse_from_bytes, Message};
use uuid::Uuid;

static mut CUR_REF: usize = 1;

/// The same thing as `cur_reference++`
fn inc_ref() -> usize {
    let old_ref = unsafe { CUR_REF };
    unsafe { CUR_REF += 1 };
    old_ref
}

pub fn send_channel_message<S: Into<String>>(
    topic: S,
    event: Event,
    payload: Option<ClientMessage>,
) {
    let mut msg = ClientChannelMessage::new();
    msg.set_topic(topic.into());
    msg.set_event(event);
    if let Some(payload) = payload {
        msg.set_payload(payload);
    }
    msg.set_field_ref(format!("{}", inc_ref()));

    match msg.write_to_bytes() {
        Ok(bytes) => send_message(bytes),
        Err(err) => error(format!(
            "Error while converting `ClientChannelMessage` to bytes: {:?}",
            err
        )),
    }
}

pub fn send_connect_message() {
    let mut connect_msg = ClientMessage::new();
    let mut content = ConnectMessage::new();
    content.set_username("Ameo".into());
    connect_msg.set_connect(content);
    let mut evt = Event::new();
    evt.set_custom_event("game".into());

    send_channel_message(CONF.network.game_channel_name, evt, Some(connect_msg))
}

pub fn join_game_channel() -> Uuid {
    let mut evt = Event::new();
    evt.set_phoenix_event(PhoenixEvent::Join);
    send_channel_message(CONF.network.game_channel_name, evt, None);

    // TODO: Get this from the reply (?)
    Uuid::nil()
}

fn warn_msg(msg_type: &str, topic: &str) {
    warn(format!(
        "Received `{}` message with topic: {}",
        msg_type, topic,
    ))
}

pub fn handle_server_msg(bytes: &[u8]) {
    let ServerChannelMessage {
        topic,
        event,
        field_ref: _,
        join_ref: _,
        payload,
        ..
    } = match parse_from_bytes(bytes) {
        Ok(msg) => msg,
        Err(err) => {
            error(format!(
                "Error parsing WebSocket message from the server: {:?}",
                err
            ));
            return;
        }
    };

    match event.into_option() {
        Some(evt) => {
            match evt.payload {
                Some(EventPayload::custom_event(_)) | Some(EventPayload::phoenix_event(PhoenixEvent::Reply)) => {
                    // warn_msg(&evt, &topic);
                    let server_msg: ServerMessage = match payload.into_option() {
                        Some(msg) => msg,
                        None => {
                            error("Received `ServerSocketMessage` without a `ServerMessage` payload");
                            return;
                        }
                    };

                    get_state().apply_msg(server_msg);
                },
                Some(EventPayload::phoenix_event(evt)) => match evt {
                    PhoenixEvent::Close => warn_msg("close", &topic),
                    PhoenixEvent::Join => warn_msg("join", &topic),
                    PhoenixEvent::Reply => warn_msg("reply", &topic),
                    PhoenixEvent::Leave => warn_msg("leave", &topic),
                    PhoenixEvent::Error => error(format!(
                        "Phoenix error; topic: {}.  Can't print payload because of insane reflection codegen bloat.",
                        topic
                    )),
                },
                None => error("Received channel event with no inner payload!"),
            }
        }
        None => warn("Received channel message with no event payload!"),
    }
}
