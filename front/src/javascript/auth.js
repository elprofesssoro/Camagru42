let isLoggedIn = false;
let currentUser = null;
let hasCheckedServer = false;

async function initializeAuth() {
	try {
		if (hasCheckedServer)
			return isLoggedIn;

		const response = await callApi("me", { method: "GET", cache: "no-store" });
		if (response.ok) {
			isLoggedIn = true;
			currentUser = await response.data;
		} else {
			isLoggedIn = false;
			currentUser = null;
		}
	} catch (error) {
		console.error("Auth check failed:", error);
		isLoggedIn = false;
		const response = await callApi("logout", { method: "POST" });
		if (!response.ok) {
			console.error("Failed to clear auth on server:", response);
		}
	} finally {
		hasCheckedServer = true;
	}

	return isLoggedIn;
}

function getAuthStatus() {
	return isLoggedIn;
}


function clearLocalAuth() {
	isLoggedIn = false;
	currentUser = null;
	hasCheckedServer = false;
}

async function checkAuthStatus() {
	const response = await callApi("me", { method: "GET", cache: "no-store" });
	return response && response.ok;
}