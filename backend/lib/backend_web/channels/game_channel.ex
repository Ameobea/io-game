defmodule BackendWeb.GameChannel do
  use Phoenix.Channel
  alias BackendWeb.{GameState, GameLoop}

  alias Backend.ProtoMessage
  alias Backend.ProtoMessage.{
    ServerMessage,
    ClientMessage,
    ConnectMessage,
    BeamAim,
  }

  def join("rooms:game", _payload, socket) do
    send(self(), :after_join)
    uuid = UUID.uuid4()
    {
      :ok,
      ServerMessage.new(%{id: ProtoMessage.to_proto_uuid(uuid)}),
      assign(socket, :player_id, uuid)
    }
  end

  def handle_info(:after_join, socket) do
    :ok = GameState.track_player(socket.topic, socket.assigns.player_id, %{
      pos_x: 0,
      pos_y: 0,
      size: 100,
      velocity_x: 0,
      velocity_y: 0,
    })
    proto_game_state = GameState.get_topic(socket.topic) |> ProtoMessage.encode_game_state_to_snapshot
    push socket, "current_game_state", proto_game_state
    {:noreply, socket}
  end

  def handle_in("game", %ClientMessage{payload: payload}, socket) do
    # IO.inspect ["handle_in game payload", payload]
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
