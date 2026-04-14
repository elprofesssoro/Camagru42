'use strict'

document.querySelector("form").addEventListener("submit", async function (event) {
	event.preventDefault();
	const email = document.querySelector("#inputEmail").value;
	if (validEmail(email) !== "1") {
		showPopup("Invalid email address");
		return;
	}
	const result = await callApi("re-pass", {
		method: "POST",
		headers: {
			"Content-Type": "application/json"
		},
		body: JSON.stringify({ email: email })
	});
	if (result && result.ok) {
		showPopup("Password reset email sent!. Check your inbox.", "success", "#inputEmail", 20000);
	}
	else {
		showPopup("An error occurred while trying to reset the password", "error");
	}

});