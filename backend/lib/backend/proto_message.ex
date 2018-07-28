defmodule Backend.ProtoMessage do
  use Protobuf, from: Path.wildcard(Path.expand("../../../schema/**/*.proto", __DIR__))

  alias Backend.ProtoMessage.{
    PlayerEntity,
    CreationEvent,
    StatusUpdate,
    ServerMessage,
    Uuid,
    Event,
    PhoenixEvent,
    ServerChannelMessage,
  }

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

  def temp_gen_server_message_1() do
    entity = PlayerEntity.new(%{size: 60, direction: :STOP})
    event = CreationEvent.new(%{
      pos_x: 50,
      pos_y: 50,
      entity: {:player, entity}
    })
    status_update = StatusUpdate.new(%{payload: {:creation_event, event}})
    ServerMessage.new(%{id: generate_uuid(), payload: {:status_update, status_update}})
    # ServerMessage.encode(server_message)
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
