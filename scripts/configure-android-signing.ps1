# Run locally with PowerShell 7. Prompts conceal passwords; nothing is stored in source.
[CmdletBinding()]
param(
    [string]$Repository = 'Ryderey/papr',
    [string]$KeystorePath = (Join-Path $PSScriptRoot '../papr-release.keystore'),
    [string]$KeyAlias
)

$ErrorActionPreference = 'Stop'
if ($PSVersionTable.PSVersion.Major -lt 7) {
    throw 'Use PowerShell 7 (pwsh) to run this script.'
}
$keystore = (Resolve-Path -LiteralPath $KeystorePath).Path
$gh = (Get-Command gh -ErrorAction Stop).Source
$keytool = (Get-Command keytool -ErrorAction Stop).Source

function Read-ConcealedPassword([string]$Prompt) {
    $secure = Read-Host $Prompt -AsSecureString
    $pointer = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($secure)
    try { return [Runtime.InteropServices.Marshal]::PtrToStringBSTR($pointer) }
    finally {
        [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($pointer)
        $secure.Dispose()
    }
}

function Set-SigningSecret([string]$Name, [string]$Value) {
    $info = [Diagnostics.ProcessStartInfo]::new()
    $info.FileName = $gh
    foreach ($argument in @('secret', 'set', $Name, '--repo', $Repository)) {
        $info.ArgumentList.Add($argument)
    }
    $info.UseShellExecute = $false
    $info.RedirectStandardInput = $true
    $info.RedirectStandardOutput = $true
    $info.RedirectStandardError = $true
    $process = [Diagnostics.Process]::Start($info)
    try {
        # Write exactly the secret, without a PowerShell pipeline's trailing newline.
        $stdout = $process.StandardOutput.ReadToEndAsync()
        $stderr = $process.StandardError.ReadToEndAsync()
        $process.StandardInput.Write($Value)
        $process.StandardInput.Close()
        $process.WaitForExit()
        $null = $stdout.GetAwaiter().GetResult()
        $null = $stderr.GetAwaiter().GetResult()
        if ($process.ExitCode -ne 0) { throw "Could not configure $Name. Check gh authentication and repository access." }
    }
    finally { $process.Dispose() }
}

# Child processes read credentials from this process's environment, not command arguments.
$previousStorePassword = [Environment]::GetEnvironmentVariable('PAPR_SIGNING_STORE_PASSWORD', 'Process')
$previousKeyPassword = [Environment]::GetEnvironmentVariable('PAPR_SIGNING_KEY_PASSWORD', 'Process')
$proofFile = Join-Path ([IO.Path]::GetTempPath()) ('papr-signing-' + [guid]::NewGuid() + '.csr')
$storePassword = $null
$keyPassword = $null
$encodedKey = $null
try {
    $storePassword = Read-ConcealedPassword 'Keystore password'
    if (-not $storePassword) { throw 'Keystore password must not be empty.' }
    [Environment]::SetEnvironmentVariable('PAPR_SIGNING_STORE_PASSWORD', $storePassword, 'Process')
    if (-not $KeyAlias) {
        $listing = & $keytool '-J-Duser.language=en' -list -v -keystore $keystore '-storepass:env' PAPR_SIGNING_STORE_PASSWORD 2>&1
        if ($LASTEXITCODE -ne 0) { throw 'Cannot open this keystore. Check the keystore password.' }
        $aliases = @([regex]::Matches(($listing -join "`n"), '(?m)^Alias name:\s*(.+)$') | ForEach-Object { $_.Groups[1].Value.Trim() })
        if ($aliases.Count -eq 1) {
            $KeyAlias = $aliases[0]
            Write-Host "Using existing key alias: $KeyAlias"
        }
        else {
            Write-Host ('Existing aliases: ' + ($aliases -join ', '))
            $KeyAlias = Read-Host 'Signing key alias'
        }
    }
    if ([string]::IsNullOrWhiteSpace($KeyAlias)) { throw 'Key alias is required.' }
    $keyPassword = Read-ConcealedPassword 'Key password (press Enter to reuse keystore password)'
    if (-not $keyPassword) { $keyPassword = $storePassword }
    [Environment]::SetEnvironmentVariable('PAPR_SIGNING_KEY_PASSWORD', $keyPassword, 'Process')

    # Check both store and private-key passwords before configuring any remote Secret.
    $certificateOutput = & $keytool '-J-Duser.language=en' -exportcert -rfc -keystore $keystore -alias $KeyAlias '-storepass:env' PAPR_SIGNING_STORE_PASSWORD 2>&1
    if ($LASTEXITCODE -ne 0) { throw 'Cannot read this signing certificate. Check keystore password and alias.' }
    $proofOutput = & $keytool '-J-Duser.language=en' -certreq -keystore $keystore -alias $KeyAlias '-storepass:env' PAPR_SIGNING_STORE_PASSWORD '-keypass:env' PAPR_SIGNING_KEY_PASSWORD -file $proofFile 2>&1
    if ($LASTEXITCODE -ne 0) { throw 'Cannot access the private signing key. Check the key password.' }
    if (($proofOutput -join "`n") -match 'Ignoring user-specified -keypass') {
        # keytool signed the CSR with the store password for this PKCS12 key.
        $keyPassword = $storePassword
        [Environment]::SetEnvironmentVariable('PAPR_SIGNING_KEY_PASSWORD', $keyPassword, 'Process')
    }
    $pem = $certificateOutput -join "`n"
    $match = [regex]::Match($pem, '(?s)-----BEGIN CERTIFICATE-----(.*?)-----END CERTIFICATE-----')
    if (-not $match.Success) { throw 'No signing certificate was exported.' }
    $certificate = [Security.Cryptography.X509Certificates.X509Certificate2]::new([Convert]::FromBase64String($match.Groups[1].Value))
    try {
        if ($certificate.NotAfter -le [DateTime]::Now) { throw 'Signing certificate has expired.' }
        $fingerprint = $certificate.GetCertHashString([Security.Cryptography.HashAlgorithmName]::SHA256).ToLowerInvariant()
    }
    finally { $certificate.Dispose() }

    $encodedKey = [Convert]::ToBase64String([IO.File]::ReadAllBytes($keystore))
    Set-SigningSecret 'ANDROID_KEYSTORE_BASE64' $encodedKey
    Set-SigningSecret 'ANDROID_KEYSTORE_PASSWORD' $storePassword
    Set-SigningSecret 'ANDROID_KEY_ALIAS' $KeyAlias
    Set-SigningSecret 'ANDROID_KEY_PASSWORD' $keyPassword
    & $gh variable set ANDROID_SIGNING_CERT_SHA256 --repo $Repository --body $fingerprint
    if ($LASTEXITCODE -ne 0) { throw 'Signing Secrets saved, but certificate variable failed. Rerun setup before packaging.' }
    Write-Host "Signing Secrets configured for $Repository. Public certificate SHA-256: $fingerprint"
    Write-Host 'Keep a secure backup of the existing keystore and passwords. Compare this certificate with the installed APK before updating.'
}
finally {
    [Environment]::SetEnvironmentVariable('PAPR_SIGNING_STORE_PASSWORD', $previousStorePassword, 'Process')
    [Environment]::SetEnvironmentVariable('PAPR_SIGNING_KEY_PASSWORD', $previousKeyPassword, 'Process')
    if (Test-Path -LiteralPath $proofFile) { Remove-Item -LiteralPath $proofFile }
    $storePassword = $null
    $keyPassword = $null
    $encodedKey = $null
}
