namespace QMK.HID.Host.Adapter.Windows.Providers;

using System.Globalization;
using QMK.HID.Host.Adapter.Windows.Providers.Helpers;

public class LayoutProvider : ProviderBase
{
    private PeriodicTimer? _timer;

    public override object GetValue() => CultureInfo.GetCultureInfo(Layout.GetKeyboardLayoutId()).TwoLetterISOLanguageName;

    public override async Task Enable()
    {
        _timer = new PeriodicTimer(TimeSpan.FromSeconds(1));
        var _ = Task.Run(async () =>
        {
            while (await _timer.WaitForNextTickAsync())
            {
                OnValueChanged(GetValue());
            }
        });

        await base.Enable();
    }

    public override Task Disable()
    {
        _timer?.Dispose();
        return base.Disable();
    }
}
