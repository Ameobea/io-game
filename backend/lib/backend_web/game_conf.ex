defmodule BackendWeb.GameConf do
  use Agent

  @config_files ["game", "network", "physics"]
  @config_dir "../config"

  def start_link() do
    Agent.start_link(fn -> {false, %{}} end, name: __MODULE__)
  end

  defp read_config(filename) do
    path = Path.join(@config_dir, filename <> ".json")
    contents = File.read!(path)
    Jason.decode!(contents)
  end

  defp load_config(filename, acc) do
    Map.put(acc, filename, read_config(filename))
  end

  defp load_config() do
    loaded_config = List.foldl(@config_files, %{}, &load_config/2)
    Agent.update(__MODULE__, fn _ -> {true, loaded_config} end)
  end

  defp config_loaded?() do
    Agent.get(__MODULE__, fn {loaded, _} -> loaded end)
  end

  def get_config(category) do
    if !config_loaded?(), do: load_config()

    Agent.get(__MODULE__, fn {_, conf} -> Map.get(conf, category) end)
  end

  def get_config(category, key) do
    if !config_loaded?(), do: load_config()

    Agent.get(__MODULE__, fn {_, conf} -> Map.get(conf, category) |> Map.get(key) end)
  end
end
