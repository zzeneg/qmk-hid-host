namespace QMK.HID.Host.Adapter.Windows.Providers;

using global::Windows.Media.Control;

public abstract class MediaProvider : ProviderBase
{
    protected GlobalSystemMediaTransportControlsSessionMediaProperties? Properties;
    private GlobalSystemMediaTransportControlsSessionManager? _manager;
    private GlobalSystemMediaTransportControlsSession? _session;

    public override async Task Enable()
    {
        _manager = await GlobalSystemMediaTransportControlsSessionManager.RequestAsync();
        _manager.CurrentSessionChanged += OnCurrentSessionChanged;
        OnCurrentSessionChanged(_manager, null);
        await base.Enable();
    }

    public override Task Disable()
    {
        if (_manager != null)
        {
            _manager.CurrentSessionChanged -= OnCurrentSessionChanged;
        }

        if (_session != null)
        {
            _session.MediaPropertiesChanged -= SessionOnMediaPropertiesChanged;
        }

        return base.Disable();
    }

    private void OnCurrentSessionChanged(GlobalSystemMediaTransportControlsSessionManager manager, CurrentSessionChangedEventArgs? args)
    {
        _session = manager.GetCurrentSession();
        if (_session == null)
        {
            OnValueChanged(string.Empty);
        }
        else
        {
            _session.MediaPropertiesChanged += SessionOnMediaPropertiesChanged;
            SessionOnMediaPropertiesChanged(_session, null);
        }
    }

    private async void SessionOnMediaPropertiesChanged(GlobalSystemMediaTransportControlsSession session, MediaPropertiesChangedEventArgs? args)
    {
        Properties = await session.TryGetMediaPropertiesAsync();
        OnValueChanged(GetValue());
    }
}

public class MediaArtistProvider : MediaProvider
{
    public override object GetValue() => Properties?.Artist ?? string.Empty;
}

public class MediaTitleProvider : MediaProvider
{
    public override object GetValue() => Properties?.Title ?? string.Empty;
}
