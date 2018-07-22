defmodule BackendWeb.GameLoop do
  use GenServer

  @timedelay 5000

  def init(_state) do
    start_tick()
    {:ok, run_tick(-@timedelay)}
  end

  def start_link(default \\ "my game loop") do
    GenServer.start_link(__MODULE__, default)
  end

  def handle_info(:tick, state) do
    start_tick()
    {:noreply, run_tick(state)}
  end

  defp run_tick(state) do
    things = BackendWeb.GameState.list("game:first")
    IO.inspect(things)
    IO.puts("before: " <> to_string(state))
    state + @timedelay
  end

  defp start_tick() do
    Process.send_after(self(), :tick, @timedelay)
  end


end
