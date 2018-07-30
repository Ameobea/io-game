defmodule BackendWeb.GameLoop do
  use GenServer
  alias BackendWeb.GameState

  @timedelay 20000
  @nanoseconds_to_seconds 1_000_000_000

  def init(_state) do
    start_tick()
    {:ok, {get_time(), []}}
  end

  def start_link(default \\ "my game loop") do
    GenServer.start_link(__MODULE__, default)
  end

  def queue_message(topic, message) do
    GenServer.call(__MODULE__, {:handle_message, topic, message})
    :ok
  end

  def handle_info(:tick, state) do
    start_tick()
    {:noreply, run_tick(state)}
  end

  def handle_call({:handle_message, topic, new_message}, _from, {time, messages}) do
    {
      :noreply,
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
    {:something_important} = calculate_messages(messages[topic])
    GameState.update_topic(topic, &update_topic(&1, time_diff, :something_important))
    update_topics(rest, time_diff, messages)
  end

  defp update_topic(topic_state, time_diff, _player_updates) do
    player_ids = Map.keys(topic_state)
    update_players(player_ids, topic_state)
  end

  defp update_players(topic_state, []), do: topic_state
  defp update_players(topic_state, [player_id | rest]) do
    topic_state
    |> Map.update(player_id, %{}, &(&1))
    |> update_players(rest)
  end

  defp calculate_messages(messages) do
    {:something_important}
  end

  defp start_tick() do
    Process.send_after(self(), :tick, @timedelay)
  end

  defp get_time(), do: System.system_time
end
