$ErrorActionPreference = 'Stop'

$toolsDir   = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$installDir = $toolsDir
Write-Host "rnp is going to be installed in '$installDir'"

if (Get-OSArchitectureWidth 64) {
    $packageArgs = @{
      packageName    = $env:ChocolateyPackageName
      unzipLocation  = $installDir
      url            = 'https://github.com/r12f/rnp/releases/download/{build_tag}/rnp.{build_tag}.windows.x64.zip'
      checksum       = '{package_zip_hash_x64}'
      checksumType   = 'sha256'
    }
} else {
    $packageArgs = @{
      packageName    = $env:ChocolateyPackageName
      unzipLocation  = $installDir
      url            = 'https://github.com/r12f/rnp/releases/download/{build_tag}/rnp.{build_tag}.windows.x86.zip'
      checksum       = '{package_zip_hash_x86}'
      checksumType   = 'sha256'
    }
}

Install-ChocolateyZipPackage @packageArgs