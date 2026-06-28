@{
    RootModule        = 'Forgum.psm1'
    ModuleVersion     = '0.4.0'
    GUID              = 'b5f8e9d3-4a72-4f5d-9e6f-2c4e1b8a3d9f'
    Author            = 'Forgum Contributors'
    Description       = 'Cross-platform cowsay + fortune + lolcat with a Rust animation engine. The PowerShell module is a thin UX layer; the engine does the rendering.'
    PowerShellVersion = '7.0'
    CompatiblePSEditions = @('Core')
    FunctionsToExport = @(
        'forgum',
        'Get-ForgumEngineBinary',
        'Get-ForgumConfigPath',
        'Invoke-ForgumEngine',
        'Initialize-ForgumConfig',
        'Set-ForgumConfig',
        'Get-ForgumConfig'
    )
    CmdletsToExport   = @()
    AliasesToExport   = @()
    PrivateData       = @{
        PSData = @{
            Tags       = @('cowsay', 'fortune', 'lolcat', 'animation', 'terminal', 'fun')
            ProjectUri = 'https://github.com/harish2222/Forgum'
            LicenseUri = 'https://github.com/harish2222/Forgum/blob/main/LICENSE'
        }
    }
}