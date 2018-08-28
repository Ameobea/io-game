defmodule NativePhysics do
  use Rustler, otp_app: :backend, crate: :native_physics

  def spawn_user(_), do: :erlang.nif_error(:nif_not_loaded)
  def tick(_, _, _), do: :erlang.nif_error(:nif_not_loaded)
  def get_snapshot(), do: :erlang.nif_error(:nif_not_loaded)

  defmodule UserDiff do
    defstruct id: UUID.uuid4(), action_type: :noop, payload: {}

    def new({id, action_type, payload}) do
      %UserDiff{id: id, action_type: action_type, payload: payload}
    end
  end

  defmodule MovementUpdate do
    defstruct pos_x: 0.0, pos_y: 0.0, rotation: 0.0, velocity_x: 0.0, velocity_y: 0.0, angular_velocity: 0.0
  end

  defmodule Update do
    defstruct id: nil, payload: nil, update_type: nil
  end
end
