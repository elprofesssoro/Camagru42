async function updateNavigation() {


	const loginBtn = document.querySelector("#nav-login");
	const logoutBtn = document.querySelector("#nav-logout");
	const createBtn = document.querySelector("#nav-create");

	if (isLoggedIn) {
		if (loginBtn) loginBtn.classList.add("hidden");
		if (logoutBtn) logoutBtn.classList.remove("hidden");
		if (createBtn) createBtn.classList.remove("hidden");
	} else {
		if (loginBtn) loginBtn.classList.remove("hidden");
		if (logoutBtn) logoutBtn.classList.add("hidden");
		if (createBtn) createBtn.classList.add("hidden");
	}
}

window.addEventListener("pageshow", async () => {
	await initializeAuth();
	updateNavigation();

	const logoutBtn = document.querySelector("#nav-logout");
	if (logoutBtn) {
		logoutBtn.addEventListener("click", async () => {
			const response = await callApi("logout", { method: "POST" });

			if (response && response.ok) {
				window.location.href = "log.html";
			}
		});
	}
});

const pageSelect = document.querySelector("#page-select");
const grid = document.querySelector(".posts-grid");
const prevButton = document.querySelector("#pages button:first-of-type");
const nextButton = document.querySelector("#pages button:last-of-type");
const pageInfo = document.querySelector("#pages p");

const tempButton = document.querySelector("#temp");

let currentPage = 1;
let itemsPerPage = parseInt(pageSelect.value);
let totalPages = 0;

updatePagination();

pageSelect.addEventListener("change", (e) => {
	itemsPerPage = parseInt(e.target.value);
	currentPage = 1;

	updatePagination();
});

prevButton.addEventListener("click", () => {
	if (currentPage > 1) {
		currentPage--;
		updatePagination();
	}
});

nextButton.addEventListener("click", () => {
	if (currentPage >= totalPages) {
		nextButton.disabled = true;
		return;
	}
	else {
		currentPage++;
		updatePagination();
	}
});

grid.addEventListener("click", async (e) => {
	const likeButton = e.target.closest(".like-btn");

	if (!likeButton || !grid.contains(likeButton)) {
		return;
	}

	const postId = likeButton.closest('.post')?.dataset.postId;
	if (!postId) postId = 0;
	if (!getAuthStatus()) {
		showPopup("You must be logged in to like a post.", "error", ".post[data-post-id='" + postId + "'] form");
		return;
	}
	const response = await callApi(
		'gallery/like?post_id=' + postId,
		{ method: 'POST' }
	);
	if (response && response.status === 401) {
		showPopup("You must be logged in to like a post.", "error", ".post[data-post-id='" + postId + "'] form");
		return;
	}
	if (response.ok) {
		const likesCounter = likeButton.querySelector('.count-likes');
		const currentLikes = Number(likesCounter?.textContent) || 0;
		if (!likesCounter) return ;
		if (response.status === 200) {
			likesCounter.textContent = String(Math.max(0, currentLikes - 1));
			likeButton.classList.remove("liked");
			return;
		}
		else if (response.status === 201) {
			likesCounter.textContent = String(currentLikes + 1);
			likeButton.classList.add("liked");
			return;
		}

	}
	else {
		showPopup("Failed to like the post. Please try again.", "error", ".post[data-post-id='" + postId + "'] form");
	}
});

grid.addEventListener("submit", async (e) => {
	e.preventDefault();
	const commentForm = e.target.closest("form");

	if (!commentForm || !grid.contains(commentForm)) {
		return;
	}

	const postId = commentForm.closest('.post')?.dataset.postId;
	if (!postId) return;

	if (!getAuthStatus()) {
		showPopup("You must be logged in to comment.", "error", ".post[data-post-id='" + postId + "'] form");
		return;
	}
	const commentInput = commentForm.querySelector('input[name="comment"]');
	const commentText = commentInput ? commentInput.value : "";

	if (commentText.trim() === "") {
		return;
	}

	const userId = 0;
	const response = await callApi(
		'gallery/comment?post_id=' + postId,
		{
			method: 'POST',
			headers: {
				"Content-Type": "application/json"
			},
			body: JSON.stringify({ comment: commentText })
		}
	);
	if (response && response.status === 401) {
		showPopup("You must be logged in to comment.", "error", ".post[data-post-id='" + postId + "'] form");
		return;
	}
	if (response.ok) {
		commentInput.value = "";
		showPopup("Comment added successfully!", "ok", ".post[data-post-id='" + postId + "'] form");
	}
	else {
		showPopup("Failed to add comment. Please try again.", "error", ".post[data-post-id='" + postId + "'] form");
	}

});

async function updatePagination() {
	const resposnse = await getPosts();
	const posts = resposnse.posts || [];
	totalPages = resposnse.total_posts > 0 ? Math.ceil(resposnse.total_posts / itemsPerPage) : 1;

	pageInfo.textContent = `Page ${currentPage} out of ${totalPages}`;

	prevButton.disabled = currentPage === 1;
	nextButton.disabled = currentPage === totalPages;

	const startIndex = (currentPage - 1) * itemsPerPage;
	const endIndex = startIndex + itemsPerPage;

	grid.innerHTML = "";

	posts.forEach((post, index) => {
		addPost(post.author, `/pub/posts/${post.img_name}`, post.likes, post.post_id);
	});

}

function addPost(author, imageSrc, likes, postId) {
	const postHTML =
		'<section class="post" data-post-id="' + postId + '">' +
		'<h2>' + author + ' shared</h2>' +
		'<img src="' + imageSrc + '" class="imagePost">' +
		'<section class="buttonPost">' +
		'<button type="button" class="like-btn">' +
		'<p class="count-likes">' + likes + '</p>' +
		'<p class="text-likes">Like</p>' +
		'</button>' +
		'<form action="/url" method="POST">' +
		'<input type="text" name="comment" placeholder="Enter your comment">' +
		'<button type="submit">Comment</button>' +
		'</form>' +
		'</section>' +
		'</section>';

	grid.insertAdjacentHTML('beforeend', postHTML);
}


async function getPosts() {

	const response = await callApi(`gallery?page=${currentPage}&per_page=${itemsPerPage}`, {
		method: "GET",
	});

	if (response) {
		console.log(response);
	}

	return response.data;
}