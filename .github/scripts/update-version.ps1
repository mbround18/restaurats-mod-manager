param(
  [Parameter(Mandatory = $true)]
  [string]$Version,
    
  [Parameter(Mandatory = $false)]
  [string]$CargoTomlPath = "Cargo.toml"
)

Write-Host "Original version: $Version"

# Parse and normalize version (strip leading zeros)
$segments = $Version -split '\.'
$trimmedSegments = @()
foreach ($segment in $segments) {
  if ($segment -match '^(\d+)-(.+)$') {
    $trimmedSegments += "$([int]$matches[1])-$($matches[2])"
  }
  else {
    $trimmedSegments += [int]$segment
  }
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
