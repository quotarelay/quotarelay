Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

Push-Location $PSScriptRoot
try {
    npm publish --access public
}
finally {
    Pop-Location
}