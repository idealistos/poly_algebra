# Create a new scene
$response = Invoke-RestMethod -Uri "http://localhost:8080/scenes" -Method Post
Write-Host "Created scene with ID: $($response.id)" 