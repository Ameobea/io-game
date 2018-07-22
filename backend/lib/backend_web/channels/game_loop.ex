defmodule BackendWeb.GameLoop do
  use GenServer

  @timedelay 1000

  def init(_state) do
    IO.puts("please")
    start_tick()
    {:ok, 0}
  end

  def start_link(default \\ "my game loop") do
    GenServer.start_link(__MODULE__, default)
  end

  def handle_info(:tick, state) do
    IO.puts("before: " <> to_string(state))
    start_tick()
    {:noreply, state + @timedelay}
  end

  defp start_tick() do
    Process.send_after(self(), :tick, @timedelay)
  end


end
