defmodule BackendWeb.GameChannel do
  use Phoenix.Channel
  alias BackendWeb.GameState

  alias Backend.ProtoMessage.{
    ServerMessage,
    ClientMessage,
    ConnectMessage,
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
    {:ok, _} = GameState.track(socket, socket.assigns.player_id, %{
      online_at: inspect(System.system_time(:seconds)),
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

  def handle_payload("game", {:connect, %ConnectMessage{username: username}}, socket) do
    assign(socket, :username, username)
  end

  def handle_in("move_up", _data, socket) do
    player_info = GameState.get_player(socket)
    %{ x: x, y: y } = player_info
    GameState.update(socket, socket.assigns.player_id, %{x: x, y: y + 1})
    {:noreply, socket}
  end

  def handle_in("temp_gen_server_message_1", _data, socket) do
    msg = Backend.ProtoMessage.temp_gen_server_message_1
    push(socket, "temp_gen_server_message_1_res", %{msg: :binary.bin_to_list(msg)})
    {:noreply, socket}
  end
end
