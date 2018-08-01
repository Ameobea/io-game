defmodule BackendWeb.GameLoop do
  use GenServer
  alias BackendWeb.GameState

  @timedelay 5000
  @nanoseconds_to_seconds 1_000_000_000

  def init(_) do
    start_tick()
    {:ok, {get_time(), %{}}}
  end

  def start_link() do
    GenServer.start_link(__MODULE__, nil, name: __MODULE__)
  end

  def queue_message(topic, message = {_player_id, _key, _value}) do
    GenServer.call(__MODULE__, {:handle_message, topic, message})
  end

  def handle_info(:tick, state) do
    start_tick()
    {:noreply, run_tick(state)}
  end

  def handle_call({:handle_message, topic, new_message}, _from, {time, messages}) do
    {
      :reply,
      nil,
      {
        time,
        Map.put(messages, topic, [new_message | Map.get(messages, topic, [])])
      }
    }
  end

  defp run_tick({prev_time, messages}) do
    curr_time = get_time()
    time_difference = (curr_time - prev_time) / @nanoseconds_to_seconds

    GameState.list_topics()
    |> update_topics(time_difference, messages)

    # asd = GameState.list("rooms:game")
    {curr_time, %{}}
  end

  defp update_topics([], _time_diff, _messages), do: nil
  defp update_topics([topic | rest], time_diff, messages) do
    player_inputs = calculate_messages(messages[topic])
    GameState.update_topic(topic, &update_topic(&1, time_diff, player_inputs))
    update_topics(rest, time_diff, messages)
  end

  defp update_topic(topic_state, _time_diff, _player_inputs) do
    player_ids = Map.keys(topic_state)
    update_players(topic_state, player_ids)
  end

  defp update_players(topic_state, []), do: topic_state
  defp update_players(topic_state, [player_id | rest]) do
    topic_state
    |> Map.update(player_id, %{}, &(&1))
    |> update_players(rest)
  end

  defp calculate_messages(nil), do: %{}
  defp calculate_messages(messages), do: calculate_messages(%{}, Enum.reverse(messages))
  defp calculate_messages(acc, []), do: acc
  defp calculate_messages(acc, [{player_id, key, value} | rest]) do
    player_input = Map.put(acc[player_id] || %{}, key, value)
    Map.put(acc, player_id, player_input)
    |> calculate_messages(rest)
  end

  defp start_tick() do
    Process.send_after(self(), :tick, @timedelay)
  end

  defp get_time(), do: System.system_time
end
