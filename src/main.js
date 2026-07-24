const invoke = window.__TAURI__?.tauri?.invoke || window.__TAURI__?.invoke || window.__TAURI__?.core?.invoke;
const listen = window.__TAURI__?.event?.listen || window.__TAURI__?.listen;

let appState = null;
let activeTargetId = null;
let activeChannelId = null;
let activeGroup = null;

window.addEventListener('DOMContentLoaded', async () => {
  // 1. INITIALIZE & LISTEN FOR P2P NETWORK EVENTS
  try {
    if (invoke) {
      appState = await invoke('get_current_data');
      if (appState && appState.identity) {
        launchApp();
      }
    }

    if (listen) {
      await listen('p2p_event', async () => {
        if (invoke) {
          appState = await invoke('get_current_data');
          renderFriends();
          renderPendingRequests();
          renderMessages();
        }
      });
    }
  } catch (err) {
    console.error('Failed to load app state:', err);
  }

  // 2. AUTH TAB SWITCHING
  const tabCreateBtn = document.getElementById('tab-create-btn');
  const tabRecoverBtn = document.getElementById('tab-recover-btn');
  const createForm = document.getElementById('create-account-form');
  const recoverForm = document.getElementById('recover-account-form');

  tabCreateBtn.addEventListener('click', () => {
    tabCreateBtn.classList.add('active');
    tabRecoverBtn.classList.remove('active');
    createForm.classList.remove('hidden');
    recoverForm.classList.add('hidden');
  });

  tabRecoverBtn.addEventListener('click', () => {
    tabRecoverBtn.classList.add('active');
    tabCreateBtn.classList.remove('active');
    recoverForm.classList.remove('hidden');
    createForm.classList.add('hidden');
  });

  // 3. GENERATE ACCOUNT & SEED
  document.getElementById('gen-seed-btn').addEventListener('click', async () => {
    const displayName = document.getElementById('create-display-name').value.trim();
    if (!displayName) return alert('Please choose a Display Name.');

    try {
      appState = await invoke('create_account', { displayName });
      document.getElementById('generated-user-id').value = appState.identity.user_id;
      document.getElementById('generated-seed').value = appState.identity.seed_phrase;
      document.getElementById('seed-display-area').classList.remove('hidden');
    } catch (err) {
      alert('Account creation error: ' + err);
    }
  });

  document.getElementById('copy-user-id-btn').addEventListener('click', () => {
    const userIdInput = document.getElementById('generated-user-id');
    navigator.clipboard.writeText(userIdInput.value);
    alert('User ID copied to clipboard!');
  });

  document.getElementById('finish-create-btn').addEventListener('click', () => {
    launchApp();
  });

  // 4. RECOVER ACCOUNT
  document.getElementById('finish-recover-btn').addEventListener('click', async () => {
    const displayName = document.getElementById('recover-display-name').value.trim();
    const userId = document.getElementById('recover-user-id').value.trim();
    const seedPhrase = document.getElementById('recover-seed').value.trim();

    if (!displayName || !seedPhrase) return alert('Display Name and Seed Phrase are required.');

    try {
      appState = await invoke('recover_account', { displayName, userId, seedPhrase });
      launchApp();
    } catch (err) {
      alert('Account recovery error: ' + err);
    }
  });

  // 5. LAUNCH MAIN APPLICATION SHELL
  function launchApp() {
    document.getElementById('auth-modal').classList.add('hidden');
    document.getElementById('app-shell').classList.remove('hidden');

    document.getElementById('user-card-name').innerText = appState.identity.display_name;
    document.getElementById('user-card-id').innerText = appState.identity.user_id.slice(0, 12) + '...';

    renderFriends();
    renderPendingRequests();
    renderGroups();
  }

  // 6. DIRECT MESSAGES VIEW SWITCHER & SUB-TABS
  document.getElementById('btn-dm-view').addEventListener('click', () => {
    activeGroup = null;
    activeTargetId = null;
    activeChannelId = null;

    document.getElementById('btn-dm-view').classList.add('active-server');
    document.querySelectorAll('.group-list .server-icon').forEach(el => el.classList.remove('active-server'));

    document.getElementById('group-nav-section').classList.add('hidden');
    document.getElementById('dm-nav-section').classList.remove('hidden');
    document.getElementById('sidebar-title').innerText = 'Direct Messages';
    document.getElementById('chat-title').innerText = 'Select a Direct Message';

    renderMessages();
  });

  document.getElementById('nav-friends-btn').addEventListener('click', () => {
    document.getElementById('nav-friends-btn').classList.add('active');
    document.getElementById('nav-pending-btn').classList.remove('active');
    document.getElementById('friends-list').classList.remove('hidden');
    document.getElementById('pending-list').classList.add('hidden');
  });

  document.getElementById('nav-pending-btn').addEventListener('click', () => {
    document.getElementById('nav-pending-btn').classList.add('active');
    document.getElementById('nav-friends-btn').classList.remove('active');
    document.getElementById('pending-list').classList.remove('hidden');
    document.getElementById('friends-list').classList.add('hidden');
  });

  // 7. MODAL TOGGLES
  const toggleModal = (modalId, show) => {
    document.getElementById(modalId).classList.toggle('hidden', !show);
  };

  document.getElementById('btn-add-group').addEventListener('click', () => toggleModal('modal-create-group', true));
  document.getElementById('close-create-group').addEventListener('click', () => toggleModal('modal-create-group', false));

  document.getElementById('btn-join-group').addEventListener('click', () => toggleModal('modal-join-group', true));
  document.getElementById('close-join-group').addEventListener('click', () => toggleModal('modal-join-group', false));

  document.getElementById('nav-add-friend-btn').addEventListener('click', () => toggleModal('modal-add-friend', true));
  document.getElementById('close-add-friend').addEventListener('click', () => toggleModal('modal-add-friend', false));

  document.getElementById('btn-open-channel-modal').addEventListener('click', () => toggleModal('modal-create-channel', true));
  document.getElementById('close-create-channel').addEventListener('click', () => toggleModal('modal-create-channel', false));

  // 8. SETTINGS MODAL & ACCOUNT MANAGEMENT
  document.getElementById('btn-open-settings').addEventListener('click', () => {
    if (!appState || !appState.identity) return;

    document.getElementById('settings-display-name').textContent = appState.identity.display_name;
    document.getElementById('settings-user-id').value = appState.identity.user_id;
    document.getElementById('settings-seed-phrase').value = appState.identity.seed_phrase;

    const groupMgmt = document.getElementById('group-management-actions');
    const deleteGroupBtn = document.getElementById('btn-delete-group');

    if (activeGroup) {
      groupMgmt.classList.remove('hidden');
      const isOwner = activeGroup.owner_id === appState.identity.user_id;
      deleteGroupBtn.classList.toggle('hidden', !isOwner);
    } else {
      groupMgmt.classList.add('hidden');
    }

    toggleModal('modal-settings', true);
  });

  document.getElementById('close-settings').addEventListener('click', () => toggleModal('modal-settings', false));

  document.getElementById('btn-copy-settings-id').addEventListener('click', () => {
    navigator.clipboard.writeText(document.getElementById('settings-user-id').value);
    alert('User ID copied to clipboard!');
  });

  document.getElementById('btn-toggle-seed-vis').addEventListener('click', (e) => {
    const seedInput = document.getElementById('settings-seed-phrase');
    if (seedInput.type === 'password') {
      seedInput.type = 'text';
      e.target.textContent = 'Hide';
    } else {
      seedInput.type = 'password';
      e.target.textContent = 'Show';
    }
  });

  document.getElementById('btn-logout').addEventListener('click', async () => {
    if (!confirm('Are you sure you want to log out? Ensure you have saved your seed phrase.')) return;
    try {
      appState = await invoke('logout');
      location.reload();
    } catch (err) {
      alert('Error logging out: ' + err);
    }
  });

  document.getElementById('btn-leave-group').addEventListener('click', async () => {
    if (!activeGroup) return;
    try {
      appState = await invoke('leave_group', { groupId: activeGroup.id });
      activeGroup = null;
      activeTargetId = null;
      activeChannelId = null;
      renderGroups();
      toggleModal('modal-settings', false);
      document.getElementById('btn-dm-view').click();
    } catch (err) {
      alert('Error leaving group: ' + err);
    }
  });

  document.getElementById('btn-delete-group').addEventListener('click', async () => {
    if (!activeGroup) return;
    if (!confirm(`Are you sure you want to delete ${activeGroup.name}?`)) return;
    try {
      appState = await invoke('delete_group', { groupId: activeGroup.id });
      activeGroup = null;
      activeTargetId = null;
      activeChannelId = null;
      renderGroups();
      toggleModal('modal-settings', false);
      document.getElementById('btn-dm-view').click();
    } catch (err) {
      alert('Error deleting group: ' + err);
    }
  });

  // 9. CREATE GROUPCHAT
  document.getElementById('submit-create-group').addEventListener('click', async () => {
    const name = document.getElementById('input-group-name').value.trim();
    if (!name) return;

    try {
      appState = await invoke('create_group', { name });
      renderGroups();
      toggleModal('modal-create-group', false);
      document.getElementById('input-group-name').value = '';
    } catch (err) {
      alert('Error creating group: ' + err);
    }
  });

  // 10. JOIN GROUPCHAT
  document.getElementById('submit-join-group').addEventListener('click', async () => {
    const groupId = document.getElementById('input-join-group-id').value.trim();
    if (!groupId) return;

    try {
      appState = await invoke('join_group', { groupId });
      renderGroups();
      toggleModal('modal-join-group', false);
      document.getElementById('input-join-group-id').value = '';
    } catch (err) {
      alert('Error joining group: ' + err);
    }
  });

  // 11. SEND P2P FRIEND REQUEST
  document.getElementById('submit-add-friend').addEventListener('click', async () => {
    const friendId = document.getElementById('input-friend-id').value.trim();
    if (!friendId) return;

    try {
      appState = await invoke('send_friend_request', { targetId: friendId });
      alert('Friend request broadcasted across P2P mesh network.');
      toggleModal('modal-add-friend', false);
      document.getElementById('input-friend-id').value = '';
    } catch (err) {
      alert('Error sending friend request: ' + err);
    }
  });

  // 12. CREATE CHANNEL
  document.getElementById('submit-create-channel').addEventListener('click', async () => {
    const channelName = document.getElementById('input-channel-name').value.trim();
    const category = document.getElementById('input-channel-category').value.trim();

    if (!channelName || !activeGroup) return;

    try {
      appState = await invoke('create_channel', {
        groupId: activeGroup.id,
        channelName,
        category
      });

      activeGroup = appState.groups.find(g => g.id === activeGroup.id);
      renderChannels(activeGroup);
      toggleModal('modal-create-channel', false);
      document.getElementById('input-channel-name').value = '';
      document.getElementById('input-channel-category').value = '';
    } catch (err) {
      alert('Permission Error: ' + err);
    }
  });

  // 13. RENDER FRIENDS
  function renderFriends() {
    const friendsContainer = document.getElementById('friends-list');
    friendsContainer.innerHTML = '';

    (appState.friends || []).forEach(friend => {
      const item = document.createElement('div');
      item.className = 'list-item';
      if (activeTargetId === friend.user_id) item.classList.add('active');
      item.textContent = `💬 ${friend.display_name}`;

      item.onclick = () => {
        activeTargetId = friend.user_id;
        activeChannelId = null;

        document.querySelectorAll('#friends-list .list-item').forEach(el => el.classList.remove('active'));
        item.classList.add('active');

        document.getElementById('chat-title').innerText = `@ ${friend.display_name}`;
        renderMessages();
      };

      friendsContainer.appendChild(item);
    });
  }

  // 14. RENDER PENDING FRIEND REQUESTS
  function renderPendingRequests() {
    const pendingContainer = document.getElementById('pending-list');
    const badge = document.getElementById('pending-badge');
    pendingContainer.innerHTML = '';

    const requests = appState.pending_requests || [];
    if (requests.length > 0) {
      badge.textContent = requests.length;
      badge.classList.remove('hidden');
    } else {
      badge.classList.add('hidden');
    }

    requests.forEach(req => {
      const item = document.createElement('div');
      item.className = 'list-item pending-item';
      
      const text = document.createElement('span');
      text.textContent = `📩 ${req.sender_name}`;

      const acceptBtn = document.createElement('button');
      acceptBtn.className = 'small-btn success-btn';
      acceptBtn.textContent = 'Accept';

      acceptBtn.onclick = async (e) => {
        e.stopPropagation();
        try {
          appState = await invoke('accept_friend_request', { requestId: req.id });
          renderFriends();
          renderPendingRequests();
        } catch (err) {
          alert('Error accepting friend request: ' + err);
        }
      };

      item.appendChild(text);
      item.appendChild(acceptBtn);
      pendingContainer.appendChild(item);
    });
  }

  // 15. RENDER GROUPS
  function renderGroups() {
    const groupListContainer = document.getElementById('group-list');
    groupListContainer.innerHTML = '';

    (appState.groups || []).forEach(group => {
      const icon = document.createElement('div');
      icon.className = 'server-icon';
      if (activeGroup && activeGroup.id === group.id) icon.classList.add('active-server');

      icon.textContent = group.name.substring(0, 2).toUpperCase();
      icon.title = group.name;

      icon.onclick = () => {
        activeGroup = group;
        activeTargetId = group.id;
        activeChannelId = null;

        document.getElementById('btn-dm-view').classList.remove('active-server');
        document.querySelectorAll('.group-list .server-icon').forEach(el => el.classList.remove('active-server'));
        icon.classList.add('active-server');

        document.getElementById('dm-nav-section').classList.add('hidden');
        document.getElementById('group-nav-section').classList.remove('hidden');
        document.getElementById('sidebar-title').innerText = 'Channels';
        document.getElementById('active-group-name').innerText = group.name;

        document.getElementById('btn-copy-group-id').onclick = () => {
          navigator.clipboard.writeText(group.id);
          alert('Group ID copied: ' + group.id);
        };

        const isOwner = group.owner_id === appState.identity.user_id;
        document.getElementById('owner-channel-controls').classList.toggle('hidden', !isOwner);

        renderChannels(group);
      };

      groupListContainer.appendChild(icon);
    });
  }

  // 16. RENDER CHANNELS
  function renderChannels(group) {
    const channelListContainer = document.getElementById('channel-list');
    channelListContainer.innerHTML = '';

    const categories = {};
    (group.channels || []).forEach(ch => {
      const catKey = ch.category || 'TEXT CHANNELS';
      if (!categories[catKey]) categories[catKey] = [];
      categories[catKey].push(ch);
    });

    for (const [catName, channels] of Object.entries(categories)) {
      const catHeader = document.createElement('div');
      catHeader.className = 'category-header';
      catHeader.textContent = catName;
      channelListContainer.appendChild(catHeader);

      channels.forEach(ch => {
        const item = document.createElement('div');
        item.className = 'list-item';
        if (activeChannelId === ch.id) item.classList.add('active');

        item.textContent = `# ${ch.name}`;

        item.onclick = () => {
          activeChannelId = ch.id;
          document.querySelectorAll('#channel-list .list-item').forEach(el => el.classList.remove('active'));
          item.classList.add('active');

          document.getElementById('chat-title').innerText = `# ${ch.name}`;
          renderMessages();
        };

        channelListContainer.appendChild(item);
      });
    }

    if (group.channels && group.channels.length > 0 && !activeChannelId) {
      activeChannelId = group.channels[0].id;
      document.getElementById('chat-title').innerText = `# ${group.channels[0].name}`;
      renderMessages();
    }
  }

  // 17. RENDER MESSAGES (XSS-SAFE)
  function renderMessages() {
    const chatContainer = document.getElementById('chat-messages');
    chatContainer.innerHTML = '';

    const sysMsg = document.createElement('div');
    sysMsg.className = 'system-message';
    sysMsg.textContent = 'Welcome to Zero-Day Chat. All messages are encrypted peer-to-peer.';
    chatContainer.appendChild(sysMsg);

    if (!activeTargetId) return;

    const filtered = (appState.messages || []).filter(m => {
      if (activeGroup) {
        return m.target_id === activeTargetId && m.channel_id === activeChannelId;
      } else {
        return m.target_id === activeTargetId || m.sender_id === activeTargetId;
      }
    });

    filtered.forEach(msg => {
      const isMine = msg.sender_id === appState?.identity?.user_id;
      const card = document.createElement('div');
      card.className = `message-card ${isMine ? 'mine' : ''}`;

      const senderDiv = document.createElement('div');
      senderDiv.className = 'message-sender';
      senderDiv.textContent = msg.sender_name || 'Unknown';

      const contentDiv = document.createElement('div');
      contentDiv.className = 'message-content';
      contentDiv.textContent = msg.content || '';

      card.appendChild(senderDiv);
      card.appendChild(contentDiv);
      chatContainer.appendChild(card);
    });

    chatContainer.scrollTop = chatContainer.scrollHeight;
  }

  // 18. SEND MESSAGE
  const sendMessage = async () => {
    const input = document.getElementById('message-input');
    const content = input.value.trim();

    if (!content || !activeTargetId) return;

    try {
      appState = await invoke('send_message', {
        targetId: activeTargetId,
        channelId: activeChannelId,
        content
      });

      renderMessages();
      input.value = '';
    } catch (err) {
      alert('Error sending message: ' + err);
    }
  };

  document.getElementById('btn-send-message').addEventListener('click', sendMessage);
  document.getElementById('message-input').addEventListener('keypress', (e) => {
    if (e.key === 'Enter') sendMessage();
  });
});
