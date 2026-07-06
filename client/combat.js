// Generic lightweight combat scene state for click interactions and run-result feedback.
// Kept separate from app.js/scene.js so future puzzle-specific death rules stay modular.
//
// Authority model: the server owns combat. Monster/player damage, death, and
// retaliation are only reflected when the backend sends the matching Damage /
// Death packets in a run result. The client never invents retaliation on a
// timer; local click handling is a lightweight preview of the player's own
// action only.

const ARENA_COMBAT = {
  '01-first-blood-batch': {
    playerMaxHp: 100,
    monsterMaxHp: 120,
    attackDamage: 40,
    retaliationDelayMs: 250,
    allowDeadFighting: false,
    actionName: 'Attack',
    targetField: 'target',
    targetValue: 1,
    monsterStatLabel: 'HP',
  },
  '02-arena-fight-while-dead': {
    playerMaxHp: 100,
    monsterMaxHp: 160,
    attackDamage: 40,
    retaliationDelayMs: 250,
    allowDeadFighting: true,
    actionName: 'Attack',
    targetField: 'target',
    targetValue: 1,
    monsterStatLabel: 'HP',
  },
  '16-cooldown-bypass-batch': {
    playerMaxHp: 100,
    monsterMaxHp: 1,
    shieldMaxHp: 150,
    attackDamage: 10,
    attackCooldownMs: 750,
    powerStrikeDamage: 50,
    powerStrikeCooldownMs: 1000,
    retaliationDelayMs: 500,
    allowDeadFighting: false,
    actionName: 'Attack',
    targetField: 'target',
    targetValue: 1,
    monsterStatLabel: 'HP',
    damageNoun: 'HP damage',
    retaliator: 'boss',
    monsterLabel: 'Boss',
  },
};

class CombatController {
  constructor(renderer, dom) {
    this.renderer = renderer;
    this.dom = dom;
    this.scenario = null;
    this.config = null;
    this.state = this.emptyState();

    this.dom.revive.onclick = () => this.revive();
    this.dom.popupClose.onclick = () => this.hidePopup();
    this.renderer.setActionGate((action) => this.allowAction(action));
  }

  emptyState() {
    return {
      active: false,
      playerHp: 0,
      monsterHp: 0,
      shieldHp: null,
      dead: false,
      monsterDead: false,
      complete: false,
    };
  }

  setScenario(scenario) {
    this.scenario = scenario;
    this.config = ARENA_COMBAT[scenario?.id] || null;
    this.reset();
  }

  reset() {
    if (!this.config) {
      this.state = this.emptyState();
      this.renderer.setHiddenLabels([]);
      this.renderer.setCombatState(null);
      this.dom.hud.classList.add('hidden');
      this.dom.death.classList.add('hidden');
      this.hidePopup();
      return;
    }

    this.state = {
      active: true,
      playerHp: this.config.playerMaxHp,
      monsterHp: this.config.monsterMaxHp,
      shieldHp: typeof this.config.shieldMaxHp === 'number' ? this.config.shieldMaxHp : null,
      dead: false,
      monsterDead: false,
      complete: false,
    };
    this.renderer.setHiddenLabels([]);
    this.dom.hud.classList.remove('hidden');
    this.dom.death.classList.add('hidden');
    this.hidePopup();
    this.render();
  }

  revive() {
    if (!this.scenario) return;
    this.renderer.setScene(this.scenario.scene);
    this.reset();
    this.setStatus('revived — scene restarted');
  }

  allowAction(action) {
    if (!this.state.active) return true;
    if (this.state.monsterDead && action.kind === 'attack') {
      this.setStatus('monster is already defeated');
      return false;
    }
    if (this.state.dead && action.kind === 'move') {
      this.setStatus('you died — revive to move again');
      return false;
    }
    if (this.state.dead && action.kind === 'attack' && !this.config.allowDeadFighting) {
      this.setStatus('you died — revive to act again');
      return false;
    }
    return true;
  }

  packetForAction(action) {
    if (!this.state.active || action.kind !== 'attack' || !this.config?.actionName) return null;
    const field = this.config.targetField || 'target';
    const target = this.config.targetValue ?? 1;
    return `send ${this.config.actionName} { ${field}: ${target} }`;
  }

  handleSceneAction(action) {
    if (!this.state.active) return false;
    if (action.kind === 'cooldown') {
      this.setStatus(`attack cooling down — wait ${action.wait_ms}ms`);
      return true;
    }
    if (action.kind !== 'attack') return false;

    this.setStatus(`attacking ${action.target || 'monster'}`);
    this.render();
    return true;
  }

  // The run result carries the authoritative server combat log. Replay the
  // server's Damage/Death packets in time order and mirror them into the HUD.
  applyRunResult(result) {
    if (!this.state.active || result.scenario_id !== this.scenario?.id) return;

    const server = (result.events || [])
      .filter((event) => event.kind === 'server')
      .slice()
      .sort((a, b) => (a.t || 0) - (b.t || 0));

    for (const event of server) {
      if (event.name === 'Damage') this.applyServerDamage(event);
      else if (event.name === 'Death') this.applyServerDeath(event);
    }

    if (result.outcome === 'win' && !this.state.monsterDead) {
      this.killMonster('challenge complete');
    }
    this.render();
  }

  monsterTarget() {
    return this.config.targetValue ?? 1;
  }

  applyServerDamage(event) {
    const fields = event.fields || {};
    const target = Number(fields.target);
    const amount = Number(fields.amount) || 0;

    if (target === this.monsterTarget()) {
      if (typeof this.state.shieldHp === 'number' && this.state.shieldHp > 0) {
        this.state.shieldHp = Math.max(0, this.state.shieldHp - amount);
        this.setStatus(`shield absorbs ${amount}; SHIELD ${this.state.shieldHp}/${this.config.shieldMaxHp}`);
      } else {
        this.state.monsterHp = Math.max(0, this.state.monsterHp - amount);
        const noun = this.config.damageNoun || 'damage';
        this.setStatus(`server: ${amount} ${noun}; ${this.config.monsterStatLabel} ${this.state.monsterHp}/${this.config.monsterMaxHp}`);
      }
    } else {
      this.state.playerHp = Math.max(0, this.state.playerHp - amount);
      this.setStatus(`server: ${this.config.retaliator || 'monster'} hit you for ${amount}; HP ${this.state.playerHp}/${this.config.playerMaxHp}`);
    }
  }

  applyServerDeath(event) {
    const target = Number(event.fields?.target);
    if (target === this.monsterTarget()) {
      this.killMonster('challenge complete');
    } else {
      this.markPlayerDead(`${this.config.retaliator || 'monster'} retaliated with a fatal blow`);
    }
  }

  markPlayerDead(reason) {
    this.state.playerHp = 0;
    this.state.dead = true;
    // Do not hide the "monster dead" outcome: in arenas that accept ghost
    // actions the player can still finish the monster after this point, and a
    // later monster Death packet will flip the scene to complete.
    if (!this.state.monsterDead) {
      this.dom.death.classList.remove('hidden');
      this.dom.deathText.textContent = `${reason}. Revive to restart the scene.`;
    }
    this.setStatus('you died');
    this.render();
  }

  killMonster(message) {
    this.state.monsterHp = 0;
    this.state.monsterDead = true;
    this.state.complete = true;
    this.renderer.setHiddenLabels([this.config.monsterLabel || (this.config.monsterStatLabel === 'SHIELD' ? 'Boss' : 'Monster')]);
    this.dom.death.classList.add('hidden');
    this.showPopup(message);
    this.setStatus('objective complete');
    this.render();
  }

  render() {
    if (!this.state.active) return;
    this.dom.playerHp.textContent = `${this.state.playerHp}/${this.config.playerMaxHp}`;
    this.dom.monsterHp.textContent = `${this.state.monsterHp}/${this.config.monsterMaxHp}`;
    this.dom.playerBar.style.width = `${Math.max(0, (this.state.playerHp / this.config.playerMaxHp) * 100)}%`;
    this.dom.monsterBar.style.width = `${Math.max(0, (this.state.monsterHp / this.config.monsterMaxHp) * 100)}%`;
    this.dom.hud.classList.toggle('dead', this.state.dead);
    this.dom.hud.classList.toggle('complete', this.state.complete);
    this.renderer.setCombatState({
      active: this.state.active,
      hp: this.state.monsterHp,
      maxHp: this.config.monsterMaxHp,
      shieldHp: this.state.shieldHp,
      maxShield: this.config.shieldMaxHp,
      monsterHp: this.state.monsterHp,
      monsterMaxHp: this.config.monsterMaxHp,
      monsterDead: this.state.monsterDead,
      statLabel: this.config.monsterStatLabel || 'HP',
    });
  }

  showPopup(message) {
    this.dom.popupTitle.textContent = 'Challenge complete';
    this.dom.popupText.textContent = message;
    this.dom.popup.classList.remove('hidden');
  }

  hidePopup() {
    this.dom.popup.classList.add('hidden');
  }

  setStatus(text) {
    this.dom.status.textContent = text;
  }
}

window.CombatController = CombatController;
