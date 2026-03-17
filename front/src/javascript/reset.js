'use strict'

document.querySelector("form").addEventListener("submit", function(event) {
	event.preventDefault();
	const email = document.querySelector("#inputEmail").value;
	if (validEmail(email) !== "1") {
		showPopup("Invalid email address");
		return;
	}
});