namespace QMK.HID.Host.Adapter.Windows;

public class AppSettings
{
    public SocketIOSettings? SocketIO { get; set; }
}

public class SocketIOSettings
{
    public string? Host { get; set; }
    public TimeSpan ReconnectDelay { get; set; }
}
