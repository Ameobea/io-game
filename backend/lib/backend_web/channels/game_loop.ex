defmodule BackendWeb.GameLoop do
  use GenServer
  alias BackendWeb.GameState
  alias BackendWeb.GameConf
  alias NativePhysics
  alias Backend.ProtoMessage
  alias Backend.ProtoMessage.{ServerMessage, Point2}

  @timedelay 17
  @nanoseconds_to_seconds 1_000_000_000

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
    start_tick()
    {:noreply, run_tick(state)}
  end

  def handle_call({:handle_message, topic, new_message}, _from, messages) do
    diff = NativePhysics.UserDiff.new(new_message)
    { :reply, nil, Map.put(messages, topic, [diff | Map.get(messages, topic, [])]) }
  end

  defp run_tick(messages) do
    {_, prev_time} = GameState.get_cur_tick_info
    {cur_tick, cur_time} = GameState.incr_tick
    time_difference = (cur_time - prev_time) / @nanoseconds_to_seconds

    topics = GameState.list_topics()

    topics |> update_topics(cur_tick, time_difference, messages)

    %{}
  end

  defp update_topics([], _tick, _time_diff, _messages), do: nil
  defp update_topics([topic | rest], tick, time_diff, messages) do
    topic_state = GameState.get_topic(topic)
    updated_topic = update_topic(topic, topic_state, tick, time_diff, Map.get(messages, topic, []))
    GameState.set_topic(topic, updated_topic)

    update_topics(rest, tick, time_diff, messages)
  end

  defp update_topic(topic, topic_state, tick, time_diff, player_inputs) do
    # TODO: rather than reversing player inputs here, just push them to the front of the buffer
    snapshot_tick_interval = GameConf.get_config("network", "snapshotTickInterval")
    send_snapshot = rem(tick, snapshot_tick_interval) == 0
    updates = NativePhysics.tick(player_inputs |> Enum.reverse, send_snapshot)
    if is_list(updates) do
      payload = updates
        |> Enum.map(&handle_update/1)
        |> Enum.filter(& !is_nil(&1))

      if Enum.count(payload) > 0 do
        BackendWeb.Endpoint.broadcast! topic, "tick", %{response: payload}
      end
    else
      IO.inspect(["PHYSICS ENGINE ERROR", player_inputs, updates])
    end

    topic_state
  end

  defp handle_update(%NativePhysics.Update{
    id: id,
    update_type: :isometry,
    payload: payload,
  }) do
    internal_movement_update = payload
      |> Map.from_struct
      |> Backend.ProtoMessage.MovementUpdate.new

    ServerMessage.Payload.new(%{
      id: ProtoMessage.to_proto_uuid(id),
      payload: {
        :movement_update,
        internal_movement_update,
      }
    })
  end

  defp handle_update(%NativePhysics.Update{
    id: id,
    update_type: :player_movement,
    payload: payload,
  }) do
    ServerMessage.Payload.new(%{
      id: ProtoMessage.to_proto_uuid(id),
      payload: {
        :player_input,
        payload
      }
    })
  end

  defp handle_update(%NativePhysics.Update{
    id: id,
    update_type: :beam_toggle,
    payload: payload,
  }) do
    ServerMessage.Payload.new(%{
      id: ProtoMessage.to_proto_uuid(id),
      payload: {
        :beam_toggle,
        payload
      }
    })
  end

  defp handle_update(%NativePhysics.Update{
    id: id,
    update_type: :beam_aim,
    payload: payload,
  }) do
    ServerMessage.Payload.new(%{
      id: ProtoMessage.to_proto_uuid(id),
      payload: { :beam_aim, Point2.new(payload) }
    })
  end

  defp handle_update(unmatched) do
    IO.inspect(["~~~~!!!! UNMATCHED UPDATE", unmatched])
    nil
  end

  defp start_tick() do
    Process.send_after(self(), :tick, @timedelay)
  end
end
