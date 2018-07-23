defmodule Backend.Message do
  use Protobuf, from: Path.wildcard(Path.expand("../../../schema/**/*.proto", __DIR__))
  alias Backend.Message.{PlayerEntity, CreationEvent, StatusUpdate, ServerMessage}

  def temp_gen_server_message_1() do
    entity = PlayerEntity.new(%{size: 60, direction: :STOP})
    event = CreationEvent.new(%{
      pos_x: 50,
      pos_y: 50,
      entity: {:player, entity}
    })
    status_update = StatusUpdate.new(%{payload: {:creation_event, event}})
    server_message = ServerMessage.new(%{payload: {:status_update, status_update}})
    ServerMessage.encode(server_message)
  end

end
