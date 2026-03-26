CREATE DATABASE camagru;
USE camagru;

CREATE TABLE User (
    id INT AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password VARCHAR(255) NOT NULL,
    isVerified BOOLEAN DEFAULT FALSE,
	verificationToken VARCHAR(255),
	resetVerificationToken VARCHAR(255),
	resetExpirationToken VARCHAR(255),
	notifyComment BOOLEAN DEFAULT TRUE
);

CREATE TABLE Post (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT REFERENCES User(id),
	postDate DATE NOT NULL,
	imagePath VARCHAR(255) NOT NULL
);

CREATE TABLE Post (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT REFERENCES User(id),
	postDate DATE NOT NULL,
	imagePath VARCHAR(255) NOT NULL
);

CREATE TABLE Like (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT REFERENCES User(id),
	post_id INT REFERENCES Post(id),
);

CREATE TABLE Comment (
    id INT AUTO_INCREMENT PRIMARY KEY,
    user_id INT REFERENCES User(id),
	post_id INT REFERENCES Post(id),
	comment VARCHAR(255) NOT NULL
);


INSERT INTO User (username, email, password) VALUES
('john_doe', 'john@example.com', 'password123'),
('jane_smith', 'jane@example.com', 'password456');

INSERT INTO Post (user_id, postDate, imagePath) VALUES
(1, '2023-01-01', 'path/to/image1.jpg'),
(2, '2023-01-02', 'path/to/image2.jpg');

INSERT INTO Comment (user_id, post_id, comment) VALUES
(1, 1, 'Great photo!'),
(2, 2, 'Nice shot!');