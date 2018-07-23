defmodule Backend.Message do
  use Protobuf, from: Path.wildcard(Path.expand("../../../schema/**/*.proto", __DIR__))
  alias Backend.Message.{PlayerEntity, CreationEvent, StatusUpdate, ServerMessage, Uuid}

  def temp_gen_server_message_1() do
    entity = PlayerEntity.new(%{size: 60, direction: :STOP})
    event = CreationEvent.new(%{
      pos_x: 50,
      pos_y: 50,
      entity: {:player, entity}
    })
    status_update = StatusUpdate.new(%{payload: {:creation_event, event}})
    server_message = ServerMessage.new(%{id: generate_uuid(), payload: {:status_update, status_update}})
    ServerMessage.encode(server_message)
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

end
