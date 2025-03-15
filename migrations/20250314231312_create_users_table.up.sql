-- Create the users table
CREATE TABLE users (
    id SERIAL PRIMARY KEY, -- Unique identifier for each user
    auth_type VARCHAR(50) NOT NULL, -- Store the authentication type as a string
    user_id BIGINT NOT NULL, -- User ID as a 64-bit integer
    name VARCHAR(255) NOT NULL, -- User's name
    avatar_url TEXT, -- URL to the user's avatar
    gravatar_id VARCHAR(255), -- Gravatar ID
    UNIQUE (auth_type, user_id) -- Ensure that the combination of auth_type and user_id is unique
);
