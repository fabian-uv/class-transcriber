const ALLOWED_EXTENSIONS = [
  "mp4", "mov", "m4v", "mkv", "webm", "avi",
  "mp3", "wav", "m4a", "aac", "flac", "ogg"
];

const translations = {
  en: {
    subtitle: "Local audio and video transcription",
    dropTitle: "Drop audio or video files here",
    dropDescription: "or choose multiple files from your computer",
    selectFiles: "Select files",
    queueTitle: "Transcription Queue",
    addFiles: "+ Add files",
    clearCompleted: "Clear completed",
    clearAll: "Clear all",
    transcribeAll: "Transcribe all",
    stopQueue: "Stop queue",
    stopping: "Stopping...",
    show: "Show in folder",
    privacy: "Your files stay on your device.",
    filesCount: (n) => `${n} file${n === 1 ? "" : "s"}`,
    transcribeQueueBtn: (n) => `Transcribe ${n} file${n === 1 ? "" : "s"}`,
    allDone: "All files have finished processing",
    queueStopped: "Queue processing was stopped",
    readyToTranscribe: "Ready in queue",
    itemDone: "Transcript ready",
    outputFolder: "Output folder:",
    sameAsSource: "Same as source",
    changeFolder: "Change",
    resetFolder: "Reset to default",
    mergeCheckbox: "Merge all into one .txt",
    mergeNow: "Merge into 1 TXT",
    showMerged: "Show merged TXT",
    mergedReady: "All transcripts merged into single file!",
    mergingWait: "Merging transcripts...",
    stages: {
      pending: "In queue",
      extracting: "Preparing audio...",
      transcribing: "Transcribing with Whisper...",
      saving: "Saving transcript...",
      done: "Completed",
      error: "Failed"
    }
  },

  es: {
    subtitle: "Transcripción local de audio y vídeo",
    dropTitle: "Arrastra archivos de audio o vídeo aquí",
    dropDescription: "o selecciona varios archivos desde tu equipo",
    selectFiles: "Seleccionar archivos",
    queueTitle: "Cola de transcripción",
    addFiles: "+ Agregar archivos",
    clearCompleted: "Limpiar completados",
    clearAll: "Vaciar cola",
    transcribeAll: "Transcribir cola",
    stopQueue: "Detener cola",
    stopping: "Deteniendo...",
    show: "Mostrar en carpeta",
    privacy: "Tus archivos permanecen en tu dispositivo.",
    filesCount: (n) => `${n} archivo${n === 1 ? "" : "s"}`,
    transcribeQueueBtn: (n) => `Transcribir ${n} archivo${n === 1 ? "" : "s"}`,
    allDone: "Procesamiento de cola completado",
    queueStopped: "Se detuvo el procesamiento de la cola",
    readyToTranscribe: "Listo en cola",
    itemDone: "Transcripción lista",
    outputFolder: "Carpeta de salida:",
    sameAsSource: "Misma del archivo",
    changeFolder: "Cambiar",
    resetFolder: "Restablecer",
    mergeCheckbox: "Juntar todas las transcripciones en un solo .txt",
    mergeNow: "Juntar en un solo TXT",
    showMerged: "Ver TXT combinado",
    mergedReady: "¡Transcripciones combinadas en un solo archivo!",
    mergingWait: "Combinando transcripciones...",
    stages: {
      pending: "En cola",
      extracting: "Preparando audio...",
      transcribing: "Transcribiendo con Whisper...",
      saving: "Guardando...",
      done: "Completado",
      error: "Falló"
    }
  }
};

let currentLanguage = "en";
let queue = [];
let isProcessing = false;
let stopRequested = false;
let lastOutputPath = null;
let mergedOutputPath = null;
let selectedOutputDir = null;

// DOM Elements
const subtitle = document.querySelector("#subtitle");
const dropZone = document.querySelector("#dropZone");
const dropTitle = document.querySelector("#dropTitle");
const dropDescription = document.querySelector("#dropDescription");
const selectButton = document.querySelector("#selectButton");

const outputFolderLabel = document.querySelector("#outputFolderLabel");
const outputFolderDisplay = document.querySelector("#outputFolderDisplay");
const changeFolderButton = document.querySelector("#changeFolderButton");
const resetFolderButton = document.querySelector("#resetFolderButton");
const mergeCheckbox = document.querySelector("#mergeCheckbox");
const mergeCheckboxText = document.querySelector("#mergeCheckboxText");

const queueSection = document.querySelector("#queueSection");
const queueTitle = document.querySelector("#queueTitle");
const queueBadge = document.querySelector("#queueBadge");
const queueList = document.querySelector("#queueList");
const addFilesButton = document.querySelector("#addFilesButton");
const clearCompletedButton = document.querySelector("#clearCompletedButton");
const clearAllButton = document.querySelector("#clearAllButton");

const progressSection = document.querySelector("#progressSection");
const progressBar = document.querySelector("#progressBar");
const progressValue = document.querySelector("#progressValue");
const statusText = document.querySelector("#statusText");

const startQueueButton = document.querySelector("#startQueueButton");
const mergeNowButton = document.querySelector("#mergeNowButton");
const stopQueueButton = document.querySelector("#stopQueueButton");
const showButton = document.querySelector("#showButton");
const showMergedButton = document.querySelector("#showMergedButton");
const privacyText = document.querySelector("#privacyText");
const languageButtons = document.querySelectorAll(".language-button");

function t() {
  return translations[currentLanguage];
}

function fileNameFromPath(path) {
  return path.split(/[\\/]/).pop();
}

function fileExtensionFromPath(path) {
  const parts = path.split(".");
  return parts.length > 1 ? parts.pop().toLowerCase() : "";
}

function updateLanguage() {
  const tr = t();

  subtitle.textContent = tr.subtitle;
  dropTitle.textContent = tr.dropTitle;
  dropDescription.textContent = tr.dropDescription;
  selectButton.textContent = tr.selectFiles;
  queueTitle.textContent = tr.queueTitle;
  addFilesButton.textContent = tr.addFiles;
  clearCompletedButton.textContent = tr.clearCompleted;
  clearAllButton.textContent = tr.clearAll;
  privacyText.textContent = tr.privacy;
  showButton.textContent = tr.show;

  outputFolderLabel.textContent = tr.outputFolder;
  changeFolderButton.textContent = tr.changeFolder;
  resetFolderButton.title = tr.resetFolder;
  mergeCheckboxText.textContent = tr.mergeCheckbox;
  mergeNowButton.textContent = tr.mergeNow;
  showMergedButton.textContent = tr.showMerged;

  updateFolderDisplay();

  languageButtons.forEach((button) => {
    button.classList.toggle("active", button.dataset.lang === currentLanguage);
  });

  renderQueue();
}

function updateFolderDisplay() {
  const tr = t();
  if (selectedOutputDir) {
    outputFolderDisplay.textContent = fileNameFromPath(selectedOutputDir) || selectedOutputDir;
    outputFolderDisplay.title = selectedOutputDir;
    resetFolderButton.classList.remove("hidden");
  } else {
    outputFolderDisplay.textContent = tr.sameAsSource;
    outputFolderDisplay.title = tr.sameAsSource;
    resetFolderButton.classList.add("hidden");
  }
}

languageButtons.forEach((button) => {
  button.addEventListener("click", () => {
    currentLanguage = button.dataset.lang;
    updateLanguage();
  });
});

changeFolderButton.addEventListener("click", async () => {
  if (isProcessing) return;
  try {
    const selected = await window.__TAURI__.dialog.open({
      directory: true,
      multiple: false
    });

    if (typeof selected === "string" && selected.trim().length > 0) {
      selectedOutputDir = selected;
      updateFolderDisplay();
    }
  } catch (error) {
    console.error("Folder picker error:", error);
  }
});

resetFolderButton.addEventListener("click", () => {
  if (isProcessing) return;
  selectedOutputDir = null;
  updateFolderDisplay();
});

function addFilesToQueue(paths) {
  if (!paths || paths.length === 0) return;

  const list = Array.isArray(paths) ? paths : [paths];
  let addedCount = 0;

  list.forEach((filePath) => {
    if (!filePath || typeof filePath !== "string") return;

    const ext = fileExtensionFromPath(filePath);
    if (!ALLOWED_EXTENSIONS.includes(ext)) {
      return;
    }

    const exists = queue.some((item) => item.path === filePath);
    if (!exists) {
      queue.push({
        id: "item-" + Date.now() + "-" + Math.random().toString(36).substring(2, 8),
        path: filePath,
        name: fileNameFromPath(filePath),
        status: "pending",
        outputPath: null,
        error: null
      });
      addedCount++;
    }
  });

  if (addedCount > 0) {
    renderQueue();
  }
}

// Creates an element with plain properties (className, title, textContent...)
// and optional children. Never parses HTML.
function el(tag, props = {}, children = []) {
  const node = document.createElement(tag);
  Object.assign(node, props);
  node.append(...children);
  return node;
}

function renderQueue() {
  const tr = t();
  const total = queue.length;

  if (total === 0) {
    dropZone.classList.remove("hidden");
    queueSection.classList.add("hidden");
    startQueueButton.classList.add("hidden");
    stopQueueButton.classList.add("hidden");
    mergeNowButton.classList.add("hidden");
    if (!isProcessing) {
      progressSection.classList.add("hidden");
    }
    return;
  }

  dropZone.classList.add("hidden");
  queueSection.classList.remove("hidden");

  const completedCount = queue.filter((i) => i.status === "done").length;
  const pendingCount = queue.filter((i) => i.status === "pending").length;
  const errorCount = queue.filter((i) => i.status === "error").length;

  queueBadge.textContent = `${completedCount}/${total} (${tr.filesCount(total)})`;

  if (completedCount > 0 && !isProcessing) {
    clearCompletedButton.classList.remove("hidden");
  } else {
    clearCompletedButton.classList.add("hidden");
  }

  clearAllButton.disabled = isProcessing;
  addFilesButton.disabled = isProcessing;
  changeFolderButton.disabled = isProcessing;
  resetFolderButton.disabled = isProcessing;

  if (isProcessing) {
    startQueueButton.classList.add("hidden");
    mergeNowButton.classList.add("hidden");
    stopQueueButton.classList.remove("hidden");
    stopQueueButton.textContent = stopRequested ? tr.stopping : tr.stopQueue;
    stopQueueButton.disabled = stopRequested;
  } else {
    stopQueueButton.classList.add("hidden");
    if (pendingCount > 0 || errorCount > 0) {
      startQueueButton.classList.remove("hidden");
      startQueueButton.textContent = tr.transcribeQueueBtn(pendingCount || errorCount);
      startQueueButton.disabled = false;
    } else {
      startQueueButton.classList.add("hidden");
    }

    if (completedCount >= 2) {
      mergeNowButton.classList.remove("hidden");
    } else {
      mergeNowButton.classList.add("hidden");
    }
  }

  if (lastOutputPath) {
    showButton.classList.remove("hidden");
  } else {
    showButton.classList.add("hidden");
  }

  if (mergedOutputPath) {
    showMergedButton.classList.remove("hidden");
  } else {
    showMergedButton.classList.add("hidden");
  }

  // Render list items
  queueList.replaceChildren();
  queue.forEach((item, index) => {
    const itemEl = document.createElement("div");
    const isInProgress = item.status === "extracting" || item.status === "transcribing";
    itemEl.className = `queue-item status-${item.status} ${isInProgress ? "in-progress" : ""}`;

    let iconText = "⏳";
    let statusLabel = tr.stages.pending;
    let subtitleText = tr.readyToTranscribe;

    if (item.status === "extracting") {
      iconText = "⚙️";
      statusLabel = tr.stages.extracting;
      subtitleText = tr.stages.extracting;
    } else if (item.status === "transcribing") {
      iconText = "⚡";
      statusLabel = tr.stages.transcribing;
      subtitleText = tr.stages.transcribing;
    } else if (item.status === "done") {
      iconText = "✓";
      statusLabel = tr.stages.done;
      subtitleText = item.outputPath ? fileNameFromPath(item.outputPath) : tr.itemDone;
    } else if (item.status === "error") {
      iconText = "⚠️";
      statusLabel = tr.stages.error;
      subtitleText = item.error ? item.error : tr.stages.error;
    }

    // Built with textContent/title, never innerHTML: file names, paths and
    // FFmpeg/whisper error messages come from outside and may contain markup.
    const copyEl = el("div", { className: "queue-item-copy" }, [
      el("span", {
        className: "queue-item-name",
        title: item.path,
        textContent: `${index + 1}. ${item.name}`,
      }),
      el("span", {
        className: "queue-item-subtitle",
        title: subtitleText,
        textContent: subtitleText,
      }),
    ]);

    const actionsEl = el("div", { className: "queue-item-actions" });

    if (item.status === "done" && item.outputPath) {
      const folderBtn = el("button", {
        className: "icon-btn btn-folder",
        title: tr.show,
        textContent: "📁",
      });
      folderBtn.type = "button";
      folderBtn.dataset.id = item.id;
      folderBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        openFolderForPath(item.outputPath);
      });
      actionsEl.append(folderBtn);
    }

    if (!isInProgress && !isProcessing) {
      const removeBtn = el("button", {
        className: "icon-btn btn-remove",
        title: "Remove",
        textContent: "✕",
      });
      removeBtn.type = "button";
      removeBtn.dataset.id = item.id;
      removeBtn.addEventListener("click", (e) => {
        e.stopPropagation();
        removeQueueItem(item.id);
      });
      actionsEl.append(removeBtn);
    }

    itemEl.append(
      el("div", { className: "queue-item-icon", textContent: iconText }),
      copyEl,
      el("span", { className: "queue-item-badge", textContent: statusLabel }),
      actionsEl
    );

    queueList.appendChild(itemEl);
  });
}

function removeQueueItem(id) {
  if (isProcessing) return;
  queue = queue.filter((i) => i.id !== id);
  renderQueue();
}

clearCompletedButton.addEventListener("click", () => {
  if (isProcessing) return;
  queue = queue.filter((i) => i.status !== "done");
  renderQueue();
});

clearAllButton.addEventListener("click", () => {
  if (isProcessing) return;
  queue = [];
  lastOutputPath = null;
  mergedOutputPath = null;
  progressSection.classList.add("hidden");
  renderQueue();
});

async function chooseFiles() {
  try {
    const selected = await window.__TAURI__.dialog.open({
      multiple: true,
      directory: false,
      filters: [
        {
          name: "Audio & Video",
          extensions: ALLOWED_EXTENSIONS
        }
      ]
    });

    if (Array.isArray(selected)) {
      addFilesToQueue(selected);
    } else if (typeof selected === "string") {
      addFilesToQueue([selected]);
    }
  } catch (error) {
    console.error("File picker error:", error);
  }
}

selectButton.addEventListener("click", chooseFiles);
addFilesButton.addEventListener("click", chooseFiles);

async function initializeDragAndDrop() {
  try {
    const appWindow = window.__TAURI__.webviewWindow.getCurrentWebviewWindow();

    await appWindow.onDragDropEvent((event) => {
      if (event.payload.type === "over") {
        dropZone.classList.add("dragging");
        queueSection.classList.add("dragging");
      }

      if (event.payload.type === "leave") {
        dropZone.classList.remove("dragging");
        queueSection.classList.remove("dragging");
      }

      if (event.payload.type === "drop") {
        dropZone.classList.remove("dragging");
        queueSection.classList.remove("dragging");
        const paths = event.payload.paths;
        if (paths && paths.length > 0) {
          addFilesToQueue(paths);
        }
      }
    });
  } catch (error) {
    console.error("Drag and drop error:", error);
  }
}

function setProgress(value) {
  const safeValue = Math.max(0, Math.min(value, 100));
  progressBar.style.width = `${safeValue}%`;
  progressValue.textContent = `${Math.round(safeValue)}%`;
}

async function mergeCompletedTranscripts() {
  const completedItems = queue.filter((i) => i.status === "done" && i.outputPath);
  if (completedItems.length === 0) return;

  statusText.textContent = t().mergingWait;

  try {
    const files = completedItems.map((i) => i.outputPath);
    const result = await window.__TAURI__.core.invoke("merge_transcripts", {
      files,
      outputDir: selectedOutputDir || null,
      combinedName: "Transcripciones_Combinadas"
    });

    mergedOutputPath = result;
    statusText.textContent = t().mergedReady;
    showMergedButton.classList.remove("hidden");
    renderQueue();
    return result;
  } catch (err) {
    console.error("Merge error:", err);
    statusText.textContent = `Merge failed: ${err}`;
  }
}

mergeNowButton.addEventListener("click", async () => {
  if (isProcessing) return;
  mergeNowButton.disabled = true;
  await mergeCompletedTranscripts();
  mergeNowButton.disabled = false;
});

async function startQueueProcessing() {
  if (isProcessing) return;

  const pendingItems = queue.filter((i) => i.status === "pending" || i.status === "error");
  if (pendingItems.length === 0) return;

  pendingItems.forEach((i) => {
    i.status = "pending";
    i.error = null;
  });

  isProcessing = true;
  stopRequested = false;
  progressSection.classList.remove("hidden");
  renderQueue();

  const total = queue.length;

  for (let i = 0; i < queue.length; i++) {
    if (stopRequested) break;

    const item = queue[i];
    if (item.status !== "pending") continue;

    const completedSoFar = queue.filter((q) => q.status === "done").length;
    const currentPercent = Math.round((completedSoFar / total) * 100);
    setProgress(currentPercent);

    statusText.textContent = `[${completedSoFar + 1}/${total}] ${item.name} - ${t().stages.extracting}`;
    item.status = "extracting";
    renderQueue();

    const activeEl = queueList.children[i];
    if (activeEl) {
      activeEl.scrollIntoView({ behavior: "smooth", block: "nearest" });
    }

    const timer = setTimeout(() => {
      if (item.status === "extracting" && isProcessing) {
        item.status = "transcribing";
        statusText.textContent = `[${completedSoFar + 1}/${total}] ${item.name} - ${t().stages.transcribing}`;
        renderQueue();
      }
    }, 1200);

    try {
      const resultPath = await window.__TAURI__.core.invoke("transcribe", {
        videoPath: item.path,
        outputDir: selectedOutputDir || null
      });

      clearTimeout(timer);
      item.status = "done";
      item.outputPath = resultPath;
      lastOutputPath = resultPath;
    } catch (err) {
      clearTimeout(timer);
      console.error(`Error transcribing ${item.name}:`, err);
      item.status = "error";
      item.error = String(err);
    }

    const newCompleted = queue.filter((q) => q.status === "done").length;
    setProgress(Math.round((newCompleted / total) * 100));
    renderQueue();
  }

  isProcessing = false;
  const finalDone = queue.filter((q) => q.status === "done").length;
  setProgress(Math.round((finalDone / total) * 100));

  if (stopRequested) {
    statusText.textContent = t().queueStopped;
  } else {
    statusText.textContent = t().allDone;

    // Check if user requested auto-merging all transcripts into one TXT
    if (mergeCheckbox.checked && finalDone >= 2) {
      await mergeCompletedTranscripts();
    }
  }

  stopRequested = false;
  renderQueue();
}

startQueueButton.addEventListener("click", startQueueProcessing);

stopQueueButton.addEventListener("click", () => {
  if (!isProcessing) return;
  stopRequested = true;
  stopQueueButton.textContent = t().stopping;
  stopQueueButton.disabled = true;
  statusText.textContent = t().stopping;
});

async function openFolderForPath(path) {
  if (!path) return;
  try {
    await window.__TAURI__.core.invoke("show_in_folder", { path });
  } catch (error) {
    console.error("Show in folder error:", error);
  }
}

showButton.addEventListener("click", () => {
  if (lastOutputPath) {
    openFolderForPath(lastOutputPath);
  }
});

showMergedButton.addEventListener("click", () => {
  if (mergedOutputPath) {
    openFolderForPath(mergedOutputPath);
  }
});

updateLanguage();
initializeDragAndDrop();