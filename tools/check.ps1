$ErrorActionPreference = "Stop"
$RepoRoot = Split-Path -Parent $PSScriptRoot

$Git = Get-Command git.exe -ErrorAction Stop
$GitRoot = Split-Path -Parent (Split-Path -Parent $Git.Source)
$Bash = Join-Path $GitRoot "bin\bash.exe"
if (-not (Test-Path $Bash -PathType Leaf)) {
    throw "Git Bash was not found beside $($Git.Source)"
}

Push-Location $RepoRoot
try {
    & $Bash "scripts/ci.sh" @args
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
} finally {
    Pop-Location
}
