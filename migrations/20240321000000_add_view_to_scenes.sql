-- Add view field to scenes table
ALTER TABLE scenes ADD COLUMN view TEXT NOT NULL DEFAULT '{"center": {"x": 0.0, "y": 0.0}, "diagonal": 25.0}'; 