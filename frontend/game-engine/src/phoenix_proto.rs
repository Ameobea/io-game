use super::send_message;
use protobuf::{parse_from_bytes, Message};
use protos::channel_messages::{
    ChannelMessage, Event, Event_oneof_payload as EventPayload, PhoenixEvent,
};

use conf::CONF;
use util::{error, warn};

static mut CUR_REF: usize = 1;

/// The same thing as `cur_reference++`
fn inc_ref() -> usize {
    let old_ref = unsafe { CUR_REF };
    unsafe { CUR_REF += 1 };
    old_ref
}

pub fn send_channel_message<S: Into<String>>(topic: S, event: Event, payload: Vec<u8>) {
    let mut msg = ChannelMessage::new();
    msg.set_topic(topic.into());
    msg.set_event(event);
    msg.set_payload(payload);
    msg.set_field_ref(format!("{}", inc_ref()));

    match msg.write_to_bytes() {
        Ok(bytes) => send_message(bytes),
        Err(err) => error(format!(
            "Error while converting `ChannelMessage` to bytes: {:?}",
            err
        )),
    }
}

pub fn join_game_channel() {
    let mut evt = Event::new();
    evt.set_phoenix_event(PhoenixEvent::Join);
    send_channel_message(
        format!("rooms:{}", CONF.network.phoenix_tag),
        evt,
        Vec::new(),
    );
}

fn warn_msg(msg_type: &str, topic: &str, payload: &[u8]) {
    warn(format!(
        "Received `{}` message with topic: {}, payload: {:?}",
        msg_type, topic, payload
    ))
}

pub fn handle_server_msg(bytes: &[u8]) {
    let ChannelMessage {
        topic,
        event,
        field_ref,
        join_ref,
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
        Some(evt) => match evt.payload {
            Some(EventPayload::custom_event(evt)) => match evt {
                _ => warn_msg(&evt, &topic, &payload),
            },
            Some(EventPayload::phoenix_event(evt)) => match evt {
                PhoenixEvent::Close => warn_msg("close", &topic, &payload),
                PhoenixEvent::Join => warn_msg("join", &topic, &payload),
                PhoenixEvent::Reply => warn_msg("reply", &topic, &payload),
                PhoenixEvent::Heartbeat => {
                    let mut evt = Event::new();
                    evt.set_phoenix_event(PhoenixEvent::Heartbeat);
                    send_channel_message("phoenix", evt, Vec::new())
                }
            },
            None => error("Received channel event with no inner payload!"),
        },
        None => warn("Received channel message with no event payload!"),
    }
}
