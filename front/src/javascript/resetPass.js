const urlParams = new URLSearchParams(window.location.search);
const resetToken = urlParams.get('token');

if (!resetToken) {
	document.querySelector("form").style.display = "none";
	showPopup("Invalid or missing reset link.", "error");
}

document.querySelector("form").addEventListener("submit", async function (event) {
	event.preventDefault();
	if (!resetToken) return;
	const password = document.querySelector("#password").value;
	const confirmPassword = document.querySelector("#confirmPassword").value;
	if (password !== confirmPassword) {
		showPopup("Passwords do not match", "error", "form");
		return;
	}
	else {
		document.querySelector("#confirmPassword").setCustomValidity("");
	}

	const is_valid = validPass(password);
	if (is_valid !== "1") {
		showPopup(is_valid, "error", "#password");
		return;
	}

	const result = await callApi(`re-pass/new?token=${resetToken}`, {
		method: "POST",
		headers: {
			"Content-Type": "application/json"
		},
		body: JSON.stringify({ password: password })
	});
	if (result && result.ok) {
		showPopup("Password reset successful!", "success", "#password");
		setTimeout(() => {
			window.location.href = "log.html";
		}, 3000);
	}
	else {
		showPopup("An error occurred while trying to reset the password", "error");
	}
});