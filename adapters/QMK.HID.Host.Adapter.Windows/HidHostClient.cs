namespace QMK.HID.Host.Adapter.Windows;

using QMK.HID.Host.Adapter.Windows.Providers;
using Serilog;
using SocketIOClient;

public class HidHostClient
{
    private static readonly IEnumerable<IProvider> Providers = new List<IProvider>
    {
        new VolumeProvider(),
        new LayoutProvider(),
        new MediaArtistProvider(),
        new MediaTitleProvider(),
    };

    private readonly SocketIOSettings _socketIOSettings;
    private readonly SocketIO _client;

    public HidHostClient(SocketIOSettings? socketIOSettings)
    {
        _socketIOSettings = socketIOSettings ?? throw new ArgumentNullException(nameof(socketIOSettings), "SocketIO settings are not defined");
        _client = new SocketIO(socketIOSettings.Host);

        ConfigureClient();
    }

    public Task Start() => ConnectWithRetry();

    private void ConfigureClient()
    {
        foreach (var provider in Providers)
        {
            RegisterProviderOnSocketIo(provider);
        }

        _client.OnConnected += (_, _) =>
        {
            Log.Information("Connected to server");
        };

        _client.OnDisconnected += async (_, _) =>
        {
            Log.Information("Disconnected from server");

            await Task.WhenAll(Providers.Select(x => x.Disable()));
            await ConnectWithRetry();
        };

        _client.OnError += (_, err) => Log.Error("Server error: {Error}", err);

        _client.On("hid-connected", async _ =>
        {
            Log.Information("Keyboard connected");
            await Task.WhenAll(Providers.Select(x => x.Enable()));
        });

        _client.On("hid-disconnected", async _ =>
        {
            Log.Information("Keyboard disconnected");
            await Task.WhenAll(Providers.Select(x => x.Disable()));
        });
    }

    private async Task ConnectWithRetry()
    {
        try
        {
            Log.Information("Trying to connect to server...");
            await _client.ConnectAsync();
        }
        catch (ConnectionException)
        {
            Log.Information("Connection failed, waiting {ReconnectDelayTotalSeconds} seconds before retrying...",
                _socketIOSettings.ReconnectDelay.TotalSeconds);
            await Task.Delay(_socketIOSettings.ReconnectDelay);
            await ConnectWithRetry();
        }
    }

    private void RegisterProviderOnSocketIo(IProvider provider)
    {
        Log.Information("Registering {ProviderName}...", provider.Name);

        provider.ValueChanged += async (_, value) =>
        {
            Log.Information("{ProviderName} sends updated value {Value}", provider.Name, value);
            if (_client.Connected)
            {
                await _client.EmitAsync(provider.Name, value);
            }
        };

        _client.On(provider.Name, async _ =>
        {
            Log.Information("Server requested {ProviderName}", provider.Name);
            var value = provider.GetValue();
            Log.Information("{ProviderName} responds with value {Value}", provider.Name, value);
            await _client.EmitAsync(provider.Name, value);
        });
    }
}
