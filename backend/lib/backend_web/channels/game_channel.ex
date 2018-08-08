defmodule BackendWeb.GameChannel do
  use Phoenix.Channel
  alias BackendWeb.{GameState, GameLoop}

  alias Backend.ProtoMessage
  alias Backend.ProtoMessage.{
    ServerMessage,
    ClientMessage,
    ConnectMessage,
    BeamAim,
    StatusUpdate,
    CreationEvent,
    PlayerEntity,
  }
  alias NativePhysics

  def join("rooms:game", _payload, socket) do
    send(self(), :after_join)
    uuid = UUID.uuid4()

    movement_update = NativePhysics.spawn_user(uuid)
    internal_movement_update = movement_update
      |> Map.from_struct
      |> Backend.ProtoMessage.MovementUpdate.new

    {
      :ok,
      ServerMessage.new(%{
        tick: 0, # TODO
        timestamp: 0, # TODO
        payload: [
          ServerMessage.Payload.new(%{
            id: ProtoMessage.to_proto_uuid(uuid),
            payload: {
              :status_update,
              StatusUpdate.new(%{
                payload: {
                  :creation_event,
                  CreationEvent.new(%{
                    movement: internal_movement_update,
                    entity: {
                      :player,
                      PlayerEntity.new(%{
                        size: 20, # TODO: read from config
                      }),
                    },
                  }),
                },
              }),
            },
          }),
        ],
      }),
      assign(socket, :player_id, uuid)
    }
  end

  def handle_info(:after_join, socket) do
    :ok = GameState.track_player(socket.topic, socket.assigns.player_id, %{})
    proto_game_state = GameState.get_topic(socket.topic) |> ProtoMessage.encode_game_state_to_snapshot
    push socket, "current_game_state", proto_game_state
    {:noreply, socket}
  end

  def handle_in("game", %ClientMessage{payload: payload}, socket) do
    handle_payload("game", payload, socket)
    {:noreply, socket}
  end

  defp handle_payload("game", {:connect, %ConnectMessage{username: username}}, socket) do
    queue_user_input(socket, :username, username)
  end
  defp handle_payload("game", {:player_move, direction}, socket) do
    queue_user_input(socket, :direction, direction)
  end
  defp handle_payload("game", {:beam_toggle, toggle}, socket) do
    queue_user_input(socket, :beam_toggle, toggle)
  end
  defp handle_payload("game", {:beam_rotation, %BeamAim{} = aim}, socket) do
    queue_user_input(socket, :beam_rotation, aim)
  end

  defp queue_user_input(socket, key, value) do
    GameLoop.queue_message(socket.topic, {socket.assigns.player_id, key, value})
  end
end
