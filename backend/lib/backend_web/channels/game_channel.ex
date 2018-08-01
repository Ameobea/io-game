defmodule BackendWeb.GameChannel do
  use Phoenix.Channel
  alias BackendWeb.{GameState, GameLoop}

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
      ServerMessage.new(%{id: Backend.ProtoMessage.to_proto_uuid(uuid)}),
      assign(socket, :player_id, uuid)
    }
  end

  def handle_info(:after_join, socket) do
    # push socket, "presence_state", GameState.list(socket)
    IO.inspect ["channel_pid", socket.channel_pid]
    :ok = GameState.track_player(socket.topic, socket.assigns.player_id, %{
      x: 0,
      y: 0,
    })
    {:noreply, socket}
  end

  def handle_in("game", %ClientMessage{payload: payload}, socket) do
    IO.inspect ["handle_in game payload", payload]
    socket = handle_payload("game", payload, socket)
    # client_message = ClientMessage.decode(data)
    {:noreply, socket}
  end

  def handle_in("temp_gen_server_message_1", _data, socket) do
    msg = Backend.ProtoMessage.temp_gen_server_message_1
    push(socket, "temp_gen_server_message_1_res", %{msg: :binary.bin_to_list(msg)})
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
    socket
  end
end
