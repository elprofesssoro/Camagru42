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
		addPost(post.author, `../..${post.image_path}`, post.likes);
	});

	console.log(`Rendered Page ${currentPage}. Showing items ${startIndex} to ${endIndex - 1}`);
}

function addPost(author, imageSrc, likes) {
	const postHTML = `
        <section class="post">
            <h2>${author} shared</h2>
            <img src="${imageSrc}" class="imagePost">
			<section class="buttonPost">
				<button>
					<p id="countLikes">${likes}</p>
					<p id="textLikes">Like</p>
				</button>
				<form action="/url" method="POST">
					<input type="text" name="comment" placeholder="Enter your comment">
					<button type="submit">Comment</button>
				</form>
			</section>
        </section>
    `;
	grid.insertAdjacentHTML('beforeend', postHTML);
}

tempButton.addEventListener("click", populateButtons)


function populateButtons() {
	for (let i = 0; i < 200; i++) {
		addPost(`User ${i + 1}`, `../../pub/image1.jpg`, Math.floor(Math.random() * 100));
	}
	updatePagination();
}


async function getPosts() {
	const response = await callApi(`gallery?page=${currentPage}&per_page=${itemsPerPage}`, {
		method: "GET",
	});
	console.log(response);
	if (response && response.data && response.data.success) {
		console.log("Login successful");
	}

	return response.data;
}