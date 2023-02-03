namespace QMK.HID.Host.Adapter.Windows;

using System.Text;
using Microsoft.Extensions.Configuration;
using Serilog;

public static class Program
{
    public static async Task Main()
    {
        Console.OutputEncoding = Encoding.UTF8;

        ConfigureLogging();

        var settings = GetSettings();

        var client = new HidHostClient(settings.SocketIO);
        await client.Start();

        Console.ReadLine();
    }

    private static AppSettings GetSettings()
    {
        var builder = new ConfigurationBuilder()
            .AddJsonFile("appsettings.json", optional: true, reloadOnChange: true)
            .AddJsonFile(AppDomain.CurrentDomain.FriendlyName + ".json", optional: true, reloadOnChange: true);

        var config = builder.Build();

        var settings = config.Get<AppSettings>();
        if (settings == null)
        {
            throw new Exception("AppSettings are not provided");
        }

        return settings;
    }

    private static void ConfigureLogging() =>
        Log.Logger = new LoggerConfiguration()
            .WriteTo.Console()
            .WriteTo.File(AppDomain.CurrentDomain.FriendlyName + ".txt")
            .CreateLogger();
}
