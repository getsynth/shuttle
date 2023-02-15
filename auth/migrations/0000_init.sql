CREATE TABLE IF NOT EXISTS users (
  user_name TEXT PRIMARY KEY,
  secret TEXT UNIQUE,
  super_user BOOLEAN DEFAULT FALSE,
  account_tier TEXT DEFAULT "basic" NOT NULL
);
