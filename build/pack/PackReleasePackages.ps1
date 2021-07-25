function PackAllReleasePackages() {
    PackPerFlavorReleases
    PackSymbolsPackagesForGithubRelease
    PackRustCrate
    PackChocolateyPackages
}

function PackPerFlavorReleases() {
    New-Item -ItemType Directory -Path ".\Releases\GithubReleases"
    New-Item -ItemType Directory -Path ".\Releases\NugetPackages"

    $flavors = @{
        "windows.x86"   = [PsCustomObject]@{
            "Root"    = "Build.Build.windowsx86";
            "Target"  = "i686-pc-windows-msvc";
            "PackZip" = $true;
            "PackTar" = $false;
        };
        "windows.x64"   = [PsCustomObject]@{
            "Root"    = "Build.Build.windowsx64";
            "Target"  = "x86_64-pc-windows-msvc";
            "PackZip" = $true;
            "PackTar" = $false;
        };
        "windows.arm64" = [PsCustomObject]@{
            "Root"    = "Build.Build.windowsarm64";
            "Target"  = "aarch64-pc-windows-msvc";
            "PackZip" = $true;
            "PackTar" = $false;
        };
        "linux.x86"     = [PsCustomObject]@{
            "Root"    = "Build.Build.linuxx86";
            "Target"  = "i686-unknown-linux-gnu";
            "PackZip" = $false;
            "PackTar" = $true;
        };
        "linux.x64"     = [PsCustomObject]@{
            "Root"    = "Build.Build.linuxx64";
            "Target"  = "x86_64-unknown-linux-gnu";
            "PackZip" = $false;
            "PackTar" = $true;
        };
        "linux.arm"     = [PsCustomObject]@{
            "Root"    = "Build.Build.linuxarm";
            "Target"  = "arm-unknown-linux-gnueabi";
            "PackZip" = $false;
            "PackTar" = $true;
        };
        "linux.arm64"   = [PsCustomObject]@{
            "Root"    = "Build.Build.linuxarm64";
            "Target"  = "aarch64-unknown-linux-gnu";
            "PackZip" = $false;
            "PackTar" = $true;
        };
        "macos.x64"     = [PsCustomObject]@{
            "Root"    = "Build.Build.macosx64";
            "Target"  = "x86_64-apple-darwin";
            "PackZip" = $false;
            "PackTar" = $true;
        };
    }

    $flavors.GetEnumerator() | ForEach-Object {
        $flavor = $_.Name
        $settings = $_.Value
        $root = $_.Value.Root
        $target = $_.Value.Target
        Write-Host "Processing build: Flavor = $flavor, Root = $root, Target = $target, PackZip = $($settings.PackZip), PackTar = $($settings.PackTar)"

        # Create zip for github release
        if ($settings.PackZip) {
            $zipFilePath = ".\Releases\GithubReleases\rnp.$(build.tag).$flavor.zip"
            Write-Host "Packing to $zipFilePath"
            7z -tzip a $zipFilePath .\$root\bin\*
        }

        # Create tar.gz for github release
        if ($settings.PackTar) {
            $tarFilePath = ".\Releases\GithubReleases\rnp.$(build.tag).$flavor.tar.gz"
            Write-Host "Packing to $tarFilePath"
            tar -cvzf $tarFilePath --directory .\$root\bin *
        }

        # Copy symbols for github release
        $symbolDir = ".\Staging\Symbols\$flavor"
        Write-Host "Copying symbol to $symbolDir"
        New-Item -ItemType Directory -Path $symbolDir
        Copy-Item -Path .\$root\symbols\* $symbolDir -Verbose -Force

        # Generate nuget package
        $nugetProjectRoot = ".\Staging\NugetPackages\$flavor"
        Write-Host "Creating nuget package under $nugetProjectRoot"
        New-Item -ItemType Directory -Path "$nugetProjectRoot" | Out-Null
        Copy-Item -Path .\$root\bin\* $nugetProjectRoot -Verbose -Force
        EvaluateTemplateFile ".\Build.Build.windowsx64\templates\nuget_packages\rnp_nupkg.csproj" "$nugetProjectRoot\rnp_nupkg.csproj"
        dotnet pack $nugetProjectRoot\rnp_nupkg.csproj -o .\Releases\NugetPackages
    }
}

# Pack symbols for github release
function PackSymbolsPackagesForGithubRelease() {
    $symbolsZipFilePath = ".\Releases\GithubReleases\rnp.$(build.tag).symbols.zip"
    Write-Host "Pack all symbols to $symbolsZipFilePath"
    7z -tzip a $symbolsZipFilePath .\Staging\Symbols\*
}

# Crate.io
function PackRustCrate() {
    New-Item -ItemType Directory -Path ".\Releases\Crate.io"
    Write-Host "Pack source as crate to .\Releases\Crate.io"
    Copy-Item -Path .\Build.Build.linuxx64\crate\* .\Releases\Crate.io -Verbose -Force
}

# Chocolatey
function PackChocolateyPackages() {
    New-Item -ItemType Directory -Path ".\Releases\Chocolatey"
    New-Item -ItemType Directory -Path ".\Staging\Chocolatey"
    New-Item -ItemType Directory -Path ".\Staging\Chocolatey\tools"

    $fileHashs = [pscustomobject]@{
        "ZipX86" = (Get-FileHash ".\Releases\GithubReleases\rnp.$(build.tag).windows.x86.zip" -Algorithm SHA256).Hash;
        "ZipX64" = (Get-FileHash ".\Releases\GithubReleases\rnp.$(build.tag).windows.x64.zip" -Algorithm SHA256).Hash;
        "BinX86" = (Get-FileHash ".\Build.Build.windowsx86\bin\rnp.exe" -Algorithm SHA256).Hash;
        "BinX64" = (Get-FileHash ".\Build.Build.windowsx64\bin\rnp.exe" -Algorithm SHA256).Hash;
    }
    Write-Host "File hash: $fileHashs"
    EvaluateTemplateFileWithFileHash ".\Build.Build.windowsx64\chocolatey\rnp.nuspec" ".\Staging\Chocolatey\rnp.nuspec" $fileHashs
    EvaluateTemplateFileWithFileHash ".\Build.Build.windowsx64\chocolatey\tools\chocolateyInstall.ps1" ".\Staging\Chocolatey\tools\chocolateyInstall.ps1" $fileHashs
    EvaluateTemplateFileWithFileHash ".\Build.Build.windowsx64\chocolatey\tools\VERIFICATION.txt" ".\Staging\Chocolatey\tools\VERIFICATION.txt" $fileHashs
    EvaluateTemplateFileWithFileHash ".\Build.Build.windowsx64\chocolatey\tools\LICENSE.txt" ".\Staging\Chocolatey\tools\LICENSE.txt" $fileHashs
    choco pack ".\Staging\Chocolatey\rnp.nuspec" --outputdirectory ".\Releases\Chocolatey\"
}

# Utility functions for evaluating templates
function EvaluateTemplateFileWithFileHash($templateFile, $targetFile, $fileHashs) {
    $templateFileContent = gc $templateFile;
    $targetFileContent = EvaluateTemplate $templateFileContent
    $targetFileContent = $targetFileContent.Replace("{rnp_bin_hash_x86}", $fileHashs.BinX86).Replace("{rnp_bin_hash_x64}", $fileHashs.BinX64).Replace("{package_zip_hash_x86}", $fileHashs.ZipX86).Replace("{package_zip_hash_x64}", $fileHashs.ZipX64);

    $utf8NoBom = New-Object System.Text.UTF8Encoding $False
    [System.IO.File]::WriteAllLines($targetFile, $targetFileContent, $utf8NoBom)
}

function EvaluateTemplateFile($templateFile, $targetFile) {
    $templateFileContent = gc $templateFile;
    $targetFileContent = EvaluateTemplate $templateFileContent

    $utf8NoBom = New-Object System.Text.UTF8Encoding $False
    [System.IO.File]::WriteAllLines($targetFile, $targetFileContent, $utf8NoBom)
}

function EvaluateTemplate($template) {
    return $template.Replace("{build_branch_name}", "$(build.branch_name)").Replace("{build_tag}", "$(build.tag)").Replace("{version}", "$(Build.BuildNumber)").Replace("{target_short}", $flavor).Replace("{target}", $target)
}

PackAllReleasePackages