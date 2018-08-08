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
    ServerMessage.Payload,
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
    msg = ServerChannelMessage.new(%{
      topic: message.topic,
      event: encode_event(message.event),
      ref: message.ref,
      payload: encode_payload(message.payload),
    })
    ServerChannelMessage.encode(msg)
  end

  # Converts a UUID into two unsigned 64-bit integers representing its raw byte data
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

  # Merge the physics state from the backend with the state held in Elixir
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

  # TODO: optimize this iteration
  defp to_snapshot_item({player_id, data = %{}}) do
    %{
      id: entity_id,
      movement: movement,
      entity_type: entity_type,
      entity_meta: entity_meta,
    } = data
    Snapshot.SnapshotItem.new(%{
      id: to_proto_uuid(entity_id),
      item: CreationEvent.new(%{
        movement: movement |> Map.from_struct |> Backend.ProtoMessage.MovementUpdate.new,
        entity: encode_entity(entity_type, entity_meta),
      }),
    })
  end

  defp encode_entity(entity_type, entity_meta) do
    # Convert map keys from strings to atoms
    # mapped_entity_meta = entity_meta |> Map.new(fn {k, v} -> {String.to_atom(k), v} end)
    IO.inspect(["ENCODING ENTITY", entity_type, entity_meta])
    {entity_type, @entity_types[entity_type].new(entity_meta)}
  end

  defp encode_event("phx_" <> event) do
    phx_event = PhoenixEvent.value(event |> String.capitalize |> String.to_atom)
    Event.new(%{payload: {:phoenix_event, phx_event} })
  end

  defp encode_event(other_event) do
    Event.new(%{payload: {:custom_event, other_event} })
  end

  defp encode_payload(%{response: payload}), do: encode_payload(payload)
  defp encode_payload(%{__struct__: ServerMessage.Payload} = payload), do: payload
  defp encode_payload(payloads) when is_list(payloads) do
    ServerMessage.new(%{
      tick: 0, # TODO
      timestamp: 0, # TODO
      payload: Enum.map(payloads, &encode_payload/1),
    })
  end
  defp encode_payload(%{}), do: nil
  defp encode_payload(payload), do: payload
end
