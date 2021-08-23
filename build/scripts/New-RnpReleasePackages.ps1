$githubReleasePackagesFolder = ".\Releases\GithubReleases"
$nugetPackageReleaseFolder = ".\Releases\NugetPackages"
$crateReleaseFolder = ".\Releases\Crate.io"
$homebrewReleaseFolder = ".\Releases\Homebrew"
$symbolStagingFolder = ".\Staging\Symbols"

function New-RnpReleasePackages
{
    Copy-RnpBuildOutputToRelease
    New-RnpMultiArchPackageWithFilePath
}

function Copy-RnpBuildOutputToRelease
{
    $flavors = @{
        "windows.x86"   = [PsCustomObject]@{
            "Root"    = "Build.Build.windowsx86";
            "Target"  = "i686-pc-windows-msvc";
        };
        "windows.x64"   = [PsCustomObject]@{
            "Root"    = "Build.Build.windowsx64";
            "Target"  = "x86_64-pc-windows-msvc";
        };
        "windows.arm64" = [PsCustomObject]@{
            "Root"    = "Build.Build.windowsarm64";
            "Target"  = "aarch64-pc-windows-msvc";
        };
        "linux.x86"     = [PsCustomObject]@{
            "Root"    = "Build.Build.linuxx86";
            "Target"  = "i686-unknown-linux-gnu";
        };
        "linux.x64"     = [PsCustomObject]@{
            "Root"    = "Build.Build.linuxx64";
            "Target"  = "x86_64-unknown-linux-gnu";
        };
        "linux.arm"     = [PsCustomObject]@{
            "Root"    = "Build.Build.linuxarm";
            "Target"  = "arm-unknown-linux-gnueabi";
        };
        "linux.arm64"   = [PsCustomObject]@{
            "Root"    = "Build.Build.linuxarm64";
            "Target"  = "aarch64-unknown-linux-gnu";
        };
        "macos.x64"     = [PsCustomObject]@{
            "Root"    = "Build.Build.macosx64";
            "Target"  = "x86_64-apple-darwin";
        };
    }

    $flavors.GetEnumerator() | ForEach-Object {
        $flavor = $_.Name

        $root = $_.Value.Root
        $target = $_.Value.Target
        Write-Host "Processing build: Flavor = $flavor, Root = $root, Target = $target"

        Copy-RnpBuildOutputToReleaseFolder "$root\symbols" "*" "$symbolStagingFolder\$flavor"
        Copy-RnpBuildOutputToReleaseFolder "$root\nuget" "*" $nugetPackageReleaseFolder
        Copy-RnpBuildOutputToReleaseFolder "$root\source" "rnp.crate.*" $crateReleaseFolder
        Copy-RnpBuildOutputToReleaseFolder "$root\source" "rnp.source.*" $githubReleasePackagesFolder
        Copy-RnpBuildOutputToReleaseFolder "$root\zipped" "*" $githubReleasePackagesFolder
        Copy-RnpBuildOutputToReleaseFolder "$root\nuget" "*" $githubReleasePackagesFolder
        Copy-RnpBuildOutputToReleaseFolder "$root\deb" "*" $githubReleasePackagesFolder
        Copy-RnpBuildOutputToReleaseFolder "$root\msix" "*" $githubReleasePackagesFolder
        Copy-RnpBuildOutputToReleaseFolder "$root\homebrew" "*" $homebrewReleaseFolder
    }
}

function Copy-RnpBuildOutputToReleaseFolder([string] $packageFolder, [string] $packageName, [string] $targetFolder)
{
    $packagePath = "$packageFolder\$packageName"

    if (-not (Test-Path $packageFolder -PathType Container)) {
        Write-Host "Build output folder is not found, skip copying files: $packagePath"
        return;
    }

    Write-Host "Build output folder is found, copying files with path: $packagePath"
    New-Item -ItemType Directory -Path $targetFolder -Force
    Copy-Item -Path $packagePath $targetFolder -Verbose -Force
}

function New-RnpMultiArchPackageWithFilePath
{
    $fileHashs = [pscustomobject]@{
        "ZipX86" = (Get-FileHash "$githubReleasePackagesFolder\rnp.*.windows.x86.zip" -Algorithm SHA256).Hash.ToLowerInvariant();
        "ZipX64" = (Get-FileHash "$githubReleasePackagesFolder\rnp.*.windows.x64.zip" -Algorithm SHA256).Hash.ToLowerInvariant();
        "BinX86" = (Get-FileHash ".\Build.Build.windowsx86\bin\rnp.exe" -Algorithm SHA256).Hash.ToLowerInvariant();
        "BinX64" = (Get-FileHash ".\Build.Build.windowsx64\bin\rnp.exe" -Algorithm SHA256).Hash.ToLowerInvariant();
        "SourceTar" = (Get-FileHash "$githubReleasePackagesFolder\rnp.source.*.tar.gz" -Algorithm SHA256).Hash.ToLowerInvariant();
    }
    Write-Host "File hash: $fileHashs"

    New-RnpChocolateyPackage
}

function New-RnpChocolateyPackage($fileHashs) {
    $chocoStagingFolder = ".\Staging\Chocolatey"
    $chocoReleaseFolder = ".\Releases\Chocolatey"
    Write-Host "Creating chocolatey package: ReleaseFolder = $chocoReleaseFolder, StagingFolder = $chocoStagingFolder"

    New-Item -ItemType Directory -Path $chocoReleaseFolder -Force
    New-Item -ItemType Directory -Path $chocoStagingFolder -Force
    New-Item -ItemType Directory -Path "$chocoStagingFolder\tools" -Force

    Expand-RnpPackageTemplateFileWithFileHash ".\Build.Build.windowsx64\choco\rnp.nuspec" "$chocoStagingFolder\rnp.nuspec" $fileHashs
    Expand-RnpPackageTemplateFileWithFileHash ".\Build.Build.windowsx64\choco\tools\chocolateyInstall.ps1" "$chocoStagingFolder\tools\chocolateyInstall.ps1" $fileHashs
    Expand-RnpPackageTemplateFileWithFileHash ".\Build.Build.windowsx64\choco\tools\VERIFICATION.txt" "$chocoStagingFolder\tools\VERIFICATION.txt" $fileHashs
    Expand-RnpPackageTemplateFileWithFileHash ".\Build.Build.windowsx64\choco\tools\LICENSE.txt" "$chocoStagingFolder\tools\LICENSE.txt" $fileHashs

    choco pack "$chocoStagingFolder\rnp.nuspec" --outputdirectory $chocoReleaseFolder
}

function Expand-RnpPackageTemplateFileWithFileHash($templateFile, $targetFile, $fileHashs) {
    $targetFileContent = Get-Content $templateFile
    $targetFileContent = $targetFileContent.Replace("{rnp_bin_hash_x86}", $fileHashs.BinX86).Replace("{rnp_bin_hash_x64}", $fileHashs.BinX64).Replace("{package_zip_hash_x86}", $fileHashs.ZipX86).Replace("{package_zip_hash_x64}", $fileHashs.ZipX64).Replace("{source_package_tar_hash}", $fileHashs.SourceTar);
    $utf8NoBom = New-Object System.Text.UTF8Encoding $False
    [System.IO.File]::WriteAllLines($targetFile, $targetFileContent, $utf8NoBom)
}

New-RnpReleasePackages