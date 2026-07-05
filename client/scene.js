// Reusable 2.5D scene renderer for Packet Hacker.
// A scene is data: { template, entities:[{sprite,x,y,label}], blocked_tiles:[{x,y,reason}] }.
// Sprites use local SVGs sourced from https://game-icons.net/ (CC BY 3.0),
// served from /client/icons/<sprite>.svg so the game works offline after checkout.

const TILE = 74;
const ORIGIN = { x: 90, y: 70 };

const ICON_NAMES = new Set([
  'anvil', 'arrow_stack', 'auction', 'blade', 'boss', 'bridge', 'cannon', 'chest',
  'crystal', 'currency', 'deed', 'gate', 'gem', 'guard', 'hero', 'key', 'mailbox',
  'monster', 'mount', 'npc', 'potion', 'quest_giver', 'relic', 'scale', 'shield',
  'shrine', 'trade_table', 'wall', 'wand'
]);

// Enemies aggro and retaliate when attacked.
const ENEMY_SPRITES = new Set(['monster', 'boss', 'guard']);

function iconUrl(sprite) {
  return ICON_NAMES.has(sprite) ? `/client/icons/${sprite}.svg` : null;
}

const ICON_CACHE = new Map();

function getIcon(sprite) {
  const url = iconUrl(sprite);
  if (!url) return null;
  if (!ICON_CACHE.has(sprite)) {
    const img = new Image();
    img.decoding = 'async';
    img.src = url;
    ICON_CACHE.set(sprite, img);
  }
  return ICON_CACHE.get(sprite);
}

// Palette per sprite kind. The icon is now the sprite itself; body/accent/shape
// define reusable presentation across many puzzles.
const SPRITES = {
  hero: { body: '#65a9ff', accent: '#bfe0ff', shape: 'actor' },
  monster: { body: '#f97066', accent: '#fecaca', shape: 'actor' },
  boss: { body: '#c084fc', accent: '#e9d5ff', shape: 'actor' },
  npc: { body: '#34d399', accent: '#bbf7d0', shape: 'actor' },
  guard: { body: '#fbbf24', accent: '#fde68a', shape: 'actor' },
  quest_giver: { body: '#38bdf8', accent: '#bae6fd', shape: 'actor' },

  gate: { body: '#94a3b8', accent: '#cbd5e1', shape: 'block' },
  wall: { body: '#64748b', accent: '#94a3b8', shape: 'block' },
  bridge: { body: '#a16207', accent: '#d9a441', shape: 'block' },
  chest: { body: '#d9a441', accent: '#fde68a', shape: 'prop' },
  mailbox: { body: '#60a5fa', accent: '#bfdbfe', shape: 'prop' },
  auction: { body: '#f59e0b', accent: '#fcd34d', shape: 'prop' },
  cannon: { body: '#475569', accent: '#94a3b8', shape: 'prop' },
  anvil: { body: '#475569', accent: '#94a3b8', shape: 'prop' },
  crystal: { body: '#22d3ee', accent: '#a5f3fc', shape: 'prop' },
  currency: { body: '#d9a441', accent: '#fde68a', shape: 'prop' },
  trade_table: { body: '#a16207', accent: '#d9a441', shape: 'prop' },
  shrine: { body: '#f0abfc', accent: '#f5d0fe', shape: 'prop' },
  lever: { body: '#facc15', accent: '#fef08a', shape: 'prop' },
  vendor: { body: '#34d399', accent: '#bbf7d0', shape: 'actor' },
  ally: { body: '#60a5fa', accent: '#bfdbfe', shape: 'actor' },
  crate: { body: '#a16207', accent: '#d9a441', shape: 'prop' },
  rift: { body: '#818cf8', accent: '#c7d2fe', shape: 'prop' },

  mount: { body: '#f472b6', accent: '#fbcfe8', shape: 'item' },
  gem: { body: '#22d3ee', accent: '#a5f3fc', shape: 'item' },
  relic: { body: '#facc15', accent: '#fef08a', shape: 'item' },
  scale: { body: '#4ade80', accent: '#bbf7d0', shape: 'item' },
  arrow_stack: { body: '#a3a3a3', accent: '#e5e5e5', shape: 'item' },
  deed: { body: '#e5e7eb', accent: '#ffffff', shape: 'item' },
  potion: { body: '#f472b6', accent: '#fbcfe8', shape: 'item' },
  blade: { body: '#e5e7eb', accent: '#ffffff', shape: 'item' },
  key: { body: '#facc15', accent: '#fef08a', shape: 'item' },
  shield: { body: '#93c5fd', accent: '#dbeafe', shape: 'item' },
  wand: { body: '#a78bfa', accent: '#ddd6fe', shape: 'item' },
};

const TEMPLATE_BG = {
  arena: ['#1a1030', '#2a1745'],
  gatehouse: ['#101a26', '#1c2c40'],
  crypt: ['#0f0f1a', '#1c1830'],
  siege: ['#241209', '#3a1f10'],
  market: ['#12210f', '#1f3a16'],
  postoffice: ['#101a26', '#1c2c40'],
  trade: ['#0f1a1a', '#183030'],
  inventory: ['#141018', '#241a2c'],
  treasury: ['#241f09', '#3a3210'],
  apothecary: ['#1a0f1a', '#301830'],
  cavern: ['#0b1420', '#132133'],
  ruins: ['#1a160f', '#2c261a'],
  vault: ['#241f09', '#3a3210'],
  guild: ['#0f1626', '#182640'],
  raid: ['#1a0f14', '#301824'],
  bridge: ['#0b1420', '#132133'],
  workshop: ['#141414', '#242424'],
  default: ['#0c1117', '#152033'],
};

function iso(x, y) {
  return {
    px: ORIGIN.x + x * TILE + y * 10,
    py: ORIGIN.y + y * (TILE * 0.62),
  };
}

// Inverse of iso(): map a pixel in canvas space back to the nearest tile.
function unIso(px, py) {
  const y = (py - ORIGIN.y) / (TILE * 0.62);
  const x = (px - ORIGIN.x - y * 10) / TILE;
  return { x: Math.round(x), y: Math.round(y) };
}

function roundRect(ctx, x, y, w, h, r) {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.arcTo(x + w, y, x + w, y + h, r);
  ctx.arcTo(x + w, y + h, x, y + h, r);
  ctx.arcTo(x, y + h, x, y, r);
  ctx.arcTo(x, y, x + w, y, r);
  ctx.closePath();
}

function drawFloor(ctx, template, w, h) {
  const [top, bottom] = TEMPLATE_BG[template] || TEMPLATE_BG.default;
  const grad = ctx.createLinearGradient(0, 0, 0, h);
  grad.addColorStop(0, top);
  grad.addColorStop(1, bottom);
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, w, h);

  ctx.strokeStyle = 'rgba(255,255,255,0.05)';
  ctx.lineWidth = 1;
  for (let gx = 0; gx <= 8; gx++) {
    for (let gy = 0; gy <= 5; gy++) {
      const a = iso(gx, gy);
      const b = iso(gx + 1, gy);
      const c = iso(gx, gy + 1);
      ctx.beginPath();
      ctx.moveTo(a.px, a.py);
      ctx.lineTo(b.px, b.py);
      ctx.moveTo(a.px, a.py);
      ctx.lineTo(c.px, c.py);
      ctx.stroke();
    }
  }
}

function drawBlockedTile(ctx, tile, pulse) {
  const { px, py } = iso(tile.x, tile.y);
  const shimmer = 0.12 + Math.sin(pulse / 240 + tile.x) * 0.04;
  ctx.save();
  ctx.fillStyle = `rgba(248, 113, 113, ${0.18 + shimmer})`;
  ctx.strokeStyle = 'rgba(252, 165, 165, 0.72)';
  ctx.lineWidth = 2;
  ctx.beginPath();
  ctx.moveTo(px, py - 22);
  ctx.lineTo(px + 34, py);
  ctx.lineTo(px, py + 22);
  ctx.lineTo(px - 34, py);
  ctx.closePath();
  ctx.fill();
  ctx.stroke();

  ctx.strokeStyle = 'rgba(15, 23, 42, 0.75)';
  ctx.lineWidth = 3;
  ctx.beginPath();
  ctx.moveTo(px - 16, py - 10);
  ctx.lineTo(px + 16, py + 10);
  ctx.moveTo(px + 16, py - 10);
  ctx.lineTo(px - 16, py + 10);
  ctx.stroke();
  ctx.restore();
}

function drawShadow(ctx, px, py) {
  ctx.fillStyle = 'rgba(0,0,0,0.35)';
  ctx.beginPath();
  ctx.ellipse(px, py + 26, 22, 8, 0, 0, Math.PI * 2);
  ctx.fill();
}

function drawGameIcon(ctx, img, px, cy, size) {
  if (!img || !img.complete || img.naturalWidth === 0) return false;
  ctx.save();
  ctx.globalCompositeOperation = 'source-over';
  ctx.filter = 'drop-shadow(0 0 4px rgba(0,0,0,.4))';
  ctx.drawImage(img, px - size / 2, cy - size / 2, size, size);
  ctx.restore();
  return true;
}

function drawFallbackGlyph(ctx, sprite, px, cy) {
  ctx.font = '13px ui-monospace, monospace';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillStyle = '#07111f';
  ctx.fillText(sprite.slice(0, 2).toUpperCase(), px, cy);
}

// Loose inventory items should live in the inventory panel, not on the floor.
// Only actors and interactable structures/props belong in the scene.
function isFloorClutter(sprite) {
  return (SPRITES[sprite] || {}).shape === 'item';
}

function drawEntity(ctx, entity, px, py, pulse, aggro) {
  const def = SPRITES[entity.sprite] || SPRITES.chest;
  drawShadow(ctx, px, py);

  const bob = def.shape === 'actor' ? Math.sin(pulse / 400 + entity.x) * 3 : 0;
  const shake = aggro ? Math.sin(pulse / 50) * 2 : 0;
  const cy = py + bob;
  const cx = px + shake;

  if (aggro) {
    ctx.save();
    ctx.strokeStyle = 'rgba(248,113,102,0.8)';
    ctx.lineWidth = 2;
    const r = 30 + Math.sin(pulse / 160) * 3;
    ctx.beginPath();
    ctx.ellipse(cx, cy - 6, r, r * 0.62, 0, 0, Math.PI * 2);
    ctx.stroke();
    ctx.restore();
  }

  ctx.save();
  ctx.shadowColor = def.body;
  ctx.shadowBlur = 14;
  ctx.fillStyle = def.body;
  if (def.shape === 'block') {
    roundRect(ctx, cx - 28, cy - 36, 56, 56, 8);
  } else if (def.shape === 'item') {
    roundRect(ctx, cx - 18, cy - 24, 36, 36, 9);
  } else {
    roundRect(ctx, cx - 24, cy - 32, 48, 48, 14);
  }
  ctx.fill();
  ctx.restore();

  ctx.fillStyle = def.accent;
  ctx.globalAlpha = 0.28;
  roundRect(ctx, cx - 21, cy - 30, 42, 15, 8);
  ctx.fill();
  ctx.globalAlpha = 1;

  const img = getIcon(entity.sprite);
  const iconSize = def.shape === 'item' ? 25 : 34;
  if (!drawGameIcon(ctx, img, cx, cy - 8, iconSize)) drawFallbackGlyph(ctx, entity.sprite, cx, cy - 8);

  ctx.font = '12px ui-monospace, monospace';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillStyle = '#d7e1ee';
  ctx.fillText(entity.label, cx, cy + 28);
}

function drawHpBadge(ctx, label, hp, maxHp, px, py) {
  const safeMax = Math.max(1, maxHp || 1);
  const ratio = Math.max(0, Math.min(1, hp / safeMax));
  const w = 82;
  const h = 18;
  const x = px - w / 2;
  const y = py - 64;

  ctx.save();
  roundRect(ctx, x, y, w, h, 7);
  ctx.fillStyle = 'rgba(7, 11, 18, 0.86)';
  ctx.fill();
  ctx.strokeStyle = 'rgba(248, 113, 113, 0.72)';
  ctx.lineWidth = 1;
  ctx.stroke();

  roundRect(ctx, x + 4, y + 10, w - 8, 4, 3);
  ctx.fillStyle = '#1f2937';
  ctx.fill();
  roundRect(ctx, x + 4, y + 10, (w - 8) * ratio, 4, 3);
  ctx.fillStyle = ratio > 0.4 ? '#ef4444' : '#f97316';
  ctx.fill();

  ctx.font = '10px ui-monospace, monospace';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'middle';
  ctx.fillStyle = '#fee2e2';
  ctx.fillText(`${label} ${hp}/${safeMax}`, px, y + 6);
  ctx.restore();
}

class SceneRenderer {
  constructor(canvas) {
    this.canvas = canvas;
    this.ctx = canvas.getContext('2d');
    this.scene = { template: 'default', entities: [] };
    this.flashes = [];
    this.strikes = [];
    this.aggro = new Set();
    this.hiddenLabels = new Set();
    this.heroPos = null;
    this.heroTarget = null;
    this.combatState = null;
    this.attackCooldownMs = 750;
    this.nextAttackAt = 0;
    this.raf = null;
    this.onAction = null;
    this.isActionAllowed = () => true;
    this.loop = this.loop.bind(this);
    canvas.addEventListener('click', (ev) => this.handleClick(ev));
    canvas.style.cursor = 'pointer';
  }

  // renderable entities (loose inventory items filtered out)
  visibleEntities() {
    return (this.scene.entities || []).filter((e) => !isFloorClutter(e.sprite) && !this.hiddenLabels.has(e.label));
  }

  entityPos(entity) {
    if (entity.sprite === 'hero' && this.heroPos) return this.heroPos;
    return { x: entity.x, y: entity.y };
  }

  setScene(scene) {
    this.scene = scene || { template: 'default', entities: [] };
    this.flashes = [];
    this.strikes = [];
    this.aggro = new Set();
    this.hiddenLabels = new Set();
    this.nextAttackAt = 0;
    const hero = (this.scene.entities || []).find((e) => e.sprite === 'hero');
    this.heroPos = hero ? { x: hero.x, y: hero.y } : null;
    this.heroTarget = null;
    this.preloadSceneIcons();
    this.start();
  }

  setActionGate(fn) {
    this.isActionAllowed = typeof fn === 'function' ? fn : () => true;
  }

  setHiddenLabels(labels) {
    this.hiddenLabels = new Set(labels || []);
    this.start();
  }

  setCombatState(state) {
    this.combatState = state || null;
    this.start();
  }

  preloadSceneIcons() {
    for (const entity of this.scene.entities || []) {
      const img = getIcon(entity.sprite);
      if (img && !img.complete) img.onload = () => this.start();
    }
  }

  // canvas-space coordinates from a mouse event (accounts for CSS scaling)
  eventToCanvas(ev) {
    const rect = this.canvas.getBoundingClientRect();
    return {
      px: (ev.clientX - rect.left) * (this.canvas.width / rect.width),
      py: (ev.clientY - rect.top) * (this.canvas.height / rect.height),
    };
  }

  entityAt(px, py) {
    let best = null;
    let bestDist = 46; // click radius in canvas px
    for (const entity of this.visibleEntities()) {
      const pos = this.entityPos(entity);
      const { px: ex, py: ey } = iso(pos.x, pos.y);
      const dist = Math.hypot(px - ex, py - (ey - 8));
      if (dist < bestDist) {
        bestDist = dist;
        best = entity;
      }
    }
    return best;
  }

  blockedTileAt(x, y) {
    return (this.scene.blocked_tiles || []).find((tile) => tile.x === x && tile.y === y) || null;
  }

  handleClick(ev) {
    const { px, py } = this.eventToCanvas(ev);
    const target = this.entityAt(px, py);
    if (target && ENEMY_SPRITES.has(target.sprite)) {
      this.attack(target);
    } else if (target && target.sprite === 'hero') {
      // clicking yourself does nothing
    } else {
      const tile = unIso(px, py);
      this.moveHeroTo(tile.x, tile.y);
    }
  }

  moveHeroTo(x, y) {
    if (!this.heroPos) return;
    const target = {
      x: Math.max(0, Math.min(8, x)),
      y: Math.max(0, Math.min(5, y)),
    };
    const blocked = this.blockedTileAt(target.x, target.y);
    if (blocked) {
      if (this.onAction) {
        this.onAction({
          kind: 'blocked',
          x: target.x,
          y: target.y,
          reason: blocked.reason || 'blocked',
        });
      }
      this.start();
      return;
    }
    if (!this.isActionAllowed({ kind: 'move', x: target.x, y: target.y })) return;
    this.heroTarget = target;
    this.reactNearbyGuards(target);
    if (this.onAction) this.onAction({ kind: 'move', x: this.heroTarget.x, y: this.heroTarget.y });
    this.start();
  }

  // Guards aggro when the player steps within one tile of them, so a blocked
  // corridor reads as "the guard is watching this approach" rather than static scenery.
  reactNearbyGuards(target) {
    for (const entity of this.visibleEntities()) {
      if (entity.sprite !== 'guard') continue;
      const pos = this.entityPos(entity);
      if (Math.abs(pos.x - target.x) <= 1 && Math.abs(pos.y - target.y) <= 1) {
        this.aggro.add(entity.label);
      }
    }
  }

  attack(enemy) {
    const now = performance.now();
    if (!this.isActionAllowed({ kind: 'attack', target: enemy.label, enemy })) return;
    if (now < this.nextAttackAt) {
      if (this.onAction) this.onAction({ kind: 'cooldown', wait_ms: Math.ceil(this.nextAttackAt - now) });
      return;
    }
    this.nextAttackAt = now + this.attackCooldownMs;
    const heroPos = this.heroPos || { x: 1, y: 3 };
    const from = iso(heroPos.x, heroPos.y);
    const to = iso(enemy.x, enemy.y);
    // player's strike travels to the enemy...
    this.strikes.push({ at: now, from, to, color: '#65a9ff' });
    // ...and the enemy retaliates ~250ms later (the fatal window).
    this.strikes.push({ at: now + 250, from: to, to: from, color: '#f97066' });
    this.aggro.add(enemy.label);
    if (this.onAction) this.onAction({ kind: 'attack', target: enemy.label });
    this.start();
  }

  playEvents(events, outcome) {
    this.flashes = [];
    const now = performance.now();
    (events || []).forEach((ev, i) => {
      this.flashes.push({ at: now + i * 70, kind: ev.kind, name: ev.name });
    });
    this.outcome = outcome;
    this.start();
  }

  start() {
    if (!this.raf) this.raf = requestAnimationFrame(this.loop);
  }

  updateHero() {
    if (!this.heroPos || !this.heroTarget) return false;
    const dx = this.heroTarget.x - this.heroPos.x;
    const dy = this.heroTarget.y - this.heroPos.y;
    if (Math.abs(dx) < 0.02 && Math.abs(dy) < 0.02) {
      this.heroPos = { x: this.heroTarget.x, y: this.heroTarget.y };
      this.heroTarget = null;
      return false;
    }
    this.heroPos = { x: this.heroPos.x + dx * 0.18, y: this.heroPos.y + dy * 0.18 };
    return true;
  }

  loop(ts) {
    const { ctx, canvas } = this;
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    drawFloor(ctx, this.scene.template, canvas.width, canvas.height);

    const heroMoving = this.updateHero();

    // z-sort by tile y so nearer entities draw on top
    const ordered = this.visibleEntities()
      .map((e) => ({ e, pos: this.entityPos(e) }))
      .sort((a, b) => a.pos.y - b.pos.y);
    for (const { e, pos } of ordered) {
      const { px, py } = iso(pos.x, pos.y);
      drawEntity(ctx, e, px, py, ts, this.aggro.has(e.label));
    }

    if (this.combatState?.active && !this.combatState.monsterDead) {
      for (const { e, pos } of ordered) {
        if (!ENEMY_SPRITES.has(e.sprite)) continue;
        const { px, py } = iso(pos.x, pos.y);
        drawHpBadge(ctx, 'HP', this.combatState.monsterHp, this.combatState.monsterMaxHp, px, py);
      }
    }

    // Draw blocked tiles as a top overlay so unwalkable squares remain visible
    // even when the blocker is represented by a guard/gate entity on that tile.
    for (const tile of this.scene.blocked_tiles || []) {
      drawBlockedTile(ctx, tile, ts);
    }

    // melee strikes (click-to-attack + retaliation)
    let strikeActive = false;
    for (const s of this.strikes) {
      const age = ts - s.at;
      if (age < 0 || age > 260) {
        if (age < 0) strikeActive = true;
        continue;
      }
      strikeActive = true;
      const t = age / 260;
      const cx = s.from.px + (s.to.px - s.from.px) * t;
      const cy = s.from.py + (s.to.py - s.from.py) * t - 6;
      ctx.globalAlpha = 1 - t * 0.6;
      ctx.fillStyle = s.color;
      roundRect(ctx, cx - 6, cy - 6, 12, 12, 4);
      ctx.fill();
      ctx.globalAlpha = 1;
    }
    this.strikes = this.strikes.filter((s) => ts - s.at <= 260);

    // packet-run flashes
    const heroRef = this.visibleEntities().find((e) => e.sprite === 'hero');
    const origin = heroRef ? iso(this.entityPos(heroRef).x, this.entityPos(heroRef).y) : { px: 120, py: 200 };
    let flashActive = 0;
    for (const f of this.flashes) {
      const age = ts - f.at;
      if (age < 0 || age > 900) continue;
      flashActive++;
      const t = age / 900;
      const color = f.kind === 'server' ? (this.outcome === 'win' ? '#86efac' : '#fca5a5') : '#65a9ff';
      ctx.globalAlpha = 1 - t;
      ctx.fillStyle = color;
      const rx = origin.px + Math.sin(f.at) * 40;
      const ry = origin.py - 40 - t * 80;
      roundRect(ctx, rx - 5, ry - 5, 10, 10, 3);
      ctx.fill();
      ctx.globalAlpha = 1;
    }

    const hasActor = this.visibleEntities().some((e) => SPRITES[e.sprite]?.shape === 'actor');
    const hasBlockedTiles = (this.scene.blocked_tiles || []).length > 0;
    const hasCombatOverlay = !!this.combatState?.active;
    if (flashActive > 0 || strikeActive || heroMoving || this.aggro.size > 0 || hasActor || hasBlockedTiles || hasCombatOverlay) {
      this.raf = requestAnimationFrame(this.loop);
    } else {
      this.raf = null;
    }
  }
}

window.SceneRenderer = SceneRenderer;
window.gameIconUrl = iconUrl;
