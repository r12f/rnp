$ErrorActionPreference = 'Stop'

$toolsDir   = "$(Split-Path -parent $MyInvocation.MyCommand.Definition)"
$installDir = $toolsDir
Write-Host "rnp is going to be installed in '$installDir'"

$packageArgs = @{
  PackageName    = $env:ChocolateyPackageName
  UnzipLocation  = $installDir
  Url64bit       = 'https://github.com/r12f/rnp/releases/download/{build_tag}/rnp.{build_tag}.windows.x64.zip'
  Checksum64     = '{package_zip_hash_x64}'
  ChecksumType64 = 'sha256'
  Url            = 'https://github.com/r12f/rnp/releases/download/{build_tag}/rnp.{build_tag}.windows.x86.zip'
  Checksum       = '{package_zip_hash_x86}'
  ChecksumType   = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs