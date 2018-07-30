defmodule BackendWeb.GameState do
  use GenServer
  def init(_state) do
    {:ok, %{}}
  end

  def track_player(topic, player_id, initial_state) do
    GenServer.call(__MODULE__, {:track_player, topic, player_id, initial_state})
    :ok
  end

  def update_topic(topic, update_fn) do
    GenServer.call(__MODULE__, {:update_topic, topic, update_fn})
    :ok
  end

  def get_topic(topic) do
    GenServer.call(__MODULE__, {:get_topic, topic})
  end

  def list_topics() do
    GenServer.call(__MODULE__, :list_topics)
  end

  def handle_call({:track_player, topic, player_id, initial_state}, _from, state) do
    {:noreply, deep_merge(state, %{topic => %{player_id => initial_state}})}
  end

  def handle_call({:get_topic, topic}, _from, state) do
    {:reply, state[topic], state}
  end

  def handle_call(:list_topics, _from, state) do
    {:reply, Map.keys(state), state}
  end

  def handle_call({:update_topic, topic, update_fn}, _from, state) do
    {:noreply, Map.update(state, topic, %{}, update_fn)}
  end

  defp deep_merge(left, right), do: Map.merge(left, right, &merge_inner/3)
  defp merge_inner(_key, %{} = left, %{} = right), do: deep_merge(left, right)
  defp merge_inner(_key, left, right), do: right
end

  def init(_state) do
    start_tick()
    {:ok, get_time()}
  end

  def start_link(default \\ "my game loop") do
    GenServer.start_link(__MODULE__, default)
  end

  def handle_info(:tick, state) do
    start_tick()
    {:noreply, run_tick(state)}
  end

  defp run_tick(prev_time) do
    curr_time = get_time()
    time_difference = (curr_time - prev_time) / @nanoseconds_to_seconds
    # asd = GameState.list("rooms:game")
    curr_time
  end

  defp start_tick() do
    Process.send_after(self(), :tick, @timedelay)
  end

  defp get_time(), do: System.system_time
end
