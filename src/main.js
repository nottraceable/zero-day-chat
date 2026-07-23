const { invoke } = window.__TAURI__.tauri;

document.addEventListener("DOMContentLoaded", async () => {
  const usernameInput = document.getElementById("username-input");
  const btnNextSeed = document.getElementById("btn-next-seed");
  const btnFinish = document.getElementById("btn-finish-onboarding");
  const seedDisplay = document.getElementById("seed-display");
  
  let currentUsername = "";
  let servers = [];
  let friends = [];
  let activeView = { type: "dm", id: null };
  let messagesStore = {};

  btnNextSeed.addEventListener("click", async () => {
    currentUsername = usernameInput.value.trim();
    if (!currentUsername) {
      alert("Please enter a valid handle.");
      return;
    }

    try {
      const identity = await invoke("create_identity", { username: currentUsername });
      seedDisplay.innerHTML = "";
      identity.seed_phrase.split(" ").forEach(word => {
        const span = document.createElement("div");
        span.className = "seed-word";
        span.textContent = word;
        seedDisplay.appendChild(span);
      });

      document.getElementById("step-username").classList.remove("active");
      document.getElementById("step-seed").classList.add("active");
    } catch (err) {
      alert("Error generating identity: " + err);
    }
  });

  btnFinish.addEventListener("click", () => {
    document.getElementById("onboarding-container").classList.remove("active");
    document.getElementById("app-container").classList.add("active");
    
    document.getElementById("display-username").textContent = currentUsername;
    document.getElementById("user-initial").textContent = currentUsername.charAt(0).toUpperCase();
    
    renderSidebar();
  });

  // Navigation & Sidebar Logic
  document.getElementById("nav-dms").addEventListener("click", () => {
    activeView = { type: "dm", id: null };
    document.getElementById("current-context-title").textContent = "Direct Messages";
    renderSidebar();
    clearChatArea();
  });

  document.getElementById("btn-add-friend").addEventListener("click", () => {
    openModal("Add Friend by Peer ID", "Peer ID or PublicKey...", (val) => {
      if (val) {
        friends.push({ id: val, name: val.substring(0, 8) + "..." });
        renderSidebar();
      }
    });
  });

  document.getElementById("btn-create-server").addEventListener("click", () => {
    openModal("Create Server", "Server Name...", (serverName) => {
      if (serverName) {
        const newServer = {
          id: Date.now().toString(),
          name: serverName,
          initial: serverName.charAt(0).toUpperCase(),
          channels: [
            { id: "general", name: "general", category: "Text Channels" }
          ]
        };
        servers.push(newServer);
        renderServerIcons();
      }
    });
  });

  function renderServerIcons() {
    const list = document.getElementById("server-list");
    list.innerHTML = "";
    servers.forEach(srv => {
      const div = document.createElement("div");
      div.className = "server-icon";
      div.textContent = srv.initial;
      div.title = srv.name;
      div.addEventListener("click", () => {
        activeView = { type: "server", id: srv.id, channelId: srv.channels[0].id };
        document.getElementById("current-context-title").textContent = srv.name;
        renderSidebar();
        renderChatMessages();
      });
      list.appendChild(div);
    });
  }

  function renderSidebar() {
    const channelsList = document.getElementById("channels-list");
    channelsList.innerHTML = "";

    if (activeView.type === "dm") {
      const header = document.createElement("div");
      header.className = "category-header";
      header.textContent = "Direct Messages";
      channelsList.appendChild(header);

      if (friends.length === 0) {
        const empty = document.createElement("div");
        empty.className = "channel-item";
        empty.style.color = "var(--text-muted)";
        empty.textContent = "No peers added yet.";
        channelsList.appendChild(empty);
      } else {
        friends.forEach(f => {
          const item = document.createElement("div");
          item.className = "friend-item";
          if (activeView.id === f.id) item.style.background = "rgba(255,255,255,0.05)";
          item.textContent = f.name;
          item.addEventListener("click", () => {
            activeView = { type: "dm", id: f.id };
            document.getElementById("active-chat-title").textContent = f.name;
            renderChatMessages();
          });
          channelsList.appendChild(item);
        });
      }
    } else if (activeView.type === "server") {
      const srv = servers.find(s => s.id === activeView.id);
      if (srv) {
        const header = document.createElement("div");
        header.className = "category-header";
        header.textContent = "Text Channels";
        channelsList.appendChild(header);

        srv.channels.forEach(ch => {
          const item = document.createElement("div");
          item.className = "channel-item";
          if (activeView.channelId === ch.id) item.style.background = "rgba(255,255,255,0.05)";
          item.textContent = "# " + ch.name;
          item.addEventListener("click", () => {
            activeView.channelId = ch.id;
            document.getElementById("active-chat-title").textContent = "# " + ch.name;
            renderSidebar();
            renderChatMessages();
          });
          channelsList.appendChild(item);
        });
      }
    }
  }

  function clearChatArea() {
    document.getElementById("active-chat-title").textContent = "Select a conversation or server channel";
    document.getElementById("messages-container").innerHTML = `
      <div class="welcome-message">
        <h3>Secure Channel Ready</h3>
        <p>End-to-end encrypted connection established across local nodes.</p>
      </div>
    `;
  }

  function renderChatMessages() {
    const container = document.getElementById("messages-container");
    container.innerHTML = "";
    const key = activeView.type === "dm" ? `dm_${activeView.id}` : `srv_${activeView.id}_${activeView.channelId}`;
    const msgs = messagesStore[key] || [];

    if (msgs.length === 0) {
      container.innerHTML = `<div class="welcome-message"><p>Beginning of encrypted transcript.</p></div>`;
      return;
    }

    msgs.forEach(m => {
      const bubble = document.createElement("div");
      bubble.className = "message-bubble";
      bubble.innerHTML = `<div class="message-author">${m.author}</div><div class="message-text">${m.text}</div>`;
      container.appendChild(bubble);
    });
    container.scrollTop = container.scrollHeight;
  }

  // Sending Messages
  document.getElementById("btn-send-message").addEventListener("click", sendMessage);
  document.getElementById("message-input").addEventListener("keypress", (e) => {
    if (e.key === "Enter") sendMessage();
  });

  function sendMessage() {
    const input = document.getElementById("message-input");
    const text = input.value.trim();
    if (!text) return;

    const key = activeView.type === "dm" ? `dm_${activeView.id}` : `srv_${activeView.id}_${activeView.channelId}`;
    if (!messagesStore[key]) messagesStore[key] = [];

    messagesStore[key].push({ author: currentUsername, text });
    input.value = "";
    renderChatMessages();
  }

  // Modal Utility
  function openModal(title, placeholder, callback) {
    document.getElementById("modal-title").textContent = title;
    const input = document.getElementById("modal-input");
    input.value = "";
    input.placeholder = placeholder;
    const overlay = document.getElementById("modal-overlay");
    overlay.classList.remove("hidden");

    const submitBtn = document.getElementById("modal-submit");
    const cancelBtn = document.getElementById("modal-cancel");

    const cleanup = () => {
      overlay.classList.add("hidden");
      submitBtn.replaceWith(submitBtn.cloneNode(true));
      cancelBtn.replaceWith(cancelBtn.cloneNode(true));
    };

    document.getElementById("modal-submit").addEventListener("click", () => {
      const val = input.value.trim();
      callback(val);
      cleanup();
    }, { once: true });

    document.getElementById("modal-cancel").addEventListener("click", () => {
      cleanup();
    }, { once: true });
  }
});
