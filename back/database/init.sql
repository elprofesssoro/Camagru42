CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    is_verified BOOLEAN DEFAULT FALSE,
    verification_token VARCHAR(255),
    reset_verification_token VARCHAR(255),
    reset_expiration_token VARCHAR(255),
    notify_comment BOOLEAN DEFAULT TRUE
);

CREATE TABLE sessions (
    session_token VARCHAR(128) PRIMARY KEY,
    user_id INT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    CONSTRAINT fk_sessions_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    user_id INT,
    post_date TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    image_path VARCHAR(255) NOT NULL UNIQUE,
    CONSTRAINT fk_posts_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

CREATE TABLE post_likes (
    user_id INT NOT NULL,
    post_id INT NOT NULL,
    PRIMARY KEY (user_id, post_id),
    CONSTRAINT fk_post_likes_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT fk_post_likes_post FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE
);

CREATE TABLE comments (
    id SERIAL PRIMARY KEY,
    user_id INT,
    post_id INT,
    comment VARCHAR(255) NOT NULL,
    CONSTRAINT fk_comments_user FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL,
    CONSTRAINT fk_comments_post FOREIGN KEY (post_id) REFERENCES posts(id) ON DELETE CASCADE
);

INSERT INTO users (username, email, password, isVerified) VALUES
('john_doe', 'john@example.com', 'Password123', TRUE),
('jane_smith', 'jane@example.com', 'Password456', FALSE);

INSERT INTO posts (user_id, post_date, image_path) VALUES
(1, '2023-01-01', 'path/to/image1.jpg'),
(2, '2023-01-02', 'path/to/image2.jpg');

INSERT INTO comments (user_id, post_id, comment) VALUES
(1, 1, 'Great photo!'),
(2, 2, 'Nice shot!');