param(
    [Parameter(Mandatory=$true)]
    [string]$PointName
)

# Create FreePoint object
$point = @{
    name = $PointName
    object_type = "FreePoint"
    properties = @{
        value = "0, 0"
    }
} | ConvertTo-Json

# Add FreePoint to scene 1
$response = Invoke-RestMethod -Uri "http://localhost:8080/scenes/1" -Method Post -Body $point -ContentType "application/json"
Write-Host "Added FreePoint '$PointName' to scene 1" 