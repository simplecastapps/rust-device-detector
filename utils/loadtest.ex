Mix.install([
  {:httpoison, "~> 1.0"},
  {:jason, "~> 1.1"}
])

defmodule Load do
  def http_opts(opts \\ []) do
    timeout = opts[:timeout] || 30_000
    timeout = opts[:timeout] || 30_000
    [timeout: timeout, recv_timeout: timeout]
  end

  @doc """
  If you have a file with a list of user agents, you can use this function to
  rapidly load test.

  >  cargo run --release --features cache  --bin rust-device-detector -- -s -p 8085 -c 100&
  >  "my_file" |> Load.test(cases: 5, random: false, drop: 8000, verbose: true, port: 8085)

  * cases - how many cases to run (default: 20000)
  * drop - drop the first N lines (default: drop random number of them)
  * random - pick a random subset of lines from the file (default: true)
  * verbose - print out the input and output (default: false)
  * port - port to send the requests to (default: 8080)
  * verbose_input - print out the input user agents (default: verbose || false)
  * verbose_output - print out the output result (default: verbose || false)
  * max_concurrency - how many concurrent requests to send (default: 100)
  * timeout - how long to wait for each response (default: 30_000 (ms))
  """
  def test(file, opts \\ []) do
    max_concurrency = opts[:max_concurrency] || 100
    timeout = opts[:timeout] || 30_000

    File.stat(file)
    |> case do
      {:ok, _} -> :ok
      _ -> raise "File #{file} not found"
    end

    lines =
      opts[:lines] ||
        System.cmd("wc", ["-l", file])
        |> elem(0)
        |> String.split(" ")
        |> hd
        |> String.to_integer()

    cases = opts[:cases] || 20000

    random =
      case opts[:random] do
        true -> :rand.uniform(lines - cases)
        false -> false
      end

    drop =
      if is_integer(random) do
        random
      else
        opts[:drop] || 0
      end

    f =
      File.stream!(file, [:read])
      |> Stream.drop(drop)
      |> Stream.take(cases)

    IO.puts("Running #{cases} cases")

    bench(
      fn ->
        f
        |> Task.async_stream(&detect(&1, opts),
          max_concurrency: max_concurrency,
          timeout: timeout,
          ordered: false
        )
        |> Stream.run()
      end,
      cases
    )
  end

  def detect(str, opts \\ []) do
    verbose = opts[:verbose] || false
    verbose_input = opts[:verbose] || opts[:verbose_input] || false
    verbose_output = opts[:verbose] || opts[:verbose_output] || false
    port = opts[:port] || 8080

    if verbose_input do
      "useragent '#{str}'" |> IO.puts()
    end

    HTTPoison.post("http://localhost:#{port}/detect", str, [], http_opts(opts))
    |> then(fn
      {:ok, %HTTPoison.Response{status_code: 200, body: b}} ->
        b
        |> Jason.decode!(pretty: false)
        |> then(fn x ->
          if verbose_output do
            %{useragent: str, result: x} |> IO.inspect()
          end
        end)
    end)
  end

  def bench(function, amount) do
    {time, value} =
      function
      |> :timer.tc()

    rate = amount / (time / 1_000_000)

    IO.puts("Function ran in #{time / 1_000_000} seconds (#{rate} / sec)")

    value
  end
end
