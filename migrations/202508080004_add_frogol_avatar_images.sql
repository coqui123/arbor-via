-- Add table for frogol avatar images
CREATE TABLE frogol_avatar_images (
    id TEXT PRIMARY KEY,
    frogol_id TEXT NOT NULL,
    image_filename TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
    FOREIGN KEY(frogol_id) REFERENCES frogols(id) ON DELETE CASCADE
);

-- Add index for efficient lookups
CREATE INDEX idx_frogol_avatar_images_frogol_id ON frogol_avatar_images(frogol_id);
