#requires -Version 5.0

[CmdletBinding()]
param()

$ErrorActionPreference = 'Stop'

function Stop-PuzzleProcess {
    param([System.Diagnostics.Process]$Process)

    if ($Process.HasExited) {
        return
    }

    if ($PSVersionTable.PSVersion.Major -ge 7) {
        $Process.Kill($true)
    }
    else {
        $processes = @(Get-CimInstance -ClassName Win32_Process -Property ProcessId, ParentProcessId)
        $processIds = @($Process.Id)

        for ($index = 0; $index -lt $processIds.Count; $index++) {
            $parentId = $processIds[$index]
            $processIds += @(
                $processes | Where-Object {
                    $_.ParentProcessId -eq $parentId -and $_.ProcessId -notin $processIds
                } | Select-Object -ExpandProperty ProcessId
            )
        }

        [array]::Reverse($processIds)
        foreach ($processId in $processIds) {
            try {
                Stop-Process -Id $processId -Force -ErrorAction Stop
            }
            catch {
                if ($_.CategoryInfo.Category -ne [System.Management.Automation.ErrorCategory]::ObjectNotFound) {
                    throw
                }
            }
        }
    }

    $Process.WaitForExit()
}

$puzzleIds = @(
    '3aa6fb7a'
    '0ca9ddb6'
    'e9afcf9a'
    '6fa7a44f'
    'd631b094'
)

$bun = (Get-Command bun -CommandType Application -ErrorAction Stop).Source
$batchId = '{0}-{1}' -f (Get-Date -Format 'yyyyMMdd-HHmmss'), [guid]::NewGuid().ToString('N')
$logDirectory = Join-Path $PSScriptRoot "logs\batch-$batchId"
New-Item -ItemType Directory -Path $logDirectory | Out-Null

$runs = @()

try {
    foreach ($puzzleId in $puzzleIds) {
        $stdout = Join-Path $logDirectory "$puzzleId.stdout.log"
        $stderr = Join-Path $logDirectory "$puzzleId.stderr.log"
        $run = [pscustomobject]@{
            Puzzle = $puzzleId
            Result = 'NOT RUN'
            ExitCode = $null
            Seconds = $null
            StartedAt = [datetime]::UtcNow
            Process = $null
        }
        $runs += $run

        try {
            $run.Process = Start-Process -FilePath $bun `
                -ArgumentList @('run', 'start', $puzzleId) `
                -WorkingDirectory $PSScriptRoot -NoNewWindow -PassThru `
                -RedirectStandardOutput $stdout -RedirectStandardError $stderr
            # Retain the handle for processes that finish before collection.
            $null = $run.Process.Handle
            Write-Host "Started $puzzleId (PID $($run.Process.Id))"
        }
        catch {
            $_ | Out-String | Set-Content -LiteralPath $stderr
            Write-Warning "Could not start $puzzleId. See $stderr"
        }
    }

    foreach ($run in $runs) {
        if ($null -eq $run.Process) {
            continue
        }

        $run.Process.WaitForExit()
        $run.ExitCode = $run.Process.ExitCode
        $run.Result = if ($run.ExitCode -eq 0) { 'PASS' } else { 'FAIL' }
        $run.Seconds = [math]::Round(
            ($run.Process.ExitTime.ToUniversalTime() - $run.StartedAt).TotalSeconds,
            1
        )
    }
}
finally {
    foreach ($run in $runs) {
        if ($null -eq $run.Process) {
            continue
        }

        try {
            Stop-PuzzleProcess -Process $run.Process
        }
        catch {
            Write-Warning "Could not stop $($run.Puzzle): $($_.Exception.Message)"
        }
        finally {
            $run.Process.Dispose()
        }
    }
}

Write-Host "`nLogs: $logDirectory"
$runs | Format-Table Puzzle, Result, ExitCode, Seconds -AutoSize | Out-Host

if (@($runs | Where-Object { $_.Result -ne 'PASS' }).Count -gt 0) {
    exit 1
}

exit 0
