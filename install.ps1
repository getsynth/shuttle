function Install-Cargo-Shuttle {
    Write-Host @"
     _           _   _   _
 ___| |__  _   _| |_| |_| | ___
/ __| '_ \| | | | __| __| |/ _ \
\__ \ | | | |_| | |_| |_| |  __/
|___/_| |_|\__,_|\__|\__|_|\___|

https://www.shuttle.dev
https://github.com/shuttle-hq/shuttle

Please file an issue if you encounter any problems!
===================================================
"@

    $Arch = [Environment]::GetEnvironmentVariable("PROCESSOR_ARCHITECTURE", [EnvironmentVariableTarget]::Machine)
    $TempDir = $Env:TEMP

    if (!(Get-Command -CommandType Application -ErrorAction SilentlyContinue cargo.exe)) {
        if ($Arch -in "AMD64", "x86") {
			Write-Host "Could not find cargo.exe, Rust may not be installed" -ForegroundColor Red
			$Confirm = Read-Host -Prompt "Would you like to install Rust via Rustup? [y/N]"
			if ($Confirm -inotin "y", "yes") {
				Write-Host "Skipping rustup install, cargo-shuttle not installed"
				return
			}
			$RustupUrl = if ($Arch -eq "AMD64") { "https://win.rustup.rs/x86_64" } else { "https://win.rustup.rs/i686" }
			Invoke-WebRequest $RustupUrl -OutFile "$TempDir\rustup.exe"
			& "$TempDir\rustup.exe" toolchain install stable
			if ($?) {
				Remove-Item -ErrorAction SilentlyContinue "$TempDir\rustup.exe"
				Write-Host "Rust installed via Rustup, please re-run this script, you may need reopen your terminal" -ForegroundColor Green
				return
			}
			else {
				Remove-Item -ErrorAction SilentlyContinue "$TempDir\rustup.exe"
				Write-Host "Rust install via Rustup failed, please install Rust manually: https://rustup.rs/" -ForegroundColor Red
				return
			}
		}
		else {
			Write-Host "Could not find cargo.exe, Rust may not be installed." -ForegroundColor Red
			Write-Host "Rustup is only provided for x86 and x86_64, not $Arch" -ForegroundColor Red
			Write-Host "Please install Rust manually, more info at: https://rust-lang.github.io/rustup/installation/other.html" -ForegroundColor Red
			return
		}
    }

    if (Get-Command -CommandType Application -ErrorAction SilentlyContinue cargo-binstall.exe) {
        Write-Host "Installing cargo-shuttle using cargo binstall"
        cargo-binstall.exe cargo-shuttle --no-confirm
        if ($?) {
            Write-Host "Installed cargo-shuttle" -ForegroundColor Green
            return
        }
        else {
            Write-Host "Could not install from release using cargo binstall, trying manual binary download" -ForegroundColor Red
        }
    }
    else {
        Write-Host "Cargo binstall not found, trying manual binary download" -ForegroundColor Red
    }

    $RepoUrl = "https://github.com/shuttle-hq/shuttle"
    $CargoHome = if ($null -ne $Env:CARGO_HOME) { $Env:CARGO_HOME } else { "$HOME\.cargo" }

    if (($Arch -eq "AMD64") -and (Get-Command -CommandType Application -ErrorAction SilentlyContinue tar.exe)) {
        (Invoke-WebRequest "$RepoUrl/releases/latest" -Headers @{ "Accept" = "application/json" }).Content -match '"tag_name":"([^"]*)"' | Out-Null
        $LatestRelease = $Matches.1
        $BinaryUrl = "$RepoUrl/releases/download/$LatestRelease/cargo-shuttle-$LatestRelease-x86_64-pc-windows-msvc.tar.gz"
        Invoke-WebRequest $BinaryUrl -OutFile "$TempDir\cargo-shuttle.tar.gz"
        New-Item -ItemType Directory -Force "$TempDir\cargo-shuttle"
        tar.exe -xzf "$TempDir\cargo-shuttle.tar.gz" -C "$TempDir\cargo-shuttle"
        Move-Item -Force "$TempDir\cargo-shuttle\cargo-shuttle-x86_64-pc-windows-msvc-$LatestRelease\cargo-shuttle.exe" "$CargoHome\bin\cargo-shuttle.exe"
        Move-Item -Force "$TempDir\cargo-shuttle\cargo-shuttle-x86_64-pc-windows-msvc-$LatestRelease\shuttle.exe" "$CargoHome\bin\shuttle.exe"
        Remove-Item -Recurse -Force -ErrorAction SilentlyContinue "$TempDir\cargo-shuttle.tar.gz", "$TempDir\cargo-shuttle"
        Write-Host "Installed cargo-shuttle" -ForegroundColor Green
        return
    }
    elseif ($Arch -ne "AMD64") {
        Write-Host "Unsupported Architecture: Binaries are not currently built for $Arch, skipping manual binary download" -ForegroundColor Red
    }
    else {
        Write-Host "Could not find tar.exe, skipping manual binary download (required to extract the release asset)" -ForegroundColor Red
    }


    if (Get-Command -CommandType Application -ErrorAction SilentlyContinue cargo.exe) {
        Write-Host "Installing cargo-shuttle using cargo install (from source)"
        cargo.exe install cargo-shuttle --locked
        if ($?) {
            Write-Host "Installed cargo-shuttle" -ForegroundColor Green
            return
        }
        else {
            Write-Host "Could not install cargo-shuttle using cargo" -ForegroundColor Red
            return
        }
    }
    else {
        Write-Host "Could not find cargo.exe, Rust may not be installed" -ForegroundColor Red
    }
}


$OldErrorAction = $ErrorActionPreference
$ErrorActionPreference = "Stop"
Install-Cargo-Shuttle
$ErrorActionPreference = $OldErrorAction
