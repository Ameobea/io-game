defmodule Backend.ProtoMessage do
  use Protobuf, from: Path.wildcard(Path.expand("../../../schema/**/*.proto", __DIR__))

  alias Backend.ProtoMessage.{
    Uuid,
    Event,
    PhoenixEvent,
    ServerChannelMessage,
    ServerMessage,
    ServerError,
    Snapshot,
    CreationEvent,
    PlayerEntity,
    AsteroidEntity,
  }
  alias NativePhysics
  alias NativePhysics.MovementUpdate

  @entity_types %{
    player: PlayerEntity,
    asteroid: AsteroidEntity
  }

  def encode_socket_message(%Phoenix.Socket.Message{payload: %{status: :error}} = message) do
    ServerChannelMessage.encode(ServerChannelMessage.new(%{
      topic: "rooms:game",
      event: Event.new(%{payload: {:phoenix_event, :Error}}),
      ref: nil,
      payload: ServerMessage.new(%{
        tick: 0, # TODO
        timestamp: 0, # TODO
        payload: [
          ServerMessage.Payload.new(%{
            id: Uuid.new(%{data_1: 0, data_2: 0}),
            payload: {
              :error,
              ServerError.new(%{reason: message.payload.response.reason}),
            }
          })
        ],
      }),
    }))
  end

  def encode_socket_message(%Phoenix.Socket.Message{} = message) do
    IO.inspect(["~~~PAYLOAD", message.payload])
    ServerChannelMessage.encode(ServerChannelMessage.new(%{
      topic: message.topic,
      event: encode_event(message.event),
      ref: message.ref,
      payload: encode_payload(message.payload),
    }))
  end

  def to_proto_uuid(uuid) do
    [part2, part1] = uuid
      |> UUID.string_to_binary!
      |> :binary.bin_to_list
      |> Enum.reverse
      |> Enum.chunk(8)
      |> Enum.map(&Integer.undigits(&1, 256))

    Uuid.new(%{data_1: part1, data_2: part2})
  end

  def generate_uuid() do
    UUID.uuid4() |> to_proto_uuid
  end

  def encode_game_state_to_snapshot(%{} = game_state) do
    items = game_state
      |> Map.merge(NativePhysics.get_snapshot(), fn _, a, b -> Map.merge(a, Map.from_struct(b)) end)
      |> Map.to_list()
      |> Enum.map(&to_snapshot_item/1)
    Snapshot.new(%{items: items})
  end

  def create_server_message([] = payloads) do
    ServerMessage.new(%{
      tick: 0, # TODO: Replace with current tick from game state
      timestamp: 0.0, # TODO: Replace with timestamp of current tick
      payload: encode_payload(payloads),
    })
  end

  defp to_snapshot_item({player_id, data = %{}}) do
    %{
      id: entity_id,
      movement: movement,
      entity_type: entity_type,
      entity_meta: entity_meta,
    } = data
    Snapshot.SnapshotItem.new(%{
      id: entity_id,
      item: CreationEvent.new(%{
        movement: movement,
        entity: encode_entity(entity_type, entity_meta),
      }),
    })
  end

  defp encode_entity(entity_type, entity_meta), do: @entity_types[entity_type].new(entity_meta)

  defp encode_event("phx_" <> event) do
    phx_event = PhoenixEvent.value(event |> String.capitalize |> String.to_atom)
    Event.new(%{payload: {:phoenix_event, phx_event} })
  end

  defp encode_event(other_event) do
    Event.new(%{payload: {:custom_event, other_event} })
  end

  defp encode_payload(%{response: payload}), do: payload
  defp encode_payload(%{}), do: nil
  defp encode_payload(payloads) when is_list(payloads) do
    Enum.map(payloads, &encode_payload/1)
  end
  defp encode_payload(%{response: payload}), do: payload
  # defp encode_payload(payload), do: payload
end
