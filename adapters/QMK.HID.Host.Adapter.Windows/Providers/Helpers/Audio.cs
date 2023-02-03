namespace QMK.HID.Host.Adapter.Windows.Providers.Helpers;

using System.Runtime.InteropServices;

internal static class Audio
{
    internal static float GetVolume() =>
        UseVolumeObject(volumeObject =>
        {
            if (volumeObject == null)
            {
                return -1;
            }

            Marshal.ThrowExceptionForHR(volumeObject.GetMasterVolumeLevelScalar(out var volumeLevel));
            return volumeLevel;
        });

    internal static bool IsMute() =>
        UseVolumeObject(volumeObject =>
        {
            if (volumeObject == null)
            {
                return false;
            }

            Marshal.ThrowExceptionForHR(volumeObject.GetMute(out var isMute));
            return isMute;
        });

    private static T UseVolumeObject<T>(Func<IAudioEndpointVolume?, T> action)
    {
        IAudioEndpointVolume? volumeObject = null;
        try
        {
            volumeObject = GetObject();
            return action(volumeObject);
        }
        finally
        {
            if (volumeObject != null)
            {
                Marshal.ReleaseComObject(volumeObject);
            }
        }
    }

    private static IAudioEndpointVolume? GetObject()
    {
        IMmDeviceEnumerator? deviceEnumerator = null;
        IMmDevice? speakers = null;
        try
        {
            deviceEnumerator = new MMDeviceEnumerator() as IMmDeviceEnumerator;

            Marshal.ThrowExceptionForHR(deviceEnumerator.GetDefaultAudioEndpoint(dataFlow: 0, role: 1, endpoint: out speakers));
            var id = typeof(IAudioEndpointVolume).GUID;

            Marshal.ThrowExceptionForHR(speakers.Activate(ref id, clsCtx: 23, activationParams: 0, out var epv));
            return epv;
        }
        finally
        {
            if (speakers != null)
            {
                Marshal.ReleaseComObject(speakers);
            }

            if (deviceEnumerator != null)
            {
                Marshal.ReleaseComObject(deviceEnumerator);
            }
        }
    }
}

[Guid("5CDF2C82-841E-4546-9722-0CF74078229A")]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
internal interface IAudioEndpointVolume
{
    int F();
    int G();
    int H();
    int I();
    int J();
    int K();
    int GetMasterVolumeLevelScalar(out float pfLevel);
    int L();
    int M();
    int N();
    int O();
    int P();
    int GetMute(out bool pbMute);
}

[Guid("D666063F-1587-4E43-81F1-B948E807363F")]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
internal interface IMmDevice
{
    int Activate(ref Guid id, int clsCtx, int activationParams, out IAudioEndpointVolume? aev);
}

[Guid("A95664D2-9614-4F35-A746-DE8DB63617E6")]
[InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
internal interface IMmDeviceEnumerator
{
    int F();
    int GetDefaultAudioEndpoint(int dataFlow, int role, out IMmDevice? endpoint);
}

[ComImport]
[Guid("BCDE0395-E52F-467C-8E3D-C4579291692E")]
internal class MMDeviceEnumerator
{
}


