Param(
    [Parameter(Mandatory = $true)]
    [string] $BuildBranchName,

    [Parameter(Mandatory = $true)]
    [string] $BuildTag,

    [Parameter(Mandatory = $true)]
    [string] $BuildNumber
)

function PackAllReleasePackages() {
    Write-Host "Pack all release packages: BuildBranch = $BuildBranchName, BuildTag = $BuildTag, BuildNumber = $BuildNumber"

    PackPerFlavorReleases
    PackSymbolsPackagesForGithubRelease
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
            $zipFilePath = ".\Releases\GithubReleases\rnp.$BuildTag.$flavor.zip"
            Write-Host "Packing to $zipFilePath"
            7z -tzip a $zipFilePath .\$root\bin\*
        }

        # Create tar.gz for github release
        if ($settings.PackTar) {
            $tarFilePath = ".\Releases\GithubReleases\rnp.$BuildTag.$flavor.tar.gz"
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
        EvaluateTemplateFile ".\Build.Build.windowsx64\templates\nuget_packages\rnp_nupkg.csproj" "$nugetProjectRoot\rnp_nupkg.csproj" $flavor $target
        dotnet pack $nugetProjectRoot\rnp_nupkg.csproj -o .\Releases\NugetPackages
    }
}

# Pack source packages for github release.
# The reason we are doing it is because we modify the version on the fly during our build process instead of
# checking in into our code base, so only the build pipeline have the final source code package.
function PackSourcePackagesForGithubRelease() {
    Write-Host "Publish source packages to .\Releases\GithubReleases"
    Copy-Item -Path .\Build.Build.linuxx64\source\* .\Releases\GithubReleases -Verbose -Force
}

# Pack symbols for github release
function PackSymbolsPackagesForGithubRelease() {
    $symbolsZipFilePath = ".\Releases\GithubReleases\rnp.$BuildTag.symbols.zip"
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
        "ZipX86" = (Get-FileHash ".\Releases\GithubReleases\rnp.$BuildTag.windows.x86.zip" -Algorithm SHA256).Hash;
        "ZipX64" = (Get-FileHash ".\Releases\GithubReleases\rnp.$BuildTag.windows.x64.zip" -Algorithm SHA256).Hash;
        "BinX86" = (Get-FileHash ".\Build.Build.windowsx86\bin\rnp.exe" -Algorithm SHA256).Hash;
        "BinX64" = (Get-FileHash ".\Build.Build.windowsx64\bin\rnp.exe" -Algorithm SHA256).Hash;
    }
    Write-Host "File hash: $fileHashs"
    EvaluateTemplateFileWithFileHash ".\Build.Build.windowsx64\templates\chocolatey\rnp.nuspec" ".\Staging\Chocolatey\rnp.nuspec" $fileHashs "" ""
    EvaluateTemplateFileWithFileHash ".\Build.Build.windowsx64\templates\chocolatey\tools\chocolateyInstall.ps1" ".\Staging\Chocolatey\tools\chocolateyInstall.ps1" $fileHashs "" ""
    EvaluateTemplateFileWithFileHash ".\Build.Build.windowsx64\templates\chocolatey\tools\VERIFICATION.txt" ".\Staging\Chocolatey\tools\VERIFICATION.txt" $fileHashs "" ""
    EvaluateTemplateFileWithFileHash ".\Build.Build.windowsx64\templates\chocolatey\tools\LICENSE.txt" ".\Staging\Chocolatey\tools\LICENSE.txt" $fileHashs "" ""
    choco pack ".\Staging\Chocolatey\rnp.nuspec" --outputdirectory ".\Releases\Chocolatey\"
}

# Utility functions for evaluating templates
function EvaluateTemplateFileWithFileHash($templateFile, $targetFile, $fileHashs, $targetShortName, $targetFullName) {
    $templateFileContent = Get-Content $templateFile;
    $targetFileContent = EvaluateTemplate $templateFileContent $targetShortName $targetFullName
    $targetFileContent = $targetFileContent.Replace("{rnp_bin_hash_x86}", $fileHashs.BinX86).Replace("{rnp_bin_hash_x64}", $fileHashs.BinX64).Replace("{package_zip_hash_x86}", $fileHashs.ZipX86).Replace("{package_zip_hash_x64}", $fileHashs.ZipX64);

    $utf8NoBom = New-Object System.Text.UTF8Encoding $False
    [System.IO.File]::WriteAllLines($targetFile, $targetFileContent, $utf8NoBom)
}

function EvaluateTemplateFile($templateFile, $targetFile, $targetShortName, $targetFullName) {
    $templateFileContent = Get-Content $templateFile;
    $targetFileContent = EvaluateTemplate $templateFileContent $targetShortName $targetFullName

    $utf8NoBom = New-Object System.Text.UTF8Encoding $False
    [System.IO.File]::WriteAllLines($targetFile, $targetFileContent, $utf8NoBom)
}

function EvaluateTemplate($template, $targetShortName, $targetFullName) {
    return $template.Replace("{build_branch_name}", $BuildBranchName).Replace("{build_tag}", $BuildTag).Replace("{version}", $BuildNumber).Replace("{target_short}", $targetShortName).Replace("{target}", $targetFullName)
}

PackAllReleasePackages