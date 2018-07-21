defmodule BackendWeb.GameChannel do
  use Phoenix.channel
  alias BackendWeb.GameState

  def join("game:first", _payload, socket) do
    send(self, :after_join)
    {:ok, socket}
  end

  def handle_info(:after_join, socket) do
    push socket, "presence_state", GameState.list(socket)
    {:ok, _} = GameState.track(socket, UUID.uuid4(), %{
      online_at: inspect(System.system_time(:seconds))
    })
    {:noreply, socket}
  end
end
