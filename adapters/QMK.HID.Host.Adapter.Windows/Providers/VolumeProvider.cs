namespace QMK.HID.Host.Adapter.Windows.Providers;

using QMK.HID.Host.Adapter.Windows.Providers.Helpers;

public class VolumeProvider : ProviderBase
{
    public override object GetValue() => (int)Math.Round(Audio.GetVolume() * 100);
}
