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

updateHistory();
activateCamera();

document.querySelector(".creations-list").addEventListener("click", (e) => {
	if (!e.target.classList.contains("delete-btn")) {
		return;
	}
	const creation = e.target.closest(".creation");
	const postId = creation.dataset.postId;

	callApi("create/delete?post_id=" + postId, { method: "DELETE" })
		.then((response) => {
			if (response && response.ok) {
				creation.remove();
				showPopup("Creation deleted successfully!", "ok");
			} else {
				showPopup("Failed to delete creation.", "error");
			}
		})
		.catch((error) => {
			console.error("Error deleting creation:", error);
			showPopup("Failed to delete creation.", "error");
		});

});

function updateHistory() {
	callApi("create/history").then((response) => {
		if (response && response.ok && Array.isArray(response.data)) {
			console.log("History loaded successfully:", response.data);
			const historyContainer = document.querySelector(".creations-list");
			if (!historyContainer) {
				console.error("History container not found");
				return;
			}
			historyContainer.innerHTML = "";
			response.data.forEach((item) => {
				const creationHTML = `
					<div class="creation" data-post-id="${item.post_id}">
						<img src="/pub/posts/${item.img_name}" alt="${item.img_name}">
						<button class="delete-btn">Delete</button>
					</div>
				`;
				historyContainer.insertAdjacentHTML('beforeend', creationHTML);
			});
		} else {
			console.error("Failed to load history:", response);
		}
	}).catch((error) => {
		console.error("Error fetching history:", error);
	});
}

function activateCamera() {
	navigator.mediaDevices.getUserMedia({ video: true }).then((stream) => {
		webcam.srcObject = stream;
		webcam.style.display = "block";
		startFrameBuffering();
		captureButton.disabled = captureLocked || !webcam.srcObject;
	}).catch((error) => {
		console.error("Error accessing webcam:", error);
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
	if (!currentSticker || !isUploaded) {
		showPopup("Add an image and a sticker before posting.", "error");
		return;
	}

	const postImg = overlay.querySelector('.post img');
	if (!postImg) {
		showPopup("No image found to post.", "error");
		return;
	}

	const imageRect = postImg.getBoundingClientRect();
	const stickerRect = currentSticker.getBoundingClientRect();

	if (!imageRect.width || !imageRect.height) {
		showPopup("Image is not ready yet.", "error");
		return;
	}

	const naturalWidth = postImg.naturalWidth || Math.round(imageRect.width);
	const naturalHeight = postImg.naturalHeight || Math.round(imageRect.height);
	const scaleX = naturalWidth / imageRect.width;
	const scaleY = naturalHeight / imageRect.height;

	const payload = {
		image: "",
		sticker_name: currentSticker.src.split('/').pop().split('.').slice(0, -1).join('.') + "." + currentSticker.src.split('/').pop().split('.').pop(),
		pos_x: Math.max(0, Math.round((stickerRect.left - imageRect.left) * scaleX)),
		pos_y: Math.max(0, Math.round((stickerRect.top - imageRect.top) * scaleY)),
		width: Math.max(1, Math.round(stickerRect.width * scaleX)),
		height: Math.max(1, Math.round(stickerRect.height * scaleY))
	};

	postButton.disabled = true;

	toDataUrl(postImg).then((imageDataUrl) => {
		payload.image = imageDataUrl;
		return callApi("create/post", {
			method: "POST",
			headers: {
				"Content-Type": "application/json"
			},
			body: JSON.stringify(payload)
		});
	}).then((response) => {
		if (response && response.ok) {
			showPopup("Image posted successfully!", "ok");
		} else {
			showPopup("Failed to post image.", "error");
			postButton.disabled = !(isStickered && isUploaded);
		}
		updateHistory();
	}).catch((error) => {
		console.error("Failed to post image", error);
		showPopup("Failed to post image.", "error");
		postButton.disabled = !(isStickered && isUploaded);
	});
}

function toDataUrl(img) {
	if (typeof img.src === "string" && img.src.startsWith("data:image/")) {
		return Promise.resolve(img.src);
	}

	return new Promise((resolve, reject) => {
		const convert = () => {
			try {
				const canvas = document.createElement("canvas");
				canvas.width = img.naturalWidth || img.width;
				canvas.height = img.naturalHeight || img.height;
				const ctx = canvas.getContext("2d");
				ctx.drawImage(img, 0, 0);
				resolve(canvas.toDataURL("image/png"));
			} catch (err) {
				reject(err);
			}
		};

		if (img.complete) {
			convert();
			return;
		}

		img.addEventListener("load", convert, { once: true });
		img.addEventListener("error", () => reject(new Error("Image failed to load")), { once: true });
	});
}

const overlay = document.querySelector("#overlay-layer");

let currentSticker = null;
let isDragging = false;
let offsetX = 0;
let offsetY = 0;

function loadStickers() {
	fetch('/pub/stickers/')
		.then(response => {
			if (!response.ok) throw new Error('Network response was not ok');
			return response.json();
		})
		.then(files => {
			const filterList = document.querySelector(".filter-list");
			if (!filterList) return;
			
			filterList.innerHTML = ""; // Clear statically hardcoded filters
			
			files.forEach((file, index) => {
				// Ensure it's a file, not a directory
				if (file.type !== "file") return;
				
				const filename = file.name;
				
				const filterItem = `
					<label class="filter-item">
						<input type="radio" name="filter" value="sticker-${index}">
						<img src="/pub/stickers/${filename}" alt="${filename}">
					</label>
				`;
				filterList.insertAdjacentHTML('beforeend', filterItem);
			});

			bindFilterListeners();
		})
		.catch(error => console.error("Error fetching stickers:", error));
}

function bindFilterListeners() {
	const filters = document.querySelectorAll('input[name="filter"]');
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
}

// Call on load
loadStickers();

function startDrag(e) {
	e.preventDefault();
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

	let newX = e.clientX - containerRect.left - offsetX;
	let newY = e.clientY - containerRect.top - offsetY;

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