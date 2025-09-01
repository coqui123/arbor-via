-- Add visibility and kind to links
ALTER TABLE links ADD COLUMN is_active INTEGER NOT NULL DEFAULT 1;
ALTER TABLE links ADD COLUMN kind TEXT NOT NULL DEFAULT 'link';
