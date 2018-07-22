defmodule BackendWeb.GameState do
  use Phoenix.Presence, otp_app: :backend,
                      pubsub_server: Backend.PubSub

  def get_player(socket) do
    hd list(socket)[socket.assigns.player_id].metas
  end
end
