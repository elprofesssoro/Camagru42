'use strict'

document.querySelector("form").addEventListener("submit", register);

async function register(event) {
	event.preventDefault();
	const email = document.querySelector("#inputEmail").value;
	const username = document.querySelector("#inputName").value;
	const password = document.querySelector("#inputPassword").value;

	const emailResult = validEmail(email);
	if (emailResult !== "1") {
		showPopup(emailResult, "error");
		return;
	}

	const nameResult = validUsername(username);
	if (nameResult !== "1") {
		showPopup(nameResult, "error");
		return;
	}

	const passResult = validPass(password);
	if (passResult !== "1") {
		showPopup(passResult, "error");
		return;
	}

	console.log("Input is valid");

	const respone = await callApi("register", {
		method: "POST",
		headers: {
			"Content-Type": "application/json"
		},
		body: JSON.stringify({ email, username, password })
	});
	if (respone && respone.ok) {
		console.log("Registration successful");
		showPopup("Registration successful!", "success");
		setTimeout(() => {
			window.location.href = "esent.html";
		}, 1000);
	}
	else if (respone.status === 409) {
		showPopup("Email or username already exists", "error");
	}
	else {
		showPopup("Registration failed", "error");
	}

}