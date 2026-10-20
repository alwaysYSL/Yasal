param(
    [Parameter(Mandatory = $true)]
    [string] $Executable,
    [switch] $OpenClose,
    [double] $MaxPrivateMB = 0,
    [double] $MaxWorkingSetMB = 0
)

$ErrorActionPreference = 'Stop'
$executablePath = (Resolve-Path -LiteralPath $Executable).Path
$runDirectory = Join-Path (Split-Path -Parent $PSScriptRoot) 'target/cpu-idle-test'
New-Item -ItemType Directory -Path $runDirectory -Force | Out-Null
$logPath = Join-Path $runDirectory 'yasal_debug.log'
Remove-Item -LiteralPath $logPath -ErrorAction SilentlyContinue
# Yasal writes its log to %LOCALAPPDATA%\Yasal by default; isolate this run.
$env:YASAL_LOG_DIR = $runDirectory

if ($OpenClose) {
    Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class YasalIdleTestMessages {
    [DllImport("user32.dll")]
    public static extern bool PostThreadMessageW(uint threadId, uint message, UIntPtr wParam, IntPtr lParam);
}
'@
    function Send-HotkeyMessage {
        $current = Get-Process -Id $process.Id -ErrorAction Stop
        foreach ($thread in $current.Threads) {
            # WM_HOTKEY to this process only. The listener's GetMessage loop
            # consumes it exactly as it consumes a registered hotkey event.
            [void][YasalIdleTestMessages]::PostThreadMessageW(
                [uint32]$thread.Id, 0x0312, [UIntPtr]::Zero, [IntPtr]::Zero)
        }
    }
}

$process = Start-Process -FilePath $executablePath -WorkingDirectory $runDirectory -WindowStyle Hidden -PassThru
try {
    if ($OpenClose) {
        Start-Sleep -Seconds 2
        Send-HotkeyMessage
        Start-Sleep -Seconds 1
        Send-HotkeyMessage
        Start-Sleep -Seconds 1
        $log = Get-Content -LiteralPath $logPath -Raw
        if ($log -notmatch 'toggle_visibility: is_visible=true' -or $log -notmatch 'toggle_visibility: is_visible=false') {
            throw 'Global hotkey did not open and close Yasal; idle measurement would be invalid.'
        }
    }

    # The reported regression starts when the hidden window's 10-second
    # repaint timer expires, so measure only after that point.
    Start-Sleep -Seconds 12
    $start = Get-Process -Id $process.Id -ErrorAction Stop
    $startCpu = $start.TotalProcessorTime.TotalSeconds
    $watch = [System.Diagnostics.Stopwatch]::StartNew()
    Start-Sleep -Seconds 4
    $end = Get-Process -Id $process.Id -ErrorAction Stop
    $watch.Stop()

    $cpuCores = ($end.TotalProcessorTime.TotalSeconds - $startCpu) / $watch.Elapsed.TotalSeconds
    $cpuPercent = 100 * $cpuCores / [Environment]::ProcessorCount
    Write-Output ('Hidden idle: {0:N3} CPU cores ({1:N2}% of {2} logical processors)' -f $cpuCores, $cpuPercent, [Environment]::ProcessorCount)
    $privateMB = $end.PrivateMemorySize64 / 1MB
    $workingSetMB = $end.WorkingSet64 / 1MB
    Write-Output ('Hidden idle memory: {0:N2} MB private, {1:N2} MB working set' -f $privateMB, $workingSetMB)

    if ($cpuCores -ge 0.10) {
        throw 'Hidden Yasal consumed at least 0.10 CPU cores after startup.'
    }
    if ($MaxPrivateMB -gt 0 -and $privateMB -gt $MaxPrivateMB) {
        throw ('Hidden Yasal private memory exceeded {0:N2} MB.' -f $MaxPrivateMB)
    }
    if ($MaxWorkingSetMB -gt 0 -and $workingSetMB -gt $MaxWorkingSetMB) {
        throw ('Hidden Yasal working set exceeded {0:N2} MB.' -f $MaxWorkingSetMB)
    }

    if ($OpenClose) {
        Send-HotkeyMessage
        Start-Sleep -Seconds 1
        $log = Get-Content -LiteralPath $logPath -Raw
        $openedCount = ([regex]::Matches($log, 'toggle_visibility: is_visible=true')).Count
        if ($openedCount -lt 2) {
            throw 'Yasal did not reopen after a hidden idle period.'
        }
    }
}
finally {
    if (Get-Process -Id $process.Id -ErrorAction SilentlyContinue) {
        Stop-Process -Id $process.Id -Force
    }
}
