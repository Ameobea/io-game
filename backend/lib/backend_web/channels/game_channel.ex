defmodule BackendWeb.GameChannel do
  use Phoenix.Channel
  alias BackendWeb.GameState

  def join("game:first", _payload, socket) do
    send(self(), :after_join)
    {:ok, assign(socket, :player_id, UUID.uuid4())}
  end

  def handle_info(:after_join, socket) do
    push socket, "presence_state", GameState.list(socket)
    {:ok, _} = GameState.track(socket, socket.assigns.player_id, %{
      online_at: inspect(System.system_time(:seconds)),
      x: 0,
      y: 0,
    })
    {:noreply, socket}
  end

  def handle_in("move_up", _data, socket) do
    player_info = GameState.get_player(socket)
    %{ x: x, y: y } = player_info
    GameState.update(socket, socket.assigns.player_id, %{x: x, y: y + 1})
    {:noreply, socket}
  end

  def handle_in("temp_gen_server_message_1", _data, socket) do
    msg = <<0, 0>> <> Backend.Message.temp_gen_server_message_1
    push(socket, "temp_gen_server_message_1_res", %{msg: msg})
    {:noreply, socket}
  end
end
