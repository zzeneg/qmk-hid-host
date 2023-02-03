namespace QMK.HID.Host.Adapter.Windows.Providers;

internal interface IProvider
{
    string Name { get; }

    Task Enable();

    Task Disable();

    object GetValue();

    event EventHandler<object>? ValueChanged;
}
