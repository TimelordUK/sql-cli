# Chocolatey packaging

This folder contains the template files used by the release workflow to publish
sql-cli to the [Chocolatey Community Repository](https://community.chocolatey.org/).

## Layout

- `sql-cli.nuspec.template` — package metadata. `__VERSION__` is replaced at pack time.
- `tools/chocolateyInstall.ps1.template` — downloads the Windows binary from the matching
  GitHub release and verifies its SHA256. `__VERSION__` and `__CHECKSUM__` are replaced
  at pack time.
- `tools/VERIFICATION.txt` — instructions for the Chocolatey moderation team to verify
  the package contents against the published source.
- `tools/LICENSE.txt` — a copy of the upstream MIT license (required by Chocolatey
  moderation for binary packages).

Chocolatey auto-shims any `.exe` placed in `tools/`, so after install the binary is
available on `PATH` as `sql-cli`.

## Publishing

Publishing is automated in `.github/workflows/release-manual.yml`. The workflow runs
after the GitHub release is created — at that point the Windows binary asset is
available at a stable URL, so the workflow:

1. Downloads `sql-cli-windows-x64.exe` from the release.
2. Computes its SHA256.
3. Substitutes `__VERSION__` and `__CHECKSUM__` into the templates.
4. Runs `choco pack` and `choco push`.

## One-time setup

1. Create a Chocolatey account at <https://community.chocolatey.org/account/Register>.
2. Generate an API key at <https://community.chocolatey.org/account>.
3. Add it as a GitHub repo secret named `CHOCOLATEY_API_KEY`.
4. The **first** submission of a brand-new package id goes through manual moderator
   review (typically a few days). Subsequent versions normally auto-pass.

## Testing locally

You need Chocolatey installed on Windows. From this folder:

```powershell
# Pretend a version & checksum:
(Get-Content sql-cli.nuspec.template) -replace '__VERSION__','1.74.0' | Set-Content sql-cli.nuspec
(Get-Content tools\chocolateyInstall.ps1.template) `
    -replace '__VERSION__','1.74.0' `
    -replace '__CHECKSUM__','<sha256 of the asset>' | Set-Content tools\chocolateyInstall.ps1
choco pack
choco install sql-cli -s . -y --force
sql-cli --version
choco uninstall sql-cli -y
```
