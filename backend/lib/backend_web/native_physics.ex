defmodule NativePhysics do
  use Rustler, otp_app: :backend, crate: :native_physics

  def spawn_user(_), do: :erlang.nif_error(:nif_not_loaded)
  def tick(_, _), do: :erlang.nif_error(:nif_not_loaded)
  def get_snapshot(), do: :erlang.nif_error(:nif_not_loaded)
end
