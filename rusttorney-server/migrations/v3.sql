-- Remove FOREIGN KEY constraint from hdid_bans
CREATE TABLE hdid_bans_new(
  hdid TEXT PRIMARY KEY,
  ban_id INTEGER NOT NULL,
  FOREIGN KEY (ban_id) REFERENCES bans(ban_id)
      ON DELETE CASCADE
);
INSERT INTO hdid_bans_new SELECT * FROM hdid_bans;
DROP TABLE hdid_bans CASCADE;
ALTER TABLE hdid_bans_new RENAME TO hdid_bans;

UPDATE general_info SET db_version = 3;