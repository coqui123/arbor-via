-- Add performance indexes for common queries
CREATE INDEX IF NOT EXISTS idx_links_frogol_active_order ON links(frogol_id, is_active, sort_order);
CREATE INDEX IF NOT EXISTS idx_leads_frogol_created ON leads(frogol_id, created_at);
CREATE INDEX IF NOT EXISTS idx_clicks_link_created ON clicks(link_id, created_at);
CREATE INDEX IF NOT EXISTS idx_frogols_user ON frogols(user_id);
