'use strict'

document.querySelector("#buttonLog").addEventListener("click", logIn);

function logIn(event) {
	event.preventDefault();
	const cred = document.querySelector("#inputCred").value;
	const password = document.querySelector("#inputPassword").value;

	if (cred.includes("@")) {
		const emailResult = validEmail(cred);
		if (emailResult !== "1") {
			showPopup(emailResult);
			return;
		}
	}
	else {
		const usernameResult = validUsername(cred);
		if (usernameResult !== "1") {
			showPopup(usernameResult);
			return;
		}
	}

	const passResult = validPass(password);
	if (passResult !== "1") {
		showPopup(passResult);
		return;
	}

	console.log("Input is valid");

}