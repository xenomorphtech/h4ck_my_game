let scenarios = [];
let selected = null;
let socket = null;
let renderer = null;
let combat = null;
let completed = new Set();
let actionSessionStarted = false;
// Client-side mirrors of the server's authoritative market/mail state. They are
// only ever refreshed from the scenario payload or from server notification
// events in a run result -- never mutated speculatively by a click handler.
let liveMarket = null;
let liveMail = null;
let liveInventory = null;

const $ = (id) => document.getElementById(id);

async function loadScenarios() {
  initConsoleTabs();
  renderer = new window.SceneRenderer($('scene-canvas'));
  combat = new window.CombatController(renderer, {
    hud: $('combat-hud'),
    playerHp: $('player-hp-text'),
    monsterHp: $('monster-hp-text'),
    playerBar: $('player-hp-bar'),
    monsterBar: $('monster-hp-bar'),
    death: $('death-overlay'),
    deathText: $('death-text'),
    revive: $('revive'),
    popup: $('complete-popup'),
    popupTitle: $('complete-title'),
    popupText: $('complete-text'),
    popupClose: $('complete-close'),
    status: $('scene-status'),
  });
  renderer.onAction = (action) => {
    // Clicking a monster plays like a normal game: it sends the real attack
    // packet to the server. The server's run result drives combat feedback.
    const combatPacket = combat.packetForAction(action);
    if (combatPacket) {
      sendPacketScript(combatPacket);
      return;
    }
    if (combat.handleSceneAction(action)) return;
    if (action.kind === 'move') {
      $('scene-status').textContent = `moving to ${action.x}, ${action.y}`;
    } else if (action.kind === 'attack') {
      $('scene-status').textContent = `attacked ${action.target}; target is aggro`;
    } else if (action.kind === 'cooldown') {
      $('scene-status').textContent = `attack cooling down — wait ${action.wait_ms}ms`;
    } else if (action.kind === 'blocked') {
      $('scene-status').textContent = `blocked tile ${action.x}, ${action.y}: ${action.reason}`;
    }
  };
  const [scenarioPayload] = await Promise.all([
    fetch('/api/scenarios').then((r) => r.json()),
    loadProgress(),
  ]);
  scenarios = scenarioPayload;
  const list = $('scenario-list');
  list.innerHTML = '';
  for (const scenario of scenarios) {
    const button = document.createElement('button');
    button.className = 'scenario';
    button.dataset.id = scenario.id;
    button.innerHTML = `<strong>${scenario.title}</strong><small>${scenario.category} · ${scenario.difficulty}</small>`;
    button.onclick = () => selectScenario(scenario);
    list.appendChild(button);
  }
  renderCompletedBadges();
  if (scenarios[0]) selectScenario(scenarios[0]);
}

async function loadProgress() {
  const progress = await fetch('/api/progress').then((r) => r.json());
  completed = new Set(progress.completed || []);
  renderCompletedBadges();
  return progress;
}

function renderCompletedBadges() {
  document.querySelectorAll('.scenario').forEach((el) => {
    const done = completed.has(el.dataset.id);
    const scenario = scenarios.find((item) => item.id === el.dataset.id);
    const label = scenario?.title || 'Puzzle';
    el.classList.toggle('completed', done);
    el.setAttribute('aria-label', done ? `${label} completed` : label);
  });
}

function selectScenario(scenario) {
  selected = scenario;
  actionSessionStarted = false;
  document.querySelectorAll('.scenario').forEach((el) => {
    el.classList.toggle('active', el.dataset.id === scenario.id);
  });
  renderCompletedBadges();
  $('scenario-title').textContent = scenario.title;
  $('scenario-meta').textContent = `${scenario.category} · ${scenario.difficulty}`;
  $('objective').textContent = scenario.objective;
  $('packets').textContent = (scenario.packets || []).join('\n');
  $('script').value = scenario.example_script || '';
  $('selected-id').textContent = scenario.title;
  $('result').textContent = '';
  $('events').innerHTML = '';
  $('lesson').classList.add('hidden');
  $('lesson-text').textContent = '';
  $('scene-status').textContent = `Loaded ${scenario.title}`;
  renderer.setScene(scenario.scene);
  combat.setScenario(scenario);
  renderSystemViews(scenario);
  renderInventory(scenario.inventory || []);
  activateConsoleTab('script-tab');
}

function renderSystemViews(scenario) {
  renderMarket(scenario.market || null);
  renderMail(scenario.mail || null);
  renderSkills(scenario.skills || null);
}

function iconNode(sprite, label) {
  const iconWrap = document.createElement('div');
  iconWrap.className = 'inventory-icon';
  const url = window.gameIconUrl ? window.gameIconUrl(sprite) : null;
  if (url) {
    const img = document.createElement('img');
    img.src = url;
    img.alt = '';
    img.loading = 'lazy';
    img.onerror = () => {
      img.remove();
      iconWrap.textContent = label.slice(0, 2).toUpperCase();
    };
    iconWrap.appendChild(img);
  } else {
    iconWrap.textContent = label.slice(0, 2).toUpperCase();
  }
  return iconWrap;
}

function renderMarket(market) {
  const panel = $('market-panel');
  const root = $('market-listings');
  root.innerHTML = '';
  liveMarket = market ? JSON.parse(JSON.stringify(market)) : null;
  panel.classList.toggle('hidden', !market);
  if (!market) {
    $('market-gold').textContent = '';
    return;
  }

  const listings = market.listings || [];
  $('market-gold').textContent = `${market.gold} gold · ${listings.length} listing${listings.length === 1 ? '' : 's'}`;
  if (!listings.length) {
    const empty = document.createElement('div');
    empty.className = 'inventory-empty';
    empty.textContent = 'No visible market listings.';
    root.appendChild(empty);
    return;
  }

  for (const listing of listings) {
    const card = document.createElement('article');
    card.className = `listing-card market-card status-${listing.status || 'active'}`;
    const text = document.createElement('div');
    text.className = 'listing-text';
    const title = document.createElement('strong');
    title.textContent = listing.item;
    const meta = document.createElement('small');
    const status = listing.status ? `${listing.status} · ` : '';
    meta.textContent = `listing #${listing.id} · ${status}stock ${listing.stock} · ${listing.note}`;
    text.append(title, meta);

    if (listing.cancel_packet) {
      const cancel = document.createElement('button');
      cancel.type = 'button';
      cancel.className = 'listing-cancel';
      cancel.textContent = 'Cancel listing';
      // Showcase the packet through gameplay: send it to the server and let the
      // server-side notifications (ListingRemoved / MailCreated) update the UI.
      cancel.onclick = () => sendPacketScript(`send ${listing.cancel_packet}`);
      text.appendChild(cancel);
    }

    const price = document.createElement('span');
    price.className = 'listing-price';
    price.textContent = `${listing.price}g`;

    card.append(iconNode(listing.sprite, listing.item), text, price);
    root.appendChild(card);
  }
}

// Add a packet line to the script editor for actions that are suggestions
// rather than immediate world interactions.
function appendScriptLine(line) {
  const editor = $('script');
  const current = editor.value.replace(/\s*$/, '');
  editor.value = current ? `${current}\n${line}\n` : `${line}\n`;
  editor.focus();
  editor.scrollTop = editor.scrollHeight;
  $('scene-status').textContent = `added: ${line}`;
}

function renderMail(mail) {
  const panel = $('mail-panel');
  const root = $('mail-messages');
  root.innerHTML = '';
  liveMail = mail ? JSON.parse(JSON.stringify(mail)) : null;
  panel.classList.toggle('hidden', !mail);
  if (!mail) {
    $('mail-count').textContent = '';
    return;
  }

  const messages = mail.messages || [];
  $('mail-count').textContent = `${messages.length} message${messages.length === 1 ? '' : 's'}`;
  if (!messages.length) {
    const empty = document.createElement('div');
    empty.className = 'inventory-empty';
    empty.textContent = 'No visible mail.';
    root.appendChild(empty);
    return;
  }

  for (const message of messages) {
    const card = document.createElement('article');
    card.className = 'listing-card mail-card';
    const text = document.createElement('div');
    text.className = 'listing-text';
    const title = document.createElement('strong');
    title.textContent = message.subject;
    const meta = document.createElement('small');
    meta.textContent = `mail #${message.id} · ${message.status}`;
    text.append(title, meta);

    const attachment = document.createElement('span');
    attachment.className = 'listing-price';
    attachment.textContent = message.attachment;

    if (isClaimableMail(message)) {
      const claim = document.createElement('button');
      claim.type = 'button';
      claim.className = 'mail-claim';
      claim.textContent = 'Claim';
      claim.onclick = () => sendPacketScript(`send ClaimMailbox { mail: ${message.id} }`);
      text.appendChild(claim);
    }

    card.append(iconNode(message.sprite, message.subject), text, attachment);
    root.appendChild(card);
  }
}

function isClaimableMail(message) {
  return Number.isFinite(Number(message.id))
    && message.status !== 'claimed'
    && message.status !== 'draft'
    && message.attachment
    && message.attachment !== 'empty';
}

function renderSkills(skills) {
  const panel = $('skill-panel');
  const root = $('skills');
  root.innerHTML = '';
  panel.classList.toggle('hidden', !skills);
  if (!skills) {
    $('skill-count').textContent = '';
    return;
  }

  const actions = skills.actions || [];
  $('skill-count').textContent = `${actions.length} skill${actions.length === 1 ? '' : 's'}`;
  if (!actions.length) {
    const empty = document.createElement('div');
    empty.className = 'inventory-empty';
    empty.textContent = 'No visible skills.';
    root.appendChild(empty);
    return;
  }

  for (const skill of actions) {
    const card = document.createElement('article');
    card.className = 'skill-card';

    const text = document.createElement('div');
    text.className = 'skill-text';
    const title = document.createElement('strong');
    title.textContent = skill.name;
    const description = document.createElement('small');
    description.textContent = skill.description;
    text.append(title, description);

    const cast = document.createElement('button');
    cast.type = 'button';
    cast.className = 'skill-cast';
    cast.textContent = 'Cast';
    cast.onclick = () => appendScriptLine(`send ${skill.cast_packet}`);

    card.append(iconNode(skill.sprite, skill.name), text, cast);
    root.appendChild(card);
  }
}

function renderInventory(items) {
  liveInventory = items ? JSON.parse(JSON.stringify(items)) : [];
  renderInventoryView(liveInventory);
}

function renderInventoryView(items) {
  const root = $('inventory');
  root.innerHTML = '';
  $('inventory-count').textContent = `${items.length} slot${items.length === 1 ? '' : 's'}`;

  if (!items.length) {
    const empty = document.createElement('div');
    empty.className = 'inventory-empty';
    empty.textContent = 'No inventory data for this puzzle.';
    root.appendChild(empty);
    return;
  }

  for (const item of items) {
    const slot = document.createElement('article');
    slot.className = `inventory-slot ${item.quantity <= 0 ? 'empty-slot' : ''}`;

    const iconWrap = iconNode(item.sprite, item.name);

    const text = document.createElement('div');
    text.className = 'inventory-text';
    const name = document.createElement('strong');
    name.textContent = item.name;
    const meta = document.createElement('small');
    meta.textContent = item.slot;
    text.append(name, meta);

    const qty = document.createElement('span');
    qty.className = 'inventory-qty';
    qty.textContent = `×${item.quantity}`;

    slot.append(iconWrap, text, qty);
    root.appendChild(slot);
  }
}

function initConsoleTabs() {
  const tabs = Array.from(document.querySelectorAll('.console-tab'));
  tabs.forEach((tab) => {
    tab.onclick = () => activateConsoleTab(tab.dataset.tab);
  });
}

function activateConsoleTab(name) {
  document.querySelectorAll('.console-tab').forEach((tab) => {
    const active = tab.dataset.tab === name;
    tab.classList.toggle('active', active);
    tab.setAttribute('aria-selected', active ? 'true' : 'false');
  });
  document.querySelectorAll('.console-pane').forEach((pane) => {
    const active = pane.dataset.pane === name;
    pane.classList.toggle('active', active);
    pane.hidden = !active;
  });
}

function ensureSocket() {
  if (socket && socket.readyState <= WebSocket.OPEN) return socket;
  const proto = location.protocol === 'https:' ? 'wss:' : 'ws:';
  socket = new WebSocket(`${proto}//${location.host}/ws`);
  socket.onopen = () => $('connection').textContent = 'connected';
  socket.onclose = () => $('connection').textContent = 'disconnected';
  socket.onerror = () => $('connection').textContent = 'socket error';
  socket.onmessage = (event) => renderResult(JSON.parse(event.data));
  return socket;
}

function runSelected() {
  if (!selected) return;
  sendScript($('script').value, 'running packets…', false);
  actionSessionStarted = true;
}

function sendPacketScript(line) {
  sendScript(`${line}\n`, `sent packet: ${line}`, actionSessionStarted);
  actionSessionStarted = true;
}

function sendScript(script, statusText, append = false) {
  if (!selected) return;
  $('scene-status').textContent = statusText;
  const ws = ensureSocket();
  const send = () => ws.send(JSON.stringify({
    type: 'run_script',
    scenario_id: selected.id,
    script,
    append,
  }));
  if (ws.readyState === WebSocket.OPEN) send();
  else ws.addEventListener('open', send, { once: true });
}

function renderResult(result) {
  if (result.outcome === 'win') {
    completed.add(result.scenario_id);
    renderCompletedBadges();
    loadProgress().catch(() => {});
  }

  $('result').className = result.outcome;
  $('result').textContent = JSON.stringify({
    ok: result.ok,
    outcome: result.outcome,
    time_ms: result.time_ms,
    state: visibleRunState(result.state),
    error: result.error,
  }, null, 2);

  $('scene-status').textContent = result.outcome === 'win'
    ? 'flag captured'
    : result.outcome === 'lose'
      ? 'objective failed — inspect packets and try again'
      : 'script error';

  const events = $('events');
  events.innerHTML = '';
  for (const packet of result.events || []) {
    const div = document.createElement('div');
    div.className = `event ${packet.kind}`;
    div.textContent = `[${packet.t}ms] ${packet.kind} ${packet.name} ${JSON.stringify(packet.fields)}`;
    events.appendChild(div);
  }

  rebaseSystemViews();
  applyServerNotifications(result.events || []);
  renderer.playEvents(result.events || [], result.outcome);
  combat.applyRunResult(result);

  if (result.outcome === 'win' && result.state && result.state.lesson) {
    $('lesson-text').textContent = result.state.lesson;
    $('lesson').classList.remove('hidden');
  } else {
    $('lesson').classList.add('hidden');
    $('lesson-text').textContent = '';
  }
}

function visibleRunState(state) {
  if (!state || typeof state !== 'object' || Array.isArray(state)) return state;
  const out = { ...state };
  delete out.scenario_id;
  delete out.internal_id;
  delete out.bug_slug;
  return out;
}

function applyServerNotifications(events) {
  let marketChanged = false;
  let mailChanged = false;
  let inventoryChanged = false;

  for (const packet of events) {
    if (packet.kind !== 'server') continue;
    const fields = packet.fields || {};
    if (packet.name === 'ListingRemoved' && liveMarket) {
      const listingId = Number(fields.listing);
      liveMarket.listings = (liveMarket.listings || []).filter((listing) => Number(listing.id) !== listingId);
      marketChanged = true;
    } else if (packet.name === 'MailCreated' && liveMail) {
      const mailId = Number(fields.mail);
      const messages = liveMail.messages || [];
      if (!messages.some((message) => Number(message.id) === mailId)) {
        messages.push({
          id: mailId,
          subject: fields.subject || 'Listing cancelled',
          attachment: fields.attachment || 'Returned item',
          sprite: fields.sprite || 'mailbox',
          status: fields.status || 'unclaimed',
        });
      }
      liveMail.messages = messages;
      mailChanged = true;
    } else if (packet.name === 'MailClaimed' && liveMail) {
      const mailId = Number(fields.mail);
      for (const message of liveMail.messages || []) {
        if (Number(message.id) === mailId) {
          message.status = fields.status || 'claimed';
        }
      }
      mailChanged = true;
    } else if (packet.name === 'InventoryAdded') {
      if (!liveInventory) liveInventory = selected ? JSON.parse(JSON.stringify(selected.inventory || [])) : [];
      addInventoryItem(fields);
      inventoryChanged = true;
    }
  }

  if (marketChanged) renderMarket(liveMarket);
  if (mailChanged) renderMail(liveMail);
  if (inventoryChanged) renderInventoryView(liveInventory);
}

function rebaseSystemViews() {
  if (!selected) return;
  liveMarket = selected.market ? JSON.parse(JSON.stringify(selected.market)) : null;
  liveMail = selected.mail ? JSON.parse(JSON.stringify(selected.mail)) : null;
  liveInventory = JSON.parse(JSON.stringify(selected.inventory || []));
  renderSystemViews({ market: liveMarket, mail: liveMail, skills: selected.skills || null });
  renderInventoryView(liveInventory);
}

function addInventoryItem(fields) {
  const name = fields.item || fields.name || 'Item';
  const quantity = Number(fields.quantity ?? fields.count ?? 1) || 1;
  const existing = (liveInventory || []).find((item) => item.name === name);
  if (existing) {
    existing.quantity = Number(existing.quantity || 0) + quantity;
    if (fields.slot) existing.slot = fields.slot;
    if (fields.sprite) existing.sprite = fields.sprite;
    return;
  }
  liveInventory.push({
    name,
    sprite: fields.sprite || 'mailbox',
    quantity,
    slot: fields.slot || 'bag',
  });
}

function resetExample() {
  if (!selected) return;
  actionSessionStarted = false;
  $('script').value = selected.example_script || '';
  $('result').textContent = '';
  $('events').innerHTML = '';
  $('lesson').classList.add('hidden');
  $('scene-status').textContent = `Reset to ${selected.title} example`;
  renderer.setScene(selected.scene);
  combat.reset();
  renderSystemViews(selected);
  renderInventory(selected.inventory || []);
}

$('run').onclick = runSelected;
$('reset').onclick = resetExample;
$('toggle-drawer').onclick = () => {
  document.body.classList.toggle('drawer-hidden');
  $('toggle-drawer').textContent = document.body.classList.contains('drawer-hidden') ? '☷' : '☰';
};

loadScenarios().catch((err) => {
  $('result').textContent = String(err && err.stack || err);
});
