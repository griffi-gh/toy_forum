CREATE TYPE role_type AS ENUM ('user', 'moderator', 'admin');
CREATE TABLE users (
  user_id serial PRIMARY KEY,
  username VARCHAR(15) NOT NULL,
  email VARCHAR(255) UNIQUE NOT NULL,
  password_hash VARCHAR NOT NULL,
  created_on TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  last_activity TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  user_role role_type NOT NULL DEFAULT 'user',
);
