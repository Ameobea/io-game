defmodule Backend.ProtoMessage do
  use Protobuf, from: Path.wildcard(Path.expand("../../../schema/**/*.proto", __DIR__))

  alias Backend.ProtoMessage.{
    PlayerEntity,
    CreationEvent,
    StatusUpdate,
    ServerMessage,
    Uuid,
    Payload,
    Event,
    ChannelMessage,
  }

  def encode_socket_message(%Phoenix.Socket.Message{} = message) do
    ChannelMessage.encode(ChannelMessage.new(%{
      topic: message.topic,
      event: Event.new(%{:})
      ref: message.ref,
      payload: encode_payload(message.payload),
    }))
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

  def generate_uuid() do
    [part2, part1] = UUID.uuid4()
      |> UUID.string_to_binary!
      |> :binary.bin_to_list
      |> Enum.reverse
      |> Enum.chunk(8)
      |> Enum.map(&Integer.undigits(&1, 256))

    Uuid.new(%{data_1: part1, data_2: part2})
  end

  defp encode_payload(nil), do: nil
  defp encode_payload(%{}), do: nil
  defp encode_payload(payload) do
    Payload.new(payload)
  end

  defp encode_event(evt) do
    # Takes a string like "phs_join" and maps it to a protobuf `Event`
    # https://github.com/mcampa/phoenix-channels/blob/b013df9a325c37c49e5b335fc63f18d85e95430d/src/constants.js#L13-L18
  end
end
