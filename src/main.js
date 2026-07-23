// Application State
const state = {
  username: "",
  peerId: "",
  publicKey: "",
  seedPhrase: [],
  currentChannel: "general"
};

// Tauri IPC Invoker Helper
async function invokeTauri(command, args = {}) {
  if (window.__TAURI__ && window.__TAURI__.core) {
    return await window.__TAURI__.core.invoke(command, args);
  } else {
    console.warn(`[Browser Fallback] Invoking '${command}'`, args);
    return mockTauriCalls(command, args);
  }
}

// Fallback logic for browser testing prior to Tauri compilation
function mockTauriCalls(command, args) {
  if (command === "generate_identity") {
    const mockWords = ["alpha", "bravo", "cipher", "delta", "echo", "foxtrot", "matrix", "node", "quantum", "shadow", "signal", "tunnel"];
    return Promise.resolve({
      peer_id: "peer_" + Math.random().toString(16).substring(2, 10),
      public_key: "pk_" + Math.random().toString(16).substring(2, 10),
      mnemonic: mockWords.join(" ")
    });
  }
  if (command === "send_peer_message") {
    return Promise.resolve({
      id: "msg_" + Date.now(),
      channel_id: args.channelId,
      sender_id: args.senderId,
      payload: args.content,
      timestamp: Math.floor(Date.now() / 1000)
    });
  }
  return Promise.resolve(null);
}

// DOM UI Switcher
function switchScreen(screenId) {
  document.querySelectorAll(".screen").forEach(s => s.classList.remove("active"));
  document.getElementById(screenId).classList.add("active");
}

function switchPanel(panelId) {
  document.querySelectorAll(".content-panel").forEach(p => p.classList.remove("active"));
  document.getElementById(panelId).classList.add("active");
}

// Initialize Application Events
document.addEventListener("DOMContentLoaded", () => {

  // 1. Splash Screen Transition
  document.getElementById("btn-start").addEventListener("click", () => {
    switchScreen("auth-screen");
  });

  // 2. Account Generation (Calling Rust identity module)
  document.getElementById("btn-generate-seed").addEventListener("click", async () => {
    const usernameInput = document.getElementById("username").value.trim();
    if (!usernameInput) {
      alert("Please enter a valid handle.");
      return;
    }

    try {
      const identity = await invokeTauri("generate_identity");
      
      state.username = usernameInput;
      state.peerId = identity.peer_id;
      state.publicKey = identity.public_key;
      state.seedPhrase = identity.mnemonic.split(" ");

      renderSeedPhrase(state.seedPhrase);

      document.getElementById("auth-step-1").classList.remove("active");
      document.getElementById("auth-step-2").classList.add("active");
    } catch (err) {
      console.error("Identity generation error:", err);
      alert("Failed to generate identity.");
    }
  });

  // 3. Enter Main Workspace
  document.getElementById("btn-enter-app").addEventListener("click", () => {
    document.getElementById("display-username").textContent = `@${state.username}`;
    document.getElementById("display-peer-id").textContent = state.peerId;
    switchScreen("app-screen");
  });

  // Navigation Panel Switching
  document.getElementById("tab-friends").addEventListener("click", (e) => {
    setActiveNav(e.target);
    switchPanel("panel-friends");
  });

  document.getElementById("tab-dms").addEventListener("click", (e) => {
    setActiveNav(e.target);
    switchPanel("panel-chat");
  });

  document.querySelectorAll(".channel-item").forEach(btn => {
    btn.addEventListener("click", (e) => {
      document.querySelectorAll(".channel-item").forEach(c => c.classList.remove("active"));
      e.target.classList.add("active");
      
      state.currentChannel = e.target.dataset.channel;
      document.getElementById("current-channel-title").textContent = `# ${state.currentChannel}`;
      switchPanel("panel-chat");
    });
  });

  // Chat Actions
  document.getElementById("btn-send-msg").addEventListener("click", sendMessage);
  document.getElementById("chat-input").addEventListener("keypress", (e) => {
    if (e.key === "Enter") sendMessage();
  });
});

function setActiveNav(element) {
  document.querySelectorAll(".nav-item").forEach(n => n.classList.remove("active"));
  element.classList.add("active");
}

function renderSeedPhrase(words) {
  const container = document.getElementById("seed-display");
  container.innerHTML = "";
  words.forEach((word, idx) => {
    const div = document.createElement("div");
    div.className = "seed-word";
    div.textContent = `${idx + 1}. ${word}`;
    container.appendChild(div);
  });
}

async function sendMessage() {
  const input = document.getElementById("chat-input");
  const text = input.value.trim();
  if (!text) return;

  try {
    const msg = await invokeTauri("send_peer_message", {
      channelId: state.currentChannel,
      content: text,
      senderId: state.username
    });

    appendMessageToUI(msg.sender_id, msg.payload, new Date(msg.timestamp * 1000));
    input.value = "";
  } catch (err) {
    console.error("Message error:", err);
  }
}

function appendMessageToUI(author, text, date) {
  const container = document.getElementById("chat-messages");
  const msgEl = document.createElement("div");
  msgEl.className = "message";

  const timeStr = date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });

  msgEl.innerHTML = `
    <span class="msg-author">${escapeHTML(author)}</span>
    <span class="msg-time">${timeStr}</span>
    <p class="msg-body">${escapeHTML(text)}</p>
  `;

  container.appendChild(msgEl);
  container.scrollTop = container.scrollHeight;
}

function escapeHTML(str) {
  return str.replace(/[&<>'"]/g, 
    tag => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', "'": '&#39;', '"': '&quot;' }[tag] || tag)
  );
}
