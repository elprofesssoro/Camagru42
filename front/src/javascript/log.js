'use strict'

document.querySelector("#buttonLog").addEventListener("click", logIn);

async function logIn(event) {
	event.preventDefault();
	const cred = document.querySelector("#inputCred").value;
	const password = document.querySelector("#inputPassword").value;

	if (cred.includes("@")) {
		const emailResult = validEmail(cred);
		if (emailResult !== "1") {
			showPopup(emailResult, "error");
			return;
		}
	}
	else {
		const usernameResult = validUsername(cred);
		if (usernameResult !== "1") {
			showPopup(usernameResult, "error");
			return;
		}
	}

	const passResult = validPass(password);
	if (passResult !== "1") {
		showPopup(passResult, "error");
		return;
	}

	console.log("Input is valid");
	const response = await callApi("login", {
		method: "POST",
		headers: {
			"Content-Type": "application/json"
		},
		body: JSON.stringify({ cred, password })
	});
	console.log(response);
	if (response && response.ok) {
		console.log("Login successful");
		window.location.href = "gallery.html";
	} else {
		if (response.status === 401) {
			showPopup("Wrong credentials", "error");
		}
		else if (response.status === 403) {
			showPopup("Account not verified", "error");
		}
	}
}