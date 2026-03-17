'use strict'

let popupAlive = false;

function showPopup(message) {
	if (popupAlive)
		return;
	const popup = document.createElement("div");
	popup.className = "error-popup";
	popup.textContent = message;

	const linkElement = document.querySelector("form");
	linkElement.after(popup);
	popupAlive = true;
	setTimeout(() => {
		popup.remove();
		popupAlive = false;
	}, 1500);
}

function validEmail(email) {

	if (email === "") {
		return "Fill in all fields";
	}

	const emailRegex = /^[^.\s@]+@[^\s@]+\.[^\s@]+$/;
	if (!emailRegex.test(email)) {
		return "Invalid email format";
	}

	return "1";
}

function validUsername(username) {

	if (username === "") {
		return "Fill in all fields";
	}

	const usernameRegex = /^[a-zA-Z0-9_]{3,20}$/;
	if (!usernameRegex.test(username)) {
		return "Invalid username format";
	}

	return "1";
}

function validPass(password) {

	if (password === "") {
		return "Fill in all fields";
	}

	const length = password.length >= 5;
	const upper = /[A-Z]/.test(password);
	const lower = /[a-z]/.test(password);
	const number = /[0-9]/.test(password);
	const passed = [length, upper, lower, number].filter(Boolean).length;
	if (passed < 2) {
		return "Invalid password format";
	}

	return "1";
}