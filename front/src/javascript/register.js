'use strict'

document.querySelector("form").addEventListener("submit", register);

function register(event) {
	event.preventDefault();
	const email = document.querySelector("#inputEmail").value;
	const username = document.querySelector("#inputName").value;
	const password = document.querySelector("#inputPassword").value;

	const emailResult = validEmail(email);
	if (emailResult !== "1") {
		showPopup(emailResult);
		return;
	}

	const nameResult = validUsername(username);
	if (nameResult !== "1") {
		showPopup(nameResult);
		return;
	}

	const passResult = validPass(password);
	if (passResult !== "1") {
		showPopup(passResult);
		return;
	}

	console.log("Input is valid");

	callApi("register", {
		method: "POST",
		headers: {
			"Content-Type": "application/json"
		},
		body: JSON.stringify({ email, username, password })
	}).then((data) => {
		console.log(data);
	});

}