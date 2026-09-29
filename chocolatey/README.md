# Chocolatey packaging

This folder contains the template files used by `publish-chocolatey.yml` to publish
sql-cli to the [Chocolatey Community Repository](https://community.chocolatey.org/).

## Layout

- `sql-cli.nuspec.template` — package metadata. `__VERSION__` is replaced at pack time.
- `tools/chocolateyInstall.ps1.template` — downloads the Windows binary from the matching
  GitHub release and verifies its SHA256. `__VERSION__` and `__CHECKSUM__` are replaced
  at pack time.

No `VERIFICATION.txt` / `LICENSE.txt`: the package downloads the binary rather than
embedding it, and the Chocolatey moderators asked for both files to be removed. The
nuspec's `packageSourceUrl` points moderators at this folder instead.

Chocolatey auto-shims any `.exe` placed in `tools/`, so after install the binary is
available on `PATH` as `sql-cli`.

## Publishing

Publishing is **manual**, and separate from the release workflow. crates.io gets every
release; Chocolatey gets only the versions we choose to submit, because each pushed
version sits in moderation for a long time.

Once a GitHub release exists, run `.github/workflows/publish-chocolatey.yml` with its
version (Actions tab, or `gh workflow run publish-chocolatey.yml -f version=1.85.13`).
It:

1. Downloads `sql-cli-windows-x64.exe` from that release.
2. Computes its SHA256.
3. Substitutes `__VERSION__` and `__CHECKSUM__` into the templates (taken from `main`,
   so packaging fixes apply to older releases too).
4. Runs `choco pack` and `choco push`, and uploads the `.nupkg` as a build artifact.

## One-time setup

1. Create a Chocolatey account at <https://community.chocolatey.org/account/Register>.
2. Generate an API key at <https://community.chocolatey.org/account>.
3. Add it as a GitHub repo secret named `CHOCOLATEY_API_KEY`.
4. The **first** submission of a brand-new package id goes through manual moderator
   review, which can take weeks to months.

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
