namespace QMK.HID.Host.Adapter.Windows.Providers;

using Serilog;

public abstract class ProviderBase : IProvider
{
    private object? _lastValueChanged = null;
    private bool _isEnabled;

    protected ProviderBase()
    {
        Name = GetType().Name.Replace("Provider", string.Empty);
    }

    public string Name { get; }

    public virtual Task Enable()
    {
        _isEnabled = true;
        Log.Information("{Name} started", Name);
        return Task.CompletedTask;
    }

    public virtual Task Disable()
    {
        _isEnabled = false;
        _lastValueChanged = null;
        Log.Information("{Name} stopped", Name);
        return Task.CompletedTask;
    }

    public abstract object GetValue();

    public event EventHandler<object>? ValueChanged;

    protected void OnValueChanged(object value)
    {
        if ((_lastValueChanged == null || !_lastValueChanged.Equals(value)) && _isEnabled)
        {
            _lastValueChanged = value;
            ValueChanged?.Invoke(this, value);
        }
    }
}
