defmodule Backend.ProtoMessage do
  use Protobuf, from: Path.wildcard(Path.expand("../../../schema/**/*.proto", __DIR__))

  alias Backend.ProtoMessage.{
    Uuid,
    Event,
    PhoenixEvent,
    ServerChannelMessage,
    ServerMessage,
    ServerError,
  }

  def encode_socket_message(%Phoenix.Socket.Message{payload: %{status: :error}} = message) do
    ServerChannelMessage.encode(ServerChannelMessage.new(%{
      topic: "rooms:game",
      event: Event.new(%{payload: {:phoenix_event, :Error}}),
      ref: nil,
      payload: ServerMessage.new(%{
        id: Uuid.new(%{data_1: 0, data_2: 0}),
        payload: {
          :error,
          ServerError.new(%{reason: message.payload.response.reason}),
        },
      }),
    }))
  end

  def encode_socket_message(%Phoenix.Socket.Message{} = message) do
    IO.inspect ["payload", message.payload]
    aa = ServerChannelMessage.encode(ServerChannelMessage.new(%{
      topic: message.topic,
      event: encode_event(message.event),
      ref: message.ref,
      payload: encode_payload(message.payload),
    }))
    IO.inspect ["out event", message, aa]
    aa
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
      |> Map.to_list()
      |> Enum.map(&to_snapshot_item/1)
    Snapshot.new(%{items: items})
  end

  defp to_snapshot_item({player_id, data = %{}}) do
    %{
      pos_x: pos_x,
      pos_y: pos_y,
      size: size,
      velocity_x: velocity_x,
      velocity_y: velocity_y,
    } = data
    Snapshot.SnapshotItem.new(%{
      id: player_id,
      item: CreationEvent.new(%{
        pos_x: pos_x,
        pos_y: pos_y,
        entity: {:player, PlayerEntity.new(%{
          size: size,
          velocity_x: velocity_x,
          velocity_y: velocity_y,
        })}
      }),
    })
  end

  defp encode_event("phx_" <> event) do
    phx_event = PhoenixEvent.value(event |> String.capitalize |> String.to_atom)
    Event.new(%{payload: {:phoenix_event, phx_event} })
  end

  defp encode_event(other_event) do
    Event.new(%{payload: {:custom_event, other_event} })
  end

  defp encode_payload(%{response: payload}), do: payload
  defp encode_payload(%{}), do: nil
  defp encode_payload(payload), do: payload
end
