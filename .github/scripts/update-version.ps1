param(
  [Parameter(Mandatory = $true)]
  [string]$Version,
    
  [Parameter(Mandatory = $false)]
  [string]$CargoTomlPath = "Cargo.toml"
)

Write-Host "Original version: $Version"

# Parse and normalize version (strip leading zeros and century prefix)
$segments = $Version -split '\.'
$trimmedSegments = @()
$isFirst = $true
foreach ($segment in $segments) {
  if ($segment -match '^(\d+)-(.+)$') {
    # Segment with suffix (e.g., "13-1")
    $num = [int]$matches[1]
    $suffix = $matches[2]
    # Strip "20" prefix from first segment if it's a year (e.g., 2026 -> 26)
    if ($isFirst -and $num -ge 2000 -and $num -lt 2100) {
      $num = $num - 2000
    }
    $trimmedSegments += "$num-$suffix"
  }
  else {
    # Plain numeric segment
    $num = [int]$segment
    # Strip "20" prefix from first segment if it's a year (e.g., 2026 -> 26)
    if ($isFirst -and $num -ge 2000 -and $num -lt 2100) {
      $num = $num - 2000
    }
    $trimmedSegments += $num
  }
  $isFirst = $false
}
$cargoVersion = $trimmedSegments -join '.'
Write-Host "Normalized version for Cargo.toml: $cargoVersion"

# Read and parse Cargo.toml line by line
$lines = Get-Content $CargoTomlPath
$inPackageSection = $false
$updated = $false

for ($i = 0; $i -lt $lines.Count; $i++) {
  $line = $lines[$i]
    
  # Track when we enter [package] section
  if ($line -match '^\[package\]') {
    $inPackageSection = $true
    continue
  }
    
  # Track when we leave [package] section (enter another section)
  if ($line -match '^\[.*\]' -and $inPackageSection) {
    $inPackageSection = $false
  }
    
  # Update version only in [package] section
  if ($inPackageSection -and $line -match '^version\s*=\s*"[^"]*"') {
    $lines[$i] = "version = `"$cargoVersion`""
    Write-Host "Updated line $($i + 1): $($lines[$i])"
    $updated = $true
    break
  }
}

if (-not $updated) {
  Write-Error "Failed to find version field in [package] section"
  exit 1
}

# Write back
$lines | Set-Content $CargoTomlPath -NoNewline:$false

# Verify the change
Write-Host "`nUpdated [package] section:"
Get-Content $CargoTomlPath | Select-String -Pattern "^\[package\]" -Context 0, 10

Write-Host "`nSuccessfully updated package version to $cargoVersion"

# Output normalized version to GitHub Actions if running in CI
if ($env:GITHUB_OUTPUT) {
  "wix_version=$cargoVersion" | Out-File -FilePath $env:GITHUB_OUTPUT -Encoding utf8 -Append
  Write-Host "Set GITHUB_OUTPUT wix_version=$cargoVersion"
}
