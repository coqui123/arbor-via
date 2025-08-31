-- 2025-08-07-0002_add_theme_to_frogols.sql
ALTER TABLE frogols ADD COLUMN theme TEXT DEFAULT 'default';
