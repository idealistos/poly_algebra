-- Add name field to scenes table
ALTER TABLE scenes ADD COLUMN name TEXT NOT NULL DEFAULT '';

-- Create unique index on name
CREATE UNIQUE INDEX idx_scenes_name ON scenes(name);

-- Update existing scenes to have names based on their IDs
UPDATE scenes SET name = 'Scene ' || id WHERE name = ''; 