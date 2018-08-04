defmodule BackendWeb.GameState do
  use GenServer

  def init(_) do
    {:ok, %{}}
  end

  def start_link() do
    GenServer.start_link(__MODULE__, nil, name: __MODULE__)
  end

  def get_player(topic, player_id) do
    GenServer.call(__MODULE__, {:get_topic, topic})
    |> Map.get(player_id)
  end

  def track_player(topic, player_id, initial_state) do
    GenServer.call(__MODULE__, {:track_player, topic, player_id, initial_state})
  end

  def update_topic(topic, update_fn) do
    GenServer.call(__MODULE__, {:update_topic, topic, update_fn})
  end

  def get_topic(topic) do
    GenServer.call(__MODULE__, {:get_topic, topic})
  end

  def list_topics() do
    GenServer.call(__MODULE__, :list_topics)
  end

  def handle_call({:track_player, topic, player_id, initial_state}, _from, state) do
    {:reply, :ok, deep_merge(state, %{topic => %{player_id => initial_state}})}
  end

  def handle_call({:get_topic, topic}, _from, state) do
    {:reply, Map.get(state, topic, %{}), state}
  end

  def handle_call(:list_topics, _from, state) do
    {:reply, Map.keys(state), state}
  end

  def handle_call({:update_topic, topic, update_fn}, _from, state) do
    new_state = Map.update(state, topic, %{}, update_fn)
    {:reply, new_state, new_state}
  end

  defp deep_merge(left, right), do: Map.merge(left, right, &merge_inner/3)
  defp merge_inner(_key, %{} = left, %{} = right), do: deep_merge(left, right)
  defp merge_inner(_key, _left, right), do: right
end
