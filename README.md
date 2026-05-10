# 📸 Camagru42

A small, Instagram-like website that allows users to create and share photomontages. Built from scratch to understand and implement the core functionalities required by modern web applications with a user base.

*(Note: This project is part of the 42 School curriculum.)*

---

## 🌟 Features

* **User Authentication:** Secure registration, login, and password recovery.
* **Email Verification:** Account activation and notification emails.
* **Gallery & Pagination:** A public feed of all user-created photomontages.
* **Studio / Camera:** Take pictures using the webcam or upload images, and overlay them with custom stickers/frames.
* **Social Interactions:** Users can like and comment on posts.

---

## 🌳 Architecture & Branches (The Learning Journey)

This project is a journey from low-level networking to modern framework architecture. To document this progress, the repository is split into distinct branches:

### 1. `main` Branch (The Hard Way)
In this branch, the backend is built **entirely from scratch without any web frameworks**. 
* Custom TCP-Listener
* Manual HTTP Request/Response parsing
* Custom routing and cookie handling
* *Goal:* Deeply understand how the HTTP protocol works under the hood.

### 2. `axum` Branch (The Modern Way)
This branch represents the refactored, production-ready version of the backend.
* Powered by the **Axum** framework.
* Utilizes Extractors, Middlewares, and efficient routing.
* Cleaner, safer, and significantly less boilerplate code.

### 🚀 Future Plans (Frontend)
Currently, the frontend is built using Vanilla JS/HTML/CSS to fulfill the base requirements. **A future branch is planned to completely rewrite the frontend using React** for a smooth, single-page application experience.

---

## 🛠️ Tech Stack

**Backend:**
* [Rust](https://www.rust-lang.org/)
* [Axum](https://github.com/tokio-rs/axum) (in the main branch)
* [Tokio](https://tokio.rs/) (Asynchronous runtime)
* [SQLx](https://github.com/launchbadge/sqlx) (Compile-time checked SQL)
* PostgreSQL (Database)

**Frontend:**
* HTML5, CSS3, Vanilla JavaScript
* *(React coming soon)*

---

## 🖼️ Screenshots

### Gallery & Feed
![Gallery View](./docs/gallery_screenshot.png)

### Studio & Photomontage
![Create Studio](./docs/studio_screenshot.png)

### User Profile
![Profile View](./docs/profile_screenshot.png)

---

## 🚀 Getting Started

1. Clone the repository:
   ```bash
   git clone [https://github.com/DEIN_USERNAME/camagru42.git](https://github.com/DEIN_USERNAME/camagru42.git)
   cd camagru42
2. Copy content from env-pub to .env
3. Run make command
