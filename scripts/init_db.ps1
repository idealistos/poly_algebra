# Check if SQLite is installed
$sqliteVersion = sqlite3 --version
if (-not $?) {
    Write-Host "SQLite is not installed. Please install SQLite first."
    exit 1
}

# Create database directory if it doesn't exist
$dbDir = "data"
if (-not (Test-Path $dbDir)) {
    New-Item -ItemType Directory -Path $dbDir
}

# Database file path
$dbPath = Join-Path $dbDir "scenes.db"

# Remove existing database if it exists
if (Test-Path $dbPath) {
    Remove-Item $dbPath
}

# Create new database
Write-Host "Creating new database at $dbPath..."
sqlite3 $dbPath ".databases"

# Read and execute migration file
Write-Host "Applying migrations..."
$migrationPath = Join-Path "migrations" "20250217_000001_create_scene_tables.sql"
if (-not (Test-Path $migrationPath)) {
    Write-Host "Migration file not found at $migrationPath"
    exit 1
}

$migrationContent = Get-Content $migrationPath -Raw
$migrationContent | sqlite3 $dbPath

# Verify tables were created
Write-Host "Verifying tables..."
sqlite3 $dbPath ".tables"
sqlite3 $dbPath ".schema scenes"
sqlite3 $dbPath ".schema scene_objects"

Write-Host "Database initialization complete!" 