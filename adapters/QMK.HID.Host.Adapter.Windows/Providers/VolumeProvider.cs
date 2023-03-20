namespace QMK.HID.Host.Adapter.Windows.Providers;

using QMK.HID.Host.Adapter.Windows.Providers.Helpers;

public class VolumeProvider : ProviderBase
{
    public override object GetValue() => (int)Math.Round(Audio.GetVolume() * 100);

    public override async Task Enable()
    {
        await base.Enable();

        OnValueChanged(GetValue());
    }
}
