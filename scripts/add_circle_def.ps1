
$scene_response = Invoke-RestMethod -Uri "http://localhost:8080/scenes" -Method Post
Write-Host "Created scene ID = $($scene_response.id)" 

# Create FixedPoint object
$point = @{
    name = "A"
    object_type = "FixedPoint"
    properties = @{
        value = "0, 0"
    }
} | ConvertTo-Json

# Add FreePoint to scene
$response = Invoke-RestMethod -Uri "http://localhost:8080/scenes/$($scene_response.id)" -Method Post -Body $point -ContentType "application/json"
Write-Host "Added FixedPoint A to scene ID = $($scene_response.id)"

# Create FreePoint object
$point = @{
    name = "X"
    object_type = "FreePoint"
    properties = @{
        value = "3, 4"
    }
} | ConvertTo-Json

# Add FreePoint to scene
$response = Invoke-RestMethod -Uri "http://localhost:8080/scenes/$($scene_response.id)" -Method Post -Body $point -ContentType "application/json"
Write-Host "Added FreePoint X to scene ID = $($scene_response.id)"

# Create Invariant object
$invariant = @{
    name = "invA"
    object_type = "Invariant"
    properties = @{
        formula = "d_sqr(A, X)"
    }
} | ConvertTo-Json

# Add Invariant to scene
$response = Invoke-RestMethod -Uri "http://localhost:8080/scenes/$($scene_response.id)" -Method Post -Body $invariant -ContentType "application/json"
Write-Host "Added Invariant invA to scene ID = $($scene_response.id)"

# Create Locus object
$locus = @{
    name = "locusA"
    object_type = "Locus"
    properties = @{
        point = "X"
    }
} | ConvertTo-Json

# Add Locus to scene
$response = Invoke-RestMethod -Uri "http://localhost:8080/scenes/$($scene_response.id)" -Method Post -Body $locus -ContentType "application/json"
Write-Host "Added Locus locusA to scene ID = $($scene_response.id)"

