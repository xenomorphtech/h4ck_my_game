// Generic lightweight combat scene state for click interactions and run-result feedback.
// Kept separate from app.js/scene.js so future puzzle-specific death rules stay modular.

const ARENA_COMBAT = {
  '01-first-blood-batch': {
    playerMaxHp: 100,
    monsterMaxHp: 120,
    attackDamage: 40,
    retaliationDelayMs: 250,
    retaliationDamage: 100,
    allowDeadFighting: false,
  },
  '02-arena-fight-while-dead': {
    playerMaxHp: 100,
    monsterMaxHp: 160,
    attackDamage: 40,
    retaliationDelayMs: 250,
    retaliationDamage: 100,
    allowDeadFighting: true,
  },
};

class CombatController {
  constructor(renderer, dom) {
    this.renderer = renderer;
    this.dom = dom;
    this.scenario = null;
    this.config = null;
    this.retaliationTimer = null;
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
    this.clearTimer();
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

  handleSceneAction(action) {
    if (!this.state.active) return false;
    if (action.kind === 'cooldown') {
      this.setStatus(`attack cooling down — wait ${action.wait_ms}ms`);
      return true;
    }
    if (action.kind !== 'attack') return false;

    this.state.monsterHp = Math.max(0, this.state.monsterHp - this.config.attackDamage);
    if (this.state.monsterHp === 0) {
      this.killMonster('challenge complete');
      return true;
    }

    this.setStatus(`hit monster for ${this.config.attackDamage}; retaliation incoming`);
    this.scheduleRetaliation();
    this.render();
    return true;
  }

  applyRunResult(result) {
    if (!this.state.active || result.scenario_id !== this.scenario?.id) return;
    this.clearTimer();
    if (result.outcome === 'win') {
      this.killMonster('challenge complete');
      return;
    }
    const attackedMonster = (result.events || []).some((event) => (
      event.kind === 'client' && event.name === 'Attack' && Number(event.fields?.target) === 1
    ));
    if (attackedMonster) {
      this.state.monsterHp = Math.max(1, this.config.monsterMaxHp - this.config.attackDamage);
      this.die('monster retaliated before the objective completed');
    } else {
      this.render();
    }
  }

  scheduleRetaliation() {
    this.clearTimer();
    this.retaliationTimer = window.setTimeout(() => {
      if (!this.state.active || this.state.monsterDead) return;
      this.die('monster hit back with a fatal blow');
    }, this.config.retaliationDelayMs);
  }

  die(reason) {
    this.state.playerHp = Math.max(0, this.state.playerHp - this.config.retaliationDamage);
    this.state.dead = this.state.playerHp <= 0;
    if (this.state.dead) {
      this.dom.death.classList.remove('hidden');
      this.dom.deathText.textContent = `${reason}. Revive to restart the scene.`;
    }
    this.setStatus(this.state.dead ? 'died' : `player HP ${this.state.playerHp}`);
    this.render();
  }

  killMonster(message) {
    this.clearTimer();
    this.state.monsterHp = 0;
    this.state.monsterDead = true;
    this.state.complete = true;
    this.renderer.setHiddenLabels(['Monster']);
    this.dom.death.classList.add('hidden');
    this.showPopup(message);
    this.setStatus('objective complete');
    this.render();
  }

  render() {
    if (!this.state.active) return;
    this.dom.playerHp.textContent = `${this.state.playerHp}/${this.config.playerMaxHp}`;
    this.dom.monsterHp.textContent = `${this.state.monsterHp}/${this.config.monsterMaxHp}`;
    this.dom.playerBar.style.width = `${(this.state.playerHp / this.config.playerMaxHp) * 100}%`;
    this.dom.monsterBar.style.width = `${(this.state.monsterHp / this.config.monsterMaxHp) * 100}%`;
    this.dom.hud.classList.toggle('dead', this.state.dead);
    this.dom.hud.classList.toggle('complete', this.state.complete);
    this.renderer.setCombatState({
      active: this.state.active,
      monsterHp: this.state.monsterHp,
      monsterMaxHp: this.config.monsterMaxHp,
      monsterDead: this.state.monsterDead,
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

  clearTimer() {
    if (this.retaliationTimer) window.clearTimeout(this.retaliationTimer);
    this.retaliationTimer = null;
  }

  setStatus(text) {
    this.dom.status.textContent = text;
  }
}

window.CombatController = CombatController;
