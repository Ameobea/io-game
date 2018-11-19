defmodule BackendWeb.GameLoop do
  use GenServer
  alias BackendWeb.GameState
  alias BackendWeb.GameConf
  alias BackendWeb.NativePhysicsServer
  alias NativePhysics
  alias Backend.ProtoMessage
  alias Backend.ProtoMessage.{ServerMessage, Point2}

  @ticks_per_second 60
  @microseconds_per_second 1_000_000

  def init(_) do
    start_tick()
    {:ok, %{}}
  end

  def start_link() do
    GenServer.start_link(__MODULE__, nil, name: __MODULE__)
  end

  def queue_message(topic, message = {_player_id, _key, _value}) do
    GenServer.call(__MODULE__, {:handle_message, topic, message})
  end

  def handle_info(:tick, state) do
    {:noreply, run_tick(state)}
  end

  def handle_call({:handle_message, topic, new_message}, _from, messages) do
    diff = NativePhysics.UserDiff.new(new_message)
    { :reply, nil, Map.put(messages, topic, [diff | Map.get(messages, topic, [])]) }
  end

  defp run_tick(messages) do
    {cur_tick, prev_tick_time} = GameState.get_cur_tick_info
    topics = GameState.list_topics()
    if Enum.empty?(topics) do
      Process.send_after(self(), :tick, 16)
    else
      update_topics(topics, cur_tick, prev_tick_time, messages)
    end

    %{}
  end

  defp update_topics([], _tick, _prev_tick_time, _messages), do: nil
  defp update_topics([topic | rest], tick, prev_tick_time, messages) do
    topic_state = GameState.get_topic(topic)
    messages_for_topic = Map.get(messages, topic, [])
    update_topic(topic, topic_state, tick, prev_tick_time, messages_for_topic)
    GameState.set_topic(topic, topic_state)

    update_topics(rest, tick, prev_tick_time, messages)
  end

  defp update_topic(topic, _topic_state, tick, prev_tick_time, player_inputs) do
    snapshot_tick_interval = GameConf.get_config "network", "snapshotTickInterval"
    send_snapshot = rem(tick, snapshot_tick_interval) == 0
    time_diff_us = (System.system_time / 1000.0) - (prev_tick_time / 1000.0)
    desired_delay_us = @microseconds_per_second / @ticks_per_second
    delay_us = desired_delay_us - time_diff_us - 400.0
    delay_us = if delay_us < 0 do
      0
    else
      Kernel.trunc delay_us
    end

    spawn NativePhysicsServer, :tick, [Enum.reverse(player_inputs), send_snapshot, delay_us, topic]
  end

  @spec handle_updates([UserDiff], String.t()) :: nil
  def handle_updates(updates, topic) do
    if is_list updates do
      payload = updates
        |> Enum.map(&handle_update/1)
        |> Enum.filter(& !is_nil(&1))

      if !Enum.empty? payload do
        BackendWeb.Endpoint.broadcast! topic, "tick", %{response: payload}
      end
    else
      IO.inspect ["PHYSICS ENGINE ERROR", updates]
    end

    # TODO: This is only valid if we have a single topic.  If we have multiple topics, we will
    # need to emulate a `Promise.all` and keep track of how many of these have finished before
    # starting the next tick.  So damned annoying...
    {next_tick, cur_time} = GameState.incr_tick
    start_tick()
  end

  defp construct_payload(id, inner_payload) do
    ServerMessage.Payload.new(%{
      id: ProtoMessage.to_proto_uuid(id),
      payload: inner_payload,
    })
  end

  defp handle_update(%NativePhysics.Update{
    id: id,
    update_type: :isometry,
    payload: payload,
  }) do
    internal_movement_update = payload
      |> Map.from_struct
      |> Backend.ProtoMessage.MovementUpdate.new

    construct_payload id, { :movement_update, internal_movement_update }
  end

  defp handle_update(%NativePhysics.Update{
    id: id,
    update_type: :player_movement,
    payload: payload,
  }) do
    construct_payload id, { :player_input, payload }
  end

  defp handle_update(%NativePhysics.Update{
    id: id,
    update_type: :beam_toggle,
    payload: payload,
  }) do
    construct_payload id, { :beam_toggle, payload }
  end

  defp handle_update(%NativePhysics.Update{
    id: id,
    update_type: :beam_aim,
    payload: payload,
  }) do
    construct_payload id, { :beam_aim, Point2.new(payload) }
  end

  defp handle_update(unmatched) do
    IO.inspect(["~~~~!!!! UNMATCHED UPDATE", unmatched])
    nil
  end

  defp start_tick() do
    Process.send_after(__MODULE__, :tick, 0)
  end
end
