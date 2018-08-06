defmodule NativePhysics do
  use Rustler, otp_app: :backend, crate: :native_physics

  def spawn_user(_), do: :erlang.nif_error(:nif_not_loaded)
  def tick(_), do: :erlang.nif_error(:nif_not_loaded)
end
