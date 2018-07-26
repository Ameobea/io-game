defmodule BackendWeb.ProtoSerializer do

  @behaviour Phoenix.Transports.Serializer

  alias Phoenix.Socket.{Reply, Message, Broadcast}

  def fastlane!(%Broadcast{} = msg) do
    {:socket_push, :text, Backend.ProtoMessage.encode_socket_message(%Message{
      topic: msg.topic,
      event: msg.event,
      payload: msg.payload
    })}
  end

  def encode!(%Reply{} = reply) do
    {:socket_push, :text, Backend.ProtoMessage.encode_socket_message(%Message{
      topic: reply.topic,
      event: "phx_reply",
      ref: reply.ref,
      payload: %{status: reply.status, response: reply.payload}
    })}
  end

  def encode!(%Message{} = msg) do
    {:socket_push, :text, Backend.ProtoMessage.encode_socket_message(msg)}
  end

  def decode!(message, _opts) do
    message
    |> Backend.ProtoMessage.ClientMessage.decode(client_message)
  end
end
