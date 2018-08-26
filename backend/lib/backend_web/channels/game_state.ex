defmodule BackendWeb.GameState do
  use GenServer

  def init(_) do
    {:ok, {%{}, 0, get_time()}}
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

  def set_topic(topic, topic_state) do
    GenServer.call(__MODULE__, {:set_topic, topic, topic_state})
  end

  def get_topic(topic) do
    GenServer.call(__MODULE__, {:get_topic, topic})
  end

  def list_topics() do
    GenServer.call(__MODULE__, :list_topics)
  end

  def get_cur_tick_info() do
    GenServer.call(__MODULE__, :get_cur_tick_info)
  end

  def incr_tick() do
    GenServer.call(__MODULE__, :incr_tick)
  end

  def handle_call({:track_player, topic, player_id, initial_state}, _from, {topics, tick, timestamp}) do
    new_topics = deep_merge(topics, %{topic => %{player_id => initial_state}})
    {:reply, :ok, {new_topics, tick, timestamp}}
  end

  def handle_call({:get_topic, topic}, _from, {topics, tick, timestamp}) do
    {:reply, Map.get(topics, topic, %{}), {topics, tick, timestamp}}
  end

  def handle_call({:set_topic, topic, topic_state}, _from, {topics, tick, timestamp}) do
    {:reply, nil, {Map.put(topics, topic, topic_state), tick, timestamp}}
  end

  def handle_call(:list_topics, _from, {topics, tick, timestamp}) do
    {:reply, Map.keys(topics), {topics, tick, timestamp}}
  end

  def handle_call({:update_topic, topic, update_fn}, _from, {topics, tick, timestamp}) do
    new_topics = Map.update(topics, topic, %{}, update_fn)
    {:reply, new_topics, {new_topics, tick, timestamp}}
  end

  def handle_call(:get_cur_tick_info, _from, {topics, tick, timestamp}) do
    {:reply, {tick, timestamp}, {topics, tick, timestamp}}
  end

  def handle_call(:incr_tick, _from, {topics, tick, timestamp}) do
    new_tick = tick + 1
    new_timestamp = get_time()

    new_state = {topics, new_tick, new_timestamp}
    {:reply, {new_tick, new_timestamp}, new_state}
  end

  defp deep_merge(left, right), do: Map.merge(left, right, &merge_inner/3)
  defp merge_inner(_key, %{} = left, %{} = right), do: deep_merge(left, right)
  defp merge_inner(_key, _left, right), do: right

  defp get_time(), do: System.system_time
end
