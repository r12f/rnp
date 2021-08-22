function Install-CargoMakeBinary
{
    $cargoMakeUrl = "https://github.com/sagiegurari/cargo-make/releases/download/0.35.0/cargo-make-v0.35.0-x86_64-pc-windows-msvc.zip"
    $cargoMakeLocalPackage = "${env:TEMP}\cargo-make.zip"

    Write-Host "Downloading cargo-make windows binary package from $cargoMakeUrl to $cargoMakeLocalPackage"
    Invoke-WebRequest $cargoMakeUrl -OutFile $cargoMakeLocalPackage

    $cargoMakeTempDir = "${env:TEMP}\cargo-make"
    Write-Host "Unzip cargo-make package $cargoMakeLocalPackage to $cargoMakeTempDir"
    Expand-Archive -Path $cargoMakeLocalPackage -DestinationPath $cargoMakeTempDir -Force

    $cargoBinaryDir = "${env:USERPROFILE}\.cargo\bin\"
    Write-Host "Copy ${cargoMakeTempDir}\cargo-make.exe to cargo binary dir: $cargoBinaryDir"
    Copy-Item -Path "${cargoMakeTempDir}\cargo-make.exe" $cargoBinaryDir -PassThru -Force
}

Install-CargoMakeBinary
