CREATE TABLE
  discord_users (
    name VARCHAR(255) NOT NULL,
    discord_id BIGINT UNSIGNED NOT NULL PRIMARY KEY,
    steam_id VARCHAR(255),
    mode SMALLINT UNSIGNED
  );
