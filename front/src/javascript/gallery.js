const pageSelect = document.querySelector("#page-select");
const grid = document.querySelector(".posts-grid");
// To Change
const posts = Array.from(document.querySelectorAll(".post"));

const prevButton = document.querySelector("#pages button:first-of-type");
const nextButton = document.querySelector("#pages button:last-of-type");
const pageInfo = document.querySelector("#pages p");

const tempButton = document.querySelector("#temp");

let currentPage = 1;
let itemsPerPage = parseInt(pageSelect.value);

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
	const totalPages = Math.ceil(posts.length / itemsPerPage);
	if (currentPage >= totalPages) {
		nextButton.disabled = true;
		return;
	}
	else {
		currentPage++;
		updatePagination();
	}
});

function updatePagination() {
	const totalPages = Math.ceil(posts.length / itemsPerPage) || 1;

	pageInfo.textContent = `Page ${currentPage} out of ${totalPages}`;

	prevButton.disabled = currentPage === 1;
	nextButton.disabled = currentPage === totalPages;

	const startIndex = (currentPage - 1) * itemsPerPage;
	const endIndex = startIndex + itemsPerPage;

	posts.forEach((post, index) => {
		if (index >= startIndex && index < endIndex) {
			post.style.display = "block";
		} else {
			post.style.display = "none";
		}
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
	posts.push(grid.lastElementChild);
}

tempButton.addEventListener("click", populateButtons)


function populateButtons() {
	for (let i = 0; i < 200; i++) {
		addPost(`User ${i + 1}`, `../../pub/image1.jpg`, Math.floor(Math.random() * 100));
	}
	updatePagination();
}