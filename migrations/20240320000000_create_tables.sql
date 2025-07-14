-- Create scenes table
CREATE TABLE IF NOT EXISTS scenes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    created_at DATETIME NOT NULL
);

-- Create scene_objects table
CREATE TABLE IF NOT EXISTS scene_objects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    scene_id INTEGER NOT NULL,
    object_type TEXT NOT NULL,
    object_name TEXT NOT NULL,
    properties TEXT NOT NULL,
    FOREIGN KEY (scene_id) REFERENCES scenes(id) ON DELETE CASCADE
); 