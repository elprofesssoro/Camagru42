const resendBtn = document.querySelector("#resend-btn");

resendBtn.disabled = true;
startCountdown(resendBtn, 60);

document.querySelector("#resend-form").addEventListener("submit", resendEmail);

async function resendEmail(event) {
	event.preventDefault();
	const email = document.querySelector("#resend-email").value;

	const emailResult = validEmail(email);
	if (emailResult !== "1") {
		showPopup(emailResult, "error");
		return;
	}

	resendBtn.disabled = true;
	startCountdown(resendBtn, 60);
	
	const response = await callApi("re-email", {
		method: "POST",
		headers: {
			"Content-Type": "application/json"
		},
		body: JSON.stringify({ email })
	});
	if (response && response.ok) {
		console.log("Verification email resent");
		showPopup("Verification email resent!", "success", "#resend-btn");
	} else if (response.status === 404) {
		showPopup("No account was found with that email", "error", "#resend-btn");
	} else if (response.status === 409) {
		showPopup("Account is already verified", "error", "#resend-btn");
	} else {
		showPopup("Failed to resend verification email", "error", "#resend-btn");
	}
}

function startCountdown(button, seconds) {
	const originalText = button.textContent;
	button.textContent = `${originalText} (${seconds}s)`;
	button.style.cursor = "not-allowed";
	button.style.opacity = "0.7";

	button.style.transform = "none";
	button.style.boxShadow = "none";

	const interval = setInterval(() => {
		seconds--;
		button.textContent = `${originalText} (${seconds}s)`;

		if (seconds <= 0) {
			clearInterval(interval);
			button.textContent = originalText;
			button.disabled = false;

			button.style.cursor = "";
			button.style.opacity = "";
			button.style.transform = "";
			button.style.boxShadow = "";
		}
	}, 1000);
}