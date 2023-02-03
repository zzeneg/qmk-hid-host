namespace QMK.HID.Host.Adapter.Windows.Providers.Helpers;

using System.Runtime.InteropServices;

internal static class Layout
{
    internal static int GetKeyboardLayoutId()
    {
        var focusedHWnd = GetForegroundWindow();
        var activeThread = GetWindowThreadProcessId(focusedHWnd, IntPtr.Zero);
        var keyboardLayoutId = (int)GetKeyboardLayout(activeThread);
        return keyboardLayoutId & 0xFFFF;
    }

    [DllImport("user32.dll")]
    private static extern IntPtr GetForegroundWindow();

    [DllImport("user32.dll")]
    private static extern uint GetWindowThreadProcessId(IntPtr hWnd, IntPtr processId);

    [DllImport("user32.dll")]
    private static extern IntPtr GetKeyboardLayout(uint idThread);
}
