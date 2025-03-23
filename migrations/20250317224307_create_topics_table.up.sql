CREATE TABLE topics (
    id SERIAL PRIMARY KEY, -- Unique identifier for each topic
    title VARCHAR(255) NOT NULL, -- Title of the topic
    created_by BIGINT NOT NULL, -- User ID of the creator (Admin)
    created_at TIMESTAMP NOT NULL DEFAULT NOW(), -- Timestamp of when the topic was created
    cleared_at TIMESTAMP, -- Timestamp of when the topic was cleared
    FOREIGN KEY (created_by) REFERENCES users(id) ON DELETE CASCADE -- Reference to the users table
);

CREATE UNIQUE INDEX unique_active_topic ON topics (cleared_at)
WHERE cleared_at IS NULL;
