const postButton = document.querySelector("#post-btn");
const fileInput = document.querySelector("#upload-file");
const resetButton = document.querySelector("#reset-btn");
const webcam = document.querySelector("#video-feed");
const captureButton = document.querySelector("#capture-btn");

fileInput.addEventListener("change", uploadImage);
postButton.addEventListener("click", postImage);
resetButton.addEventListener("click", resetImage);
captureButton.addEventListener("click", captureImage);

let isUploaded = false;
let isStickered = false;
let hasCapturedWithSticker = false;
let captureLocked = false;

const currentFrameCanvas = document.createElement('canvas');
const previousFrameCanvas = document.createElement('canvas');
const currentFrameCtx = currentFrameCanvas.getContext('2d');
const previousFrameCtx = previousFrameCanvas.getContext('2d');
let frameLoopStarted = false;

activateCamera();

function activateCamera() {
	navigator.mediaDevices.getUserMedia({ video: true }).then((stream) => {
		webcam.srcObject = stream;
		webcam.style.display = "block";
		startFrameBuffering();
		captureButton.disabled = captureLocked || !webcam.srcObject;
	}).catch((error) => {
		console.error("Error accessinCag webcam:", error);
	});
}

function startFrameBuffering() {
	if (frameLoopStarted) return;
	frameLoopStarted = true;

	const tick = () => {
		if (webcam.videoWidth > 0 && webcam.videoHeight > 0 && webcam.srcObject) {
			if (currentFrameCanvas.width !== webcam.videoWidth || currentFrameCanvas.height !== webcam.videoHeight) {
				currentFrameCanvas.width = webcam.videoWidth;
				currentFrameCanvas.height = webcam.videoHeight;
				previousFrameCanvas.width = webcam.videoWidth;
				previousFrameCanvas.height = webcam.videoHeight;
			}

			previousFrameCtx.drawImage(currentFrameCanvas, 0, 0);
			currentFrameCtx.drawImage(webcam, 0, 0, currentFrameCanvas.width, currentFrameCanvas.height);
		}

		if (typeof webcam.requestVideoFrameCallback === 'function') {
			webcam.requestVideoFrameCallback(() => tick());
		} else {
			requestAnimationFrame(tick);
		}
	};

	tick();
}

function uploadImage(event) {
	clearEditorState();
	hasCapturedWithSticker = false;
	captureLocked = false;
	postButton.disabled = true;
	captureButton.disabled = true;

	const file = event.target.files[0];
	if (!file) return;

	if (webcam.srcObject) {
		const tracks = webcam.srcObject.getTracks();
		tracks.forEach(track => track.stop());
		webcam.srcObject = null;
	}

	webcam.style.display = "none";

	const imageUrl = URL.createObjectURL(file);
	const postHTML = `
        <div class="post">
            <img src="${imageUrl}" alt="Uploaded Image" style="max-width: 100%;">
        </div>
    `;
	document.querySelector("#overlay-layer").insertAdjacentHTML('afterbegin', postHTML);
	isUploaded = true;
	postButton.disabled = !(isStickered && isUploaded);
}

function clearEditorState() {
	overlay.innerHTML = "";
	if (currentSticker) {
		currentSticker.remove();
		currentSticker = null;
	}
	isStickered = false;
	isUploaded = false;
	hasCapturedWithSticker = false;
}

function resetImage() {
	clearEditorState();
	captureLocked = false;
	postButton.disabled = true;
	captureButton.disabled = false;
	if (webcam.srcObject) {
		const tracks = webcam.srcObject.getTracks();
		tracks.forEach(track => track.stop());
		webcam.srcObject = null;
	}
	activateCamera();
}

function captureImage(event) {
	if (captureLocked) return;
	if (!currentSticker) return;
	if (!previousFrameCanvas.width || !previousFrameCanvas.height) return;

	const oldPost = overlay.querySelector('.post');
	if (oldPost) oldPost.remove();

	const imageUrl = previousFrameCanvas.toDataURL('image/png');
	const postHTML = `
        <div class="post">
            <img src="${imageUrl}" alt="Captured Image" style="max-width: 100%;">
        </div>
    `;
	overlay.insertAdjacentHTML('afterbegin', postHTML);
	isUploaded = true;
	hasCapturedWithSticker = true;
	captureLocked = true;
	postButton.disabled = false;
	captureButton.disabled = true;
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
		hasCapturedWithSticker = false;
		postButton.disabled = !(isStickered && isUploaded);

		currentSticker.addEventListener('mousedown', startDrag);
	});
});

function startDrag(e) {
	e.preventDefault(); // Prevent default image dragging behavior
	isDragging = true;
	currentSticker.style.cursor = 'grabbing';
	captureButton.disabled = true;
	postButton.disabled = true;
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
	captureButton.disabled = captureLocked || !webcam.srcObject;
	hasCapturedWithSticker = false;
	postButton.disabled = !(isStickered && isUploaded);
	document.removeEventListener('mousemove', drag);
	document.removeEventListener('mouseup', endDrag);
}


const creations = document.querySelectorAll(".creation");

creations.forEach((creation) => {
	creation.addEventListener("click", () => {
		console.log(creation.children[0].alt);
	});
});