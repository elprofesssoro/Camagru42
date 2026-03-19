const postButton = document.querySelector("#post-btn");
const fileInput = document.querySelector("#upload-file");
const resetButton = document.querySelector("#reset-btn");
const webcam = document.querySelector("#video-feed");

fileInput.addEventListener("change", uploadImage);
postButton.addEventListener("click", postImage);
resetButton.addEventListener("click", resetImage);


let isUploaded = false;
let isStickered = false;

activateCamera();

function activateCamera() {
	navigator.mediaDevices.getUserMedia({ video: true }).then((stream) => {
		webcam.srcObject = stream;
	}).catch((error) => {
		console.error("Error accessing webcam:", error);
	});
}

function uploadImage(event) {
	resetImage()
	const file = event.target.files[0];
	if (!file) return;

	if (webcam.srcObject) {
		const tracks = webcam.srcObject.getTracks();
		tracks.forEach(track => track.stop());
		webcam.srcObject = null;
	}

	const imageUrl = URL.createObjectURL(file);
	const postHTML = `
        <div class="post">
            <img src="${imageUrl}" alt="Uploaded Image" style="max-width: 100%;">
        </div>
    `;
	document.querySelector("#overlay-layer").insertAdjacentHTML('afterbegin', postHTML);
	isUploaded = true;
}

function resetImage() {
	const overlay = document.querySelector("#overlay-layer");
	overlay.innerHTML = "";
	if (currentSticker) {
		currentSticker.remove();
		currentSticker = null;
	}
	postButton.disabled = true;
	isStickered = false;
	isUploaded = false;
}

function postImage() {
	if (!currentSticker)
		return;
}

const filters = document.querySelectorAll('input[name="filter"]');
const overlay = document.querySelector("#overlay-layer");

let currentSticker = null;
let isDragging = false;
let offsetX = 0;
let offsetY = 0;

filters.forEach(radio => {
	radio.addEventListener('change', (e) => {
		if (currentSticker && currentSticker.parentElement) {
			currentSticker.remove();
		}

		const imgSrc = e.target.nextElementSibling.src;
		currentSticker = document.createElement('img');
		currentSticker.src = imgSrc;
		currentSticker.draggable = false;
		currentSticker.style.position = 'absolute';
		currentSticker.style.left = '50px';
		currentSticker.style.top = '50px';
		currentSticker.style.cursor = 'grab';
		currentSticker.style.maxWidth = '150px';
		currentSticker.style.zIndex = '100';

		overlay.appendChild(currentSticker);


		isStickered = true;
		if (isStickered && isUploaded) postButton.disabled = false;
		else postButton.disabled = true;

		currentSticker.addEventListener('mousedown', startDrag);
	});
});

function startDrag(e) {
	e.preventDefault(); // Prevent default image dragging behavior
	isDragging = true;
	currentSticker.style.cursor = 'grabbing';

	const rect = currentSticker.getBoundingClientRect();
	offsetX = e.clientX - rect.left;
	offsetY = e.clientY - rect.top;

	document.addEventListener('mousemove', drag);
	document.addEventListener('mouseup', endDrag);
}

function drag(e) {
	if (!isDragging) return;

	const containerRect = overlay.getBoundingClientRect();

	// Calculate new position relative to the overlay container
	let newX = e.clientX - containerRect.left - offsetX;
	let newY = e.clientY - containerRect.top - offsetY;

	// Apply new position
	currentSticker.style.left = `${newX}px`;
	currentSticker.style.top = `${newY}px`;
}

function endDrag() {
	isDragging = false;
	if (currentSticker) currentSticker.style.cursor = 'grab';
	document.removeEventListener('mousemove', drag);
	document.removeEventListener('mouseup', endDrag);
}