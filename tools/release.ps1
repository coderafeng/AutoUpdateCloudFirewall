# Build a release binary with the platform-suffixed asset name, and optionally
# publish it together with the multilingual release notes.
#
# The canonical release path is `.github/workflows/release.yml`: pushing a `v*`
# tag builds all three platforms in CI and attaches them to the release. This
# script is the local escape hatch — it builds the platform it runs on.
#
#   pwsh tools/release.ps1 -Version 1.0.0
#   pwsh tools/release.ps1 -Version 1.0.0 -Publish
#
# Asset naming follows the platform binary convention:
#   dist/auto-update-cloud-firewall-x86_64-windows.exe
#   dist/auto-update-cloud-firewall-x86_64-darwin
#   dist/auto-update-cloud-firewall-x86_64-linux
#
# Each platform has to be built on its own machine (or in CI): the script only
# builds the platform it runs on unless -Platform names another installed
# rustup target.

param(
    # Defaults to the version in Cargo.toml.
    [string]$Version = "",
    # windows | darwin | linux. Defaults to the current host.
    [ValidateSet("windows", "darwin", "linux")]
    [string]$Platform = "",
    # Asset file name, without the platform suffix.
    [string]$AssetStem = "auto-update-cloud-firewall",
    # Release notes handed to `gh release create`.
    [string]$Notes = "",
    [string]$Title = "",
    # Skip `cargo fmt --check`, clippy and the test suite.
    [switch]$SkipChecks,
    # Run `gh release create` / `gh release upload` instead of printing the command.
    [switch]$Publish
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

if (-not $Version) {
    $line = Select-String -Path "Cargo.toml" -Pattern '^version = "(.+)"$' | Select-Object -First 1
    if (-not $line) { throw "cannot read the version from Cargo.toml" }
    $Version = $line.Matches[0].Groups[1].Value
}

if (-not $Platform) {
    if ($env:OS -eq "Windows_NT") {
        $Platform = "windows"
    } elseif ((Test-Path variable:IsMacOS) -and $IsMacOS) {
        $Platform = "darwin"
    } else {
        $Platform = "linux"
    }
}

switch ($Platform) {
    "windows" { $triple = "x86_64-pc-windows-msvc"; $extension = ".exe" }
    "darwin" { $triple = "x86_64-apple-darwin"; $extension = "" }
    "linux" { $triple = "x86_64-unknown-linux-gnu"; $extension = "" }
}

$asset = "$AssetStem-x86_64-$Platform$extension"
$notesPath = if ($Notes) { $Notes } else { "docs/releases/$Version.md" }
if (-not (Test-Path $notesPath)) {
    throw "release notes not found: $notesPath"
}
if (-not $Title) { $Title = $Version }

Write-Host "version  : $Version"
Write-Host "platform : $Platform ($triple)"
Write-Host "asset    : dist/$asset"
Write-Host "notes    : $notesPath"
Write-Host ""

if (-not $SkipChecks) {
    Write-Host "==> cargo fmt --check"
    cargo fmt --all --check
    if ($LASTEXITCODE -ne 0) { throw "formatting is not clean" }

    Write-Host "==> cargo clippy"
    cargo clippy --all-targets -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw "clippy reported problems" }

    Write-Host "==> cargo test"
    cargo test
    if ($LASTEXITCODE -ne 0) { throw "tests failed" }
}

Write-Host "==> cargo build --release --target $triple"
cargo build --release --target $triple
if ($LASTEXITCODE -ne 0) {
    throw "build failed; run 'rustup target add $triple' if the target is missing"
}

$built = "target/$triple/release/$AssetStem$extension"
if (-not (Test-Path $built)) { throw "build output not found: $built" }

New-Item -ItemType Directory -Force -Path "dist" | Out-Null
Copy-Item $built "dist/$asset" -Force

# Linux also ships a tarball: it preserves the executable bit and carries the
# license and readme next to the binary.
if ($Platform -eq "linux") {
    $pack = "target/pack/$asset"
    New-Item -ItemType Directory -Force -Path $pack | Out-Null
    Copy-Item "dist/$asset", "LICENSE", "README.md" $pack -Force
    tar -czf "dist/$asset.tar.gz" -C (Split-Path -Parent $pack) $asset
    if ($LASTEXITCODE -ne 0) { throw "failed to create dist/$asset.tar.gz" }
    Write-Host "==> wrote dist/$asset.tar.gz"
}

$sizeMb = [math]::Round((Get-Item "dist/$asset").Length / 1MB, 1)
Write-Host "==> wrote dist/$asset ($sizeMb MB)"
Write-Host ""

# Every asset that is already present in dist/ is uploaded together, so the
# Windows build does not need to know about the other two.
$assets = @(Get-ChildItem "dist" -Filter "$AssetStem-x86_64-*" -File | ForEach-Object { $_.FullName })
$tag = "v$Version"

Write-Host "assets to upload:"
$assets | ForEach-Object { Write-Host "  $_" }
Write-Host ""

if (-not $Publish) {
    Write-Host "to publish:"
    Write-Host "  gh release create $tag $($assets -join ' ') --title `"$Title`" --notes-file $notesPath"
    Write-Host ""
    Write-Host "or, when the release already exists:"
    Write-Host "  gh release upload $tag $($assets -join ' ') --clobber"
    return
}

# `gh release view` exits non-zero when the release does not exist yet; keep that
# from aborting the script when native-command error handling is enabled.
$nativePreference = $ErrorActionPreference
$ErrorActionPreference = "Continue"
gh release view $tag *> $null
$releaseExists = ($LASTEXITCODE -eq 0)
$ErrorActionPreference = $nativePreference

if ($releaseExists) {
    Write-Host "==> release $tag exists, uploading $($assets.Count) asset(s)"
    gh release upload $tag @assets --clobber
} else {
    Write-Host "==> creating release $tag"
    gh release create $tag @assets --title $Title --notes-file $notesPath
}
if ($LASTEXITCODE -ne 0) { throw "gh failed" }
