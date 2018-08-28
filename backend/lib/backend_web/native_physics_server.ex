# Because Elixir just is that way.

defmodule BackendWeb.NativePhysicsServer do
  alias NativePhysics
  alias BackendWeb.GameLoop

  use GenServer

  def init(_) do
    {:ok, nil}
  end

  def start_link() do
    GenServer.start_link(__MODULE__, nil, name: __MODULE__)
  end

  def tick(player_inputs, send_snapshot, delay_us, topic) do
    GenServer.call(__MODULE__, {:tick, player_inputs, send_snapshot, delay_us, topic})
  end

  def handle_call({:tick, player_inputs, send_snapshot, delay_us, topic}, _from, nil) do
    updates = NativePhysics.tick(player_inputs, send_snapshot, delay_us)
    GameLoop.handle_updates(updates, topic)
    {:reply, nil, nil}
  end

  def handle_info(:tick, _), do: {:noreply, nil}
end
