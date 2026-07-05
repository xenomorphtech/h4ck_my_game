let scenarios = [];
let selected = null;
let socket = null;
let renderer = null;
let combat = null;
let completed = new Set();

const $ = (id) => document.getElementById(id);

async function loadScenarios() {
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
  renderInventory(scenario.inventory || []);
}

function renderInventory(items) {
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

    const iconWrap = document.createElement('div');
    iconWrap.className = 'inventory-icon';
    const url = window.gameIconUrl ? window.gameIconUrl(item.sprite) : null;
    if (url) {
      const img = document.createElement('img');
      img.src = url;
      img.alt = '';
      img.loading = 'lazy';
      img.onerror = () => {
        img.remove();
        iconWrap.textContent = item.name.slice(0, 2).toUpperCase();
      };
      iconWrap.appendChild(img);
    } else {
      iconWrap.textContent = item.name.slice(0, 2).toUpperCase();
    }

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
  $('scene-status').textContent = 'running packets…';
  const ws = ensureSocket();
  const send = () => ws.send(JSON.stringify({
    type: 'run_script',
    scenario_id: selected.id,
    script: $('script').value,
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
    state: result.state,
    error: result.error,
  }, null, 2);

  $('scene-status').textContent = result.outcome === 'win'
    ? 'objective complete'
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

function resetExample() {
  if (!selected) return;
  $('script').value = selected.example_script || '';
  $('result').textContent = '';
  $('events').innerHTML = '';
  $('lesson').classList.add('hidden');
  $('scene-status').textContent = `Reset to ${selected.title} example`;
  renderer.setScene(selected.scene);
  combat.reset();
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
