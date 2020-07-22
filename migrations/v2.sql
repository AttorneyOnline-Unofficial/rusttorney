-- Remove PRIMARY KEY constraint from `hdid`
CREATE TABLE IF NOT EXISTS hdids_new (
	hdid TEXT,
	ipid int NOT NULL,
	FOREIGN KEY (ipid) REFERENCES ipids(ipid)
		ON DELETE SET NULL,
	UNIQUE (hdid, ipid)
);
INSERT INTO hdids_new SELECT * FROM hdids;
DROP TABLE hdids CASCADE;
ALTER TABLE hdids_new RENAME TO hdids;

UPDATE general_info SET db_version = 2;