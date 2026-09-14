import "./styles.css";

import { appConfig } from "./config";
import { DemoGateway } from "./demo";
import {
  badgeList,
  formatCountdown,
  formatDump,
  formatGlory,
  heatAt,
  heatClearAt,
  heatCost,
  minimumAction,
  noActionScoreForecast,
  parseDumpInput,
  rankPlayers,
  scoreDeltaForBalanceChange,
  secondsUntilHeatAvailable,
  shortAddress,
  territoryLevel,
  threatViews,
} from "./rules";
import type {
  ActionKind,
  ActionRequest,
  GameGateway,
  PlayerView,
  StrategySnapshot,
} from "./types";

const ACTION_COPY: Record<ActionKind, { title: string; description: string }> = {
  dump: {
    title: "DUMP",
    description: "Push unlocked DUMP into the target's deterministic lane. Armed Guard ricochets it back.",
  },
  absorb: {
    title: "ABSORB",
    description: "Take unlocked DUMP from a rival. It hurts your score, locks for six hours, and converts 50% into capped Guard.",
  },
  redirect: {
    title: "REDIRECT",
    description: "Arm one lane for a single incoming DUMP. Guard bounces the burden back, then the lane rearms after fifteen minutes.",
  },
};

class StrategyRoom {
  private gateway!: GameGateway;
  private snapshot?: StrategySnapshot;
  private action: ActionKind = "dump";
  private selectedTarget?: string;
  private atlasFilter: "all" | "leaders" | "burdened" = "all";
  private busy = false;

  constructor() {}

  async start(): Promise<void> {
    if (appConfig.liveEnabled) {
      const { SolanaGateway } = await import("./chain");
      this.gateway = new SolanaGateway();
    } else {
      this.gateway = new DemoGateway();
    }
    this.bind();
    if (!appConfig.liveEnabled) {
      await this.connect();
    } else {
      this.setStatus("Live program configured. Connect a Solana wallet to enter the Strategy Room.");
      this.renderMode();
    }
    window.setInterval(() => this.renderFastState(), 1_000);
    window.setInterval(() => void this.refreshSilently(), appConfig.liveEnabled ? 15_000 : 30_000);
  }

  private bind(): void {
    element<HTMLButtonElement>("connectButton").addEventListener("click", () => void this.connect());
    element<HTMLButtonElement>("registerButton").addEventListener("click", () =>
      void this.lifecycle("Preparing commitment and registration", () => this.gateway.register()));
    element<HTMLButtonElement>("revealButton").addEventListener("click", () =>
      void this.lifecycle("Revealing the saved entropy secret", () => this.gateway.reveal()));
    element<HTMLButtonElement>("claimButton").addEventListener("click", () =>
      void this.lifecycle("Claiming the starting burden", () => this.gateway.claimAllocation()));
    element<HTMLButtonElement>("advanceButton").addEventListener("click", () =>
      void this.lifecycle("Running the next permissionless epoch transition", () => this.gateway.advanceEpoch()));
    element<HTMLButtonElement>("settleButton").addEventListener("click", () =>
      void this.lifecycle("Settling one player and collecting the caller bounty", () => this.gateway.settleNextPlayer()));
    element<HTMLButtonElement>("rewardButton").addEventListener("click", () =>
      void this.lifecycle("Claiming earned GLORY", () => this.gateway.claimPlayerReward()));
    element<HTMLButtonElement>("bondButton").addEventListener("click", () =>
      void this.lifecycle("Reclaiming the eligible entry bond", () => this.gateway.claimBond()));
    element<HTMLButtonElement>("badgesButton").addEventListener("click", () =>
      void this.lifecycle("Synchronizing final achievement badges", () => this.gateway.refreshBadges()));
    element<HTMLButtonElement>("backupSecretButton").addEventListener("click", () =>
      void this.secretOperation("Downloading the current reveal-secret backup", () =>
        this.gateway.exportRevealSecret()));
    element<HTMLInputElement>("restoreSecretInput").addEventListener("change", (event) => {
      const input = event.currentTarget as HTMLInputElement;
      const file = input.files?.[0];
      if (file) {
        void this.secretOperation("Validating and restoring the reveal-secret backup", () =>
          this.gateway.importRevealSecret(file));
      }
      input.value = "";
    });
    element<HTMLButtonElement>("executeButton").addEventListener("click", () => void this.execute());
    element<HTMLButtonElement>("selectedTarget").addEventListener("click", () => {
      document.querySelector(".atlas-section")?.scrollIntoView({ behavior: "smooth", block: "start" });
    });
    for (const tab of document.querySelectorAll<HTMLButtonElement>(".action-tab")) {
      tab.addEventListener("click", () => {
        this.action = (tab.dataset.action ?? "dump") as ActionKind;
        this.renderOrders();
      });
    }
    for (const chip of document.querySelectorAll<HTMLButtonElement>(".chip")) {
      chip.addEventListener("click", () => {
        this.atlasFilter = (chip.dataset.filter ?? "all") as typeof this.atlasFilter;
        this.renderAtlas();
      });
    }
    for (const field of ["actionAmount", "actorLane", "targetLane"] as const) {
      element<HTMLInputElement | HTMLSelectElement>(field).addEventListener("input", () =>
        this.renderActionPreview());
    }
  }

  private async connect(): Promise<void> {
    if (this.busy) return;
    this.busy = true;
    this.setStatus(appConfig.liveEnabled ? "Requesting Solana wallet connection…" : "Loading the playable Strategy Room demo…");
    try {
      this.snapshot = await this.gateway.connect();
      this.selectedTarget ??= this.snapshot.players.find((player) => !player.isSelf)?.address;
      this.render();
      this.setStatus(
        this.snapshot.mode === "live"
          ? "Live Solana state loaded."
          : "Global-field demo loaded. Click any wallet tile, then issue an order.",
        "success",
      );
    } catch (error) {
      this.setStatus(errorMessage(error), "error");
    } finally {
      this.busy = false;
      this.syncButtons();
    }
  }

  private async execute(): Promise<void> {
    if (!this.snapshot || this.busy) return;
    const self = this.selfPlayer();
    if (!self) {
      this.setStatus("Join and claim an allocation before issuing orders.", "error");
      return;
    }
    const actorLane = Number(element<HTMLSelectElement>("actorLane").value);
    const request: ActionRequest = { kind: this.action, actorLane };
    if (this.action !== "redirect") {
      const amount = parseDumpInput(element<HTMLInputElement>("actionAmount").value);
      if (!this.selectedTarget || amount === null || amount <= 0n) {
        this.setStatus("Choose a target and enter a valid DUMP amount.", "error");
        return;
      }
      request.target = this.selectedTarget;
      request.amount = amount;
      request.targetLane = Number(element<HTMLSelectElement>("targetLane").value);
    }

    this.busy = true;
    this.syncButtons();
    this.setStatus(`Simulating ${this.action.toUpperCase()} and requesting approval…`);
    try {
      const signature = await this.gateway.act(request);
      this.snapshot = await this.gateway.refresh();
      this.render();
      this.setStatus(
        this.snapshot.mode === "live"
          ? `${this.action.toUpperCase()} confirmed: ${shortAddress(signature, 8)}`
          : `${this.action.toUpperCase()} resolved in the demo room.`,
        "success",
      );
    } catch (error) {
      this.setStatus(errorMessage(error), "error");
    } finally {
      this.busy = false;
      this.syncButtons();
    }
  }

  private async lifecycle(label: string, operation: () => Promise<string>): Promise<void> {
    if (this.busy) return;
    this.busy = true;
    this.syncButtons();
    this.setStatus(`${label}…`);
    try {
      const signature = await operation();
      this.snapshot = await this.gateway.refresh();
      this.render();
      this.setStatus(
        this.snapshot.mode === "live"
          ? `Confirmed: ${shortAddress(signature, 8)}. Keep the downloaded reveal-secret backup safe.`
          : `${label} complete in demo mode.`,
        "success",
      );
    } catch (error) {
      this.setStatus(errorMessage(error), "error");
    } finally {
      this.busy = false;
      this.syncButtons();
    }
  }

  private async secretOperation(label: string, operation: () => Promise<void>): Promise<void> {
    if (this.busy) return;
    this.busy = true;
    this.syncButtons();
    this.setStatus(`${label}…`);
    try {
      await operation();
      if (this.snapshot) this.snapshot = await this.gateway.refresh();
      this.render();
      this.setStatus(`${label} complete. Store the file somewhere private and durable.`, "success");
    } catch (error) {
      this.setStatus(errorMessage(error), "error");
    } finally {
      this.busy = false;
      this.syncButtons();
    }
  }

  private async refreshSilently(): Promise<void> {
    if (!this.snapshot || this.busy) return;
    try {
      this.snapshot = await this.gateway.refresh();
      this.render();
    } catch {
      // Preserve the last usable snapshot. Explicit actions still surface errors.
    }
  }

  private render(): void {
    this.renderMode();
    this.renderEpoch();
    this.renderSelf();
    this.renderOrders();
    this.renderThreatView();
    this.renderAtlas();
    this.renderLeaderboard();
    this.renderFeed();
    this.syncButtons();
  }

  private renderMode(): void {
    const mode = this.snapshot?.mode ?? (appConfig.liveEnabled ? "live" : "demo");
    text("modeBadge", mode.toUpperCase());
    text("clusterLabel", this.snapshot?.clusterLabel ?? appConfig.clusterLabel);
    text("connectButton", mode === "live" && this.snapshot ? "CONNECTED" : "ENTER ROOM");
  }

  private renderEpoch(): void {
    if (!this.snapshot) return;
    const { epoch } = this.snapshot;
    text("epochNumber", epoch.number.toString());
    text("phaseBadge", epoch.phase.toUpperCase());
    element("phaseBadge").dataset.phase = epoch.phase;
    text("participantCount", epoch.participantCount.toLocaleString("en-US"));
    text("revealedCount", epoch.revealedCount.toLocaleString("en-US"));
    text("rewardPool", formatGlory(epoch.playerRewardPool));
    text("advanceButton", transitionLabel(epoch.phase));
    text(
      "settleButton",
      epoch.phase === "settling"
        ? `SETTLE NEXT · ${Math.max(0, epoch.participantCount - epoch.settledCount)} LEFT`
        : "SETTLE NEXT PLAYER",
    );
    this.renderFastState();
  }

  private renderFastState(): void {
    if (!this.snapshot) return;
    const now = Math.floor(Date.now() / 1000);
    const { epoch } = this.snapshot;
    const countdown = ["complete", "cancelled"].includes(epoch.phase)
      ? "CLOSED"
      : formatCountdown(epoch.phaseEndsAt, now);
    text("phaseCountdown", countdown);
    text("phaseCaption", phaseCaption(epoch.phase));
    let progress = 0;
    if (epoch.activeEndsAt > epoch.activeStartsAt) {
      progress = Math.max(
        0,
        Math.min(100, ((now - epoch.activeStartsAt) * 100) / (epoch.activeEndsAt - epoch.activeStartsAt)),
      );
    }
    element<HTMLElement>("epochProgress").style.width = `${progress}%`;
    const self = this.selfPlayer();
    if (self) {
      this.setHeat("dump", heatAt(self.dumpHeat, now));
      this.setHeat("absorb", heatAt(self.absorbHeat, now));
    }
    this.renderScoreForecast(now);
    this.renderResourceTimeline(now);
  }

  private renderSelf(): void {
    if (!this.snapshot) return;
    const self = this.selfPlayer();
    text("walletAddress", this.snapshot.walletAddress ? shortAddress(this.snapshot.walletAddress, 7) : "No wallet");
    text("gloryBalance", formatGlory(this.snapshot.gloryBalance));
    if (!self) {
      text("selfAlias", "Spectator");
      text("selfScore", "—");
      text("selfBalance", "—");
      text("selfRank", "—");
      element("laneGrid").replaceChildren(emptyMessage("Join this epoch to unlock four action lanes."));
      element("badgeShelf").replaceChildren(emptyMessage("No player record for this epoch."));
      this.setHeat("dump", 0);
      this.setHeat("absorb", 0);
      return;
    }

    const ranked = rankPlayers(this.snapshot.players.filter((player) => player.eligible));
    const rank = ranked.findIndex((player) => player.address === self.address);
    text("selfAlias", self.alias);
    text("selfScore", formatDump(self.projectedScore));
    text("selfBalance", formatDump(self.balance));
    text("selfRank", rank >= 0 ? `#${rank + 1}` : "INELIGIBLE");
    this.renderLanes(self);
    const badges = badgeList(self.badges);
    const shelf = element("badgeShelf");
    shelf.replaceChildren();
    if (badges.length === 0) {
      shelf.append(emptyMessage("No badges yet. Chaos leaves receipts."));
    } else {
      for (const badge of badges) {
        const item = document.createElement("span");
        item.className = "badge";
        const icon = document.createElement("b");
        icon.textContent = badge.icon;
        item.append(icon, badge.label);
        shelf.append(item);
      }
    }
  }

  private renderLanes(player: PlayerView): void {
    const selected = Number(element<HTMLSelectElement>("actorLane").value);
    const grid = element("laneGrid");
    grid.replaceChildren();
    const now = Math.floor(Date.now() / 1000);
    for (const lane of player.lanes) {
      const card = document.createElement("button");
      card.type = "button";
      card.className = `lane-card${lane.index === selected ? " selected" : ""}`;
      card.addEventListener("click", () => {
        element<HTMLSelectElement>("actorLane").value = String(lane.index);
        this.renderSelf();
        this.renderActionPreview();
      });
      const top = document.createElement("span");
      top.className = "lane-top";
      const laneName = document.createElement("span");
      laneName.textContent = `LANE ${lane.index + 1}`;
      const armed = document.createElement("span");
      armed.className = lane.redirectArmed ? "armed-tag" : "";
      armed.textContent = lane.redirectArmed ? "◆ ARMED" : "○ OPEN";
      top.append(laneName, armed);
      const balance = document.createElement("strong");
      balance.textContent = formatDump(lane.balance);
      const guard = laneStat("GUARD", formatDump(lane.guard));
      const locked = laneStat(
        "LOCK",
        lane.lockedUntil > now ? `${formatDump(lane.lockedAmount)} · ${formatCountdown(lane.lockedUntil, now)}` : "CLEAR",
      );
      card.append(top, balance, guard, locked);
      grid.append(card);
    }
  }

  private renderOrders(): void {
    const copy = ACTION_COPY[this.action];
    text("actionTitle", copy.title);
    text("actionDescription", copy.description);
    for (const tab of document.querySelectorAll<HTMLButtonElement>(".action-tab")) {
      tab.classList.toggle("active", tab.dataset.action === this.action);
      tab.setAttribute("aria-selected", String(tab.dataset.action === this.action));
    }
    const isRedirect = this.action === "redirect";
    element("targetField").hidden = isRedirect;
    element("amountField").hidden = isRedirect;
    element("targetLaneField").hidden = isRedirect || this.action === "dump";
    text("actorLaneLabel", this.action === "dump" ? "SOURCE LANE" : this.action === "absorb" ? "DESTINATION LANE" : "GUARD LANE");
    const execute = element<HTMLButtonElement>("executeButton");
    execute.dataset.action = this.action;
    execute.textContent = `EXECUTE ${copy.title}`;
    this.renderSelectedTarget();
    this.renderActionPreview();
    if (this.selfPlayer()) this.renderSelf();
  }

  private renderSelectedTarget(): void {
    const target = this.targetPlayer();
    text(
      "selectedTarget",
      target ? `${target.alias} // ${shortAddress(target.address, 6)}` : "Choose from the global field ↓",
    );
  }

  private renderActionPreview(): void {
    const preview = element("actionPreview");
    preview.replaceChildren();
    const self = this.selfPlayer();
    const actorLaneIndex = Number(element<HTMLSelectElement>("actorLane").value);
    const lane = self?.lanes[actorLaneIndex];
    const now = Math.floor(Date.now() / 1000);
    if (!self || !lane) {
      preview.append(previewLine("STATUS", "Join and claim an allocation to issue orders.", true));
      return;
    }
    if (this.action === "redirect") {
      preview.append(
        previewLine("AVAILABLE GUARD", formatDump(lane.guard)),
        previewLine("CURRENT STATE", lane.redirectArmed ? "ALREADY ARMED" : now >= lane.redirectReadyAt ? "READY" : formatCountdown(lane.redirectReadyAt, now), lane.guard === 0n),
        previewLine("ON HIT", `Up to ${formatDump(lane.guard)} returns to the attacker.`),
        previewLine("SCORE EFFECT", "±0 until an incoming attack resolves"),
      );
      return;
    }

    const target = this.targetPlayer();
    const amount = parseDumpInput(element<HTMLInputElement>("actionAmount").value);
    if (!target || amount === null || amount <= 0n) {
      preview.append(previewLine("STATUS", "Choose a target and enter a valid amount.", true));
      return;
    }
    const minimum = minimumAction(self.startingAllocation);
    const heat = this.action === "dump" ? self.dumpHeat : self.absorbHeat;
    const cost = heatCost(amount, self.startingAllocation);
    const wait = secondsUntilHeatAvailable(heat, now, amount, self.startingAllocation);
    const redirectRisk = this.action === "dump"
      && target.lanes.some((candidate) => candidate.redirectArmed);
    const effect = this.action === "dump"
      ? redirectRisk
        ? `Attempt ${formatDump(amount)} // ${target.alias} receives only the remainder after Guard`
        : `You −${formatDump(amount)} // ${target.alias} +${formatDump(amount)}`
      : `You +${formatDump(amount)} locked // ${target.alias} −${formatDump(amount)}`;
    const scoreDelta = scoreDeltaForBalanceChange(
      this.action === "dump" ? -amount : amount,
      now,
      this.snapshot?.epoch.activeStartsAt ?? now,
      this.snapshot?.epoch.activeEndsAt ?? now,
    );
    preview.append(
      previewLine("PROJECTED MOVE", effect),
      previewLine(
        redirectRisk ? "DELTA IF NONE RETURNS" : "NO-ACTION SCORE DELTA",
        `${formatSignedDump(scoreDelta)} if the balance change lasts to close`,
      ),
      previewLine("HEAT COST", cost > 10_000 ? "OVER CAP" : `${(cost / 100).toFixed(1)}%`, cost > 10_000),
      previewLine("EARLIEST EXECUTION", wait === 0 ? "NOW" : Number.isFinite(wait) ? `IN ${formatCountdown(now + wait, now)}` : "IMPOSSIBLE", wait > 0),
      previewLine("MINIMUM MOVE", formatDump(minimum), amount < minimum),
    );
    if (redirectRisk) {
      preview.append(previewLine(
        "RICOCHET RISK",
        "Target has public armed Guard; the deterministic hit lane is resolved at execution.",
        true,
      ));
    }
  }

  private renderScoreForecast(now: number): void {
    const chart = element<SVGSVGElement>("scoreForecastChart");
    chart.replaceChildren();
    const self = this.selfPlayer();
    if (!this.snapshot || !self) {
      text("scoreForecastNow", "—");
      text("scoreForecastFinal", "—");
      text("scoreForecastDelta", "—");
      text("scoreForecastCaption", "Join the epoch to calculate a score trajectory.");
      return;
    }
    const points = noActionScoreForecast(self, this.snapshot.epoch, now, 9);
    if (points.length === 0) {
      text("scoreForecastCaption", "A forecast is unavailable before an active interval exists.");
      return;
    }
    const first = points[0]!;
    const last = points.at(-1)!;
    text("scoreForecastNow", formatDump(first.score));
    text("scoreForecastFinal", formatDump(last.score));
    text("scoreForecastDelta", formatSignedDump(last.score - first.score));

    const left = 15;
    const right = 305;
    const top = 16;
    const bottom = 105;
    const scores = points.map((point) => point.score);
    const minimum = scores.reduce((value, score) => score < value ? score : value);
    const maximum = scores.reduce((value, score) => score > value ? score : value);
    const range = maximum - minimum || 1n;
    const coordinates = points.map((point, index) => {
      const x = points.length === 1
        ? (left + right) / 2
        : left + ((right - left) * index) / (points.length - 1);
      const scaled = Number(((point.score - minimum) * 10_000n) / range) / 10_000;
      const y = bottom - scaled * (bottom - top);
      return { x, y };
    });
    for (const y of [top, (top + bottom) / 2, bottom]) {
      chart.append(svgNode("line", {
        x1: String(left), y1: String(y), x2: String(right), y2: String(y), class: "forecast-grid-line",
      }));
    }
    if (coordinates.length > 1) {
      const path = coordinates.map((point, index) =>
        `${index === 0 ? "M" : "L"}${point.x.toFixed(2)},${point.y.toFixed(2)}`).join(" ");
      chart.append(
        svgNode("path", {
          d: `${path} L${right},${bottom} L${left},${bottom} Z`,
          class: "forecast-area",
        }),
        svgNode("path", { d: path, class: "forecast-line" }),
      );
    }
    for (const point of coordinates) {
      chart.append(svgNode("circle", {
        cx: point.x.toFixed(2), cy: point.y.toFixed(2), r: "3", class: "forecast-point",
      }));
    }
    chart.append(
      svgText(left, 120, "NOW", "start"),
      svgText(right, 120, "ACTIVE CLOSE", "end"),
    );
    const drift = last.score - first.score;
    text(
      "scoreForecastCaption",
      self.settled
        ? "Final score is settled and cannot move."
        : `If nobody changes your balance, the current burden adds ${formatSignedDump(drift)} by close. Incoming DUMP and future orders are not predicted.`,
    );
  }

  private renderResourceTimeline(now: number): void {
    const list = element<HTMLOListElement>("resourceTimeline");
    list.replaceChildren();
    const self = this.selfPlayer();
    if (!this.snapshot || !self) {
      const item = document.createElement("li");
      item.append(emptyMessage("Join an epoch to expose Heat, lock, and Guard timers."));
      list.append(item);
      return;
    }
    const resources: { label: string; at: number; state?: string }[] = [
      { label: "DUMP Heat", at: heatClearAt(self.dumpHeat, now) },
      { label: "ABSORB Heat", at: heatClearAt(self.absorbHeat, now) },
    ];
    for (const lane of self.lanes) {
      if (lane.lockedAmount > 0n && lane.lockedUntil > now) {
        resources.push({
          label: `Lane ${lane.index + 1} unlock · ${formatDump(lane.lockedAmount)}`,
          at: lane.lockedUntil,
        });
      }
      if (lane.redirectArmed) {
        resources.push({
          label: `Lane ${lane.index + 1} REDIRECT`,
          at: now,
          state: `ARMED · ${formatDump(lane.guard)} Guard`,
        });
      } else if (lane.guard > 0n && lane.redirectReadyAt > now) {
        resources.push({ label: `Lane ${lane.index + 1} rearm`, at: lane.redirectReadyAt });
      }
    }
    resources.sort((left, right) => left.at - right.at || left.label.localeCompare(right.label));
    const horizon = Math.max(1, this.snapshot.epoch.activeEndsAt - now);
    for (const resource of resources) {
      const item = document.createElement("li");
      const row = document.createElement("div");
      row.className = "timeline-row";
      const label = document.createElement("span");
      label.textContent = resource.label;
      const value = document.createElement("b");
      value.textContent = resource.state
        ?? (resource.at <= now
          ? "READY NOW"
          : resource.at > this.snapshot.epoch.activeEndsAt
            ? `AFTER CLOSE · ${formatCountdown(resource.at, now)}`
            : formatCountdown(resource.at, now));
      row.append(label, value);
      const track = document.createElement("div");
      track.className = "timeline-track";
      const fill = document.createElement("span");
      const progress = Math.max(0, Math.min(100, ((resource.at - now) * 100) / horizon));
      fill.style.width = `${resource.at <= now ? 2 : progress}%`;
      if (resource.at <= now) fill.className = "ready";
      track.append(fill);
      item.append(row, track);
      list.append(item);
    }
  }

  private renderThreatView(): void {
    const graph = element<SVGSVGElement>("threatGraph");
    const list = element("threatList");
    const dossier = element("targetDossier");
    graph.replaceChildren();
    list.replaceChildren();
    dossier.replaceChildren();
    const self = this.selfPlayer();
    if (!this.snapshot || !self) {
      graph.append(svgText(160, 92, "NO PLAYER SIGNAL", "middle"));
      list.append(emptyMessage("Join an epoch to build rivalry intelligence."));
      dossier.append(emptyMessage("Choose a wallet tile to inspect it."));
      return;
    }

    const threats = threatViews(this.snapshot.feed, self.address).slice(0, 6);
    const center = { x: 160, y: 90 };
    const maxVolume = threats.reduce(
      (maximum, threat) => {
        const volume = threat.incomingDump + threat.outgoingDump;
        return volume > maximum ? volume : maximum;
      },
      1n,
    );
    threats.forEach((threat, index) => {
      const angle = -Math.PI / 2 + (index * Math.PI * 2) / Math.max(1, threats.length);
      const point = { x: center.x + Math.cos(angle) * 110, y: center.y + Math.sin(angle) * 62 };
      const volume = threat.incomingDump + threat.outgoingDump;
      const width = 1 + Number((volume * 4_000n) / maxVolume) / 1_000;
      const direction = threat.incomingDump > 0n && threat.outgoingDump > 0n
        ? "mutual"
        : threat.outgoingDump > 0n ? "outgoing" : "incoming";
      graph.append(svgNode("line", {
        x1: String(center.x), y1: String(center.y), x2: String(point.x), y2: String(point.y),
        class: `threat-edge ${direction}`, "stroke-width": width.toFixed(2),
      }));
    });
    threats.forEach((threat, index) => {
      const angle = -Math.PI / 2 + (index * Math.PI * 2) / Math.max(1, threats.length);
      const point = { x: center.x + Math.cos(angle) * 110, y: center.y + Math.sin(angle) * 62 };
      const player = this.snapshot?.players.find((candidate) => candidate.address === threat.address);
      graph.append(
        svgNode("circle", {
          cx: String(point.x), cy: String(point.y), r: "10", class: "threat-node",
        }),
        svgText(point.x, point.y + (point.y < center.y ? -15 : 21), player?.alias ?? shortAddress(threat.address), "middle"),
      );
      const chip = document.createElement("button");
      chip.type = "button";
      chip.className = `threat-chip${threat.address === this.selectedTarget ? " selected" : ""}`;
      const name = document.createElement("strong");
      name.textContent = player?.alias ?? shortAddress(threat.address);
      const volume = document.createElement("span");
      volume.textContent = `IN ${formatDump(threat.incomingDump, true)} · OUT ${formatDump(threat.outgoingDump, true)}`;
      chip.append(name, volume);
      chip.addEventListener("click", () => {
        if (!player || player.isSelf) return;
        this.selectedTarget = player.address;
        this.renderSelectedTarget();
        this.renderActionPreview();
        this.renderAtlas();
        this.renderThreatView();
      });
      list.append(chip);
    });
    graph.append(
      svgNode("circle", {
        cx: String(center.x), cy: String(center.y), r: "15", class: "threat-node self",
      }),
      svgText(center.x, center.y + 3, "YOU", "middle"),
    );
    if (threats.length === 0) {
      graph.append(svgText(
        160,
        145,
        this.snapshot.mode === "live" ? "INDEXED HISTORY NOT CONNECTED" : "NO OBSERVED DUMP YET",
        "middle",
      ));
      list.append(emptyMessage(
        this.snapshot.mode === "live"
          ? "Live accounts do not contain historical events; attach the optional indexer for rivalry history."
          : "DUMP exchanges involving you will appear here.",
      ));
    }

    const target = this.targetPlayer();
    if (!target) {
      dossier.append(emptyMessage("Choose a wallet tile to inspect it."));
      return;
    }
    const observed = threats.find((threat) => threat.address === target.address);
    dossier.append(
      dossierCell("TARGET", `${target.alias} // ${shortAddress(target.address, 5)}`, true),
      dossierCell("PROJECTED SCORE", formatDump(target.projectedScore)),
      dossierCell("CURRENT BURDEN", formatDump(target.balance)),
      dossierCell("MEANINGFUL ACTIONS", target.meaningfulActions.toLocaleString("en-US")),
      dossierCell("ARMED LANES", target.lanes.filter((lane) => lane.redirectArmed).length.toString()),
      dossierCell(
        "OBSERVED EXCHANGE",
        observed
          ? `IN ${formatDump(observed.incomingDump, true)} / OUT ${formatDump(observed.outgoingDump, true)}`
          : "NO DIRECT DUMP IN FEED",
      ),
    );
  }

  private renderAtlas(): void {
    if (!this.snapshot) return;
    for (const chip of document.querySelectorAll<HTMLButtonElement>(".chip")) {
      chip.classList.toggle("active", chip.dataset.filter === this.atlasFilter);
    }
    const ranked = rankPlayers(this.snapshot.players.filter((player) => player.allocationClaimed));
    let candidates = ranked;
    if (this.atlasFilter === "leaders") {
      candidates = ranked.slice(0, Math.max(8, this.snapshot.epoch.winnerCount));
    } else if (this.atlasFilter === "burdened") {
      candidates = [...ranked].reverse().slice(0, 24);
    } else if (ranked.length > 120) {
      candidates = [...ranked.slice(0, 60), ...ranked.slice(-60)];
    }
    const grid = element("territoryGrid");
    grid.replaceChildren();
    candidates.forEach((player) => {
      const rank = ranked.findIndex((candidate) => candidate.address === player.address) + 1;
      const territory = document.createElement("button");
      territory.type = "button";
      const sizeClass = rank <= 3 ? " large tall" : rank <= 7 ? " large" : "";
      territory.className = `territory level-${territoryLevel(player)}${sizeClass}${player.isSelf ? " self" : ""}${player.address === this.selectedTarget ? " selected" : ""}`;
      territory.disabled = player.isSelf === true;
      territory.title = `${player.alias}: score ${formatDump(player.projectedScore)}`;
      territory.addEventListener("click", () => {
        this.selectedTarget = player.address;
        this.renderSelectedTarget();
        this.renderActionPreview();
        this.renderAtlas();
        this.renderThreatView();
        document.querySelector(".orders")?.scrollIntoView({ behavior: "smooth", block: "center" });
      });
      const rankLabel = document.createElement("span");
      rankLabel.className = "rank";
      rankLabel.textContent = `#${rank}`;
      const alias = document.createElement("strong");
      alias.textContent = player.isSelf ? `${player.alias} (YOU)` : player.alias;
      const score = document.createElement("small");
      score.textContent = formatDump(player.projectedScore, true);
      territory.append(rankLabel, alias, score);
      if (player.lanes.some((lane) => lane.redirectArmed)) {
        const shield = document.createElement("span");
        shield.className = "shield";
        shield.textContent = "◇";
        shield.title = "At least one REDIRECT is armed";
        territory.append(shield);
      }
      grid.append(territory);
    });
  }

  private renderLeaderboard(): void {
    if (!this.snapshot) return;
    const ranked = rankPlayers(this.snapshot.players.filter((player) => player.eligible));
    const visible = ranked.slice(0, 10);
    const self = ranked.find((player) => player.isSelf);
    if (self && !visible.includes(self)) visible.push(self);
    const list = element("leaderboardList");
    list.replaceChildren();
    for (const player of visible) {
      const rank = ranked.indexOf(player) + 1;
      const item = document.createElement("li");
      if (player.isSelf) item.className = "self";
      const number = document.createElement("span");
      number.textContent = `#${rank}`;
      const identity = document.createElement("span");
      identity.className = "leader-identity";
      const alias = document.createElement("strong");
      alias.textContent = player.alias;
      const address = document.createElement("small");
      address.textContent = shortAddress(player.address, 5);
      identity.append(alias, address);
      const score = document.createElement("span");
      score.className = "leader-score";
      score.textContent = formatDump(player.projectedScore);
      const impact = document.createElement("span");
      impact.textContent = `${Number(player.impactPpm / 10_000n).toLocaleString("en-US")}%`;
      item.append(number, identity, score, impact);
      list.append(item);
    }
  }

  private renderFeed(): void {
    if (!this.snapshot) return;
    const list = element("feedList");
    list.replaceChildren();
    for (const event of this.snapshot.feed.slice(0, 10)) {
      const item = document.createElement("li");
      item.className = event.kind;
      const message = document.createElement("span");
      message.textContent = event.message;
      const time = document.createElement("time");
      time.dateTime = new Date(event.timestamp * 1_000).toISOString();
      time.textContent = relativeTime(event.timestamp);
      item.append(message, time);
      list.append(item);
    }
  }

  private setHeat(kind: "dump" | "absorb", value: number): void {
    const percent = Math.max(0, Math.min(100, value / 100));
    text(`${kind}HeatLabel`, `${percent.toFixed(0)}%`);
    element<HTMLElement>(`${kind}HeatBar`).style.width = `${percent}%`;
  }

  private syncButtons(): void {
    const epoch = this.snapshot?.epoch;
    const self = this.selfPlayer();
    element<HTMLButtonElement>("connectButton").disabled = this.busy;
    element<HTMLButtonElement>("registerButton").disabled = this.busy || !epoch || epoch.phase !== "registration" || Boolean(self);
    element<HTMLButtonElement>("revealButton").disabled = this.busy || !epoch || epoch.phase !== "reveal" || !self || self.revealed;
    element<HTMLButtonElement>("claimButton").disabled = this.busy || !epoch || !["planning", "active"].includes(epoch.phase) || !self || self.allocationClaimed;
    element<HTMLButtonElement>("executeButton").disabled = this.busy || !epoch || epoch.phase !== "active" || !self?.allocationClaimed;
    element<HTMLButtonElement>("advanceButton").disabled = this.busy || !epoch;
    element<HTMLButtonElement>("settleButton").disabled = this.busy || !epoch || epoch.phase !== "settling" || epoch.settledCount >= epoch.participantCount;
    element<HTMLButtonElement>("rewardButton").disabled = this.busy || !epoch || epoch.phase !== "complete" || !self?.isWinner || self.rewardClaimed;
    element<HTMLButtonElement>("bondButton").disabled = this.busy || !epoch || !["complete", "cancelled"].includes(epoch.phase) || !self || self.bondClaimed;
    element<HTMLButtonElement>("badgesButton").disabled = this.busy || !epoch || epoch.phase !== "complete" || !self?.settled;
    element<HTMLButtonElement>("backupSecretButton").disabled = this.busy || !this.snapshot?.walletAddress || !self;
    element<HTMLInputElement>("restoreSecretInput").disabled = this.busy || !this.snapshot?.walletAddress || !self;
  }

  private selfPlayer(): PlayerView | undefined {
    return this.snapshot?.players.find((player) => player.isSelf);
  }

  private targetPlayer(): PlayerView | undefined {
    return this.snapshot?.players.find((player) => player.address === this.selectedTarget);
  }

  private setStatus(message: string, kind: "normal" | "success" | "error" = "normal"): void {
    const bar = element("statusBar");
    bar.textContent = message;
    bar.dataset.kind = kind;
  }
}

function element<T extends Element = HTMLElement>(id: string): T {
  const found = document.getElementById(id);
  if (!found) throw new Error(`Missing required element #${id}`);
  return found as unknown as T;
}

function text(id: string, value: string): void {
  element(id).textContent = value;
}

function laneStat(label: string, value: string): HTMLElement {
  const row = document.createElement("span");
  row.className = "lane-stat";
  const name = document.createElement("small");
  name.textContent = label;
  const amount = document.createElement("b");
  amount.textContent = value;
  row.append(name, amount);
  return row;
}

function previewLine(label: string, value: string, warning = false): HTMLElement {
  const row = document.createElement("div");
  if (warning) row.className = "warning";
  const name = document.createElement("span");
  name.textContent = `${label}: `;
  const result = document.createElement("strong");
  result.textContent = value;
  row.append(name, result);
  return row;
}

function dossierCell(label: string, value: string, wide = false): HTMLElement {
  const cell = document.createElement("span");
  if (wide) cell.className = "dossier-wide";
  const name = document.createElement("small");
  name.textContent = label;
  const result = document.createElement("b");
  result.textContent = value;
  cell.append(name, result);
  return cell;
}

function svgNode<K extends keyof SVGElementTagNameMap>(
  tag: K,
  attributes: Record<string, string>,
): SVGElementTagNameMap[K] {
  const node = document.createElementNS("http://www.w3.org/2000/svg", tag);
  for (const [name, value] of Object.entries(attributes)) node.setAttribute(name, value);
  return node;
}

function svgText(
  x: number,
  y: number,
  value: string,
  anchor: "start" | "middle" | "end",
): SVGTextElement {
  const node = svgNode("text", {
    x: String(x),
    y: String(y),
    class: "svg-label",
    "text-anchor": anchor,
  });
  node.textContent = value;
  return node;
}

function formatSignedDump(value: bigint): string {
  return value > 0n ? `+${formatDump(value)}` : value < 0n ? formatDump(value) : "±0";
}

function emptyMessage(message: string): HTMLElement {
  const node = document.createElement("span");
  node.className = "empty-copy";
  node.textContent = message;
  return node;
}

function phaseCaption(phase: StrategySnapshot["epoch"]["phase"]): string {
  return {
    registration: "Commit a secret and reserve a place",
    reveal: "Reveal secrets; non-revealers inherit 10B DUMP",
    planning: "Inspect the chaos and prepare defenses",
    active: "Weighted score is live",
    settling: "Permissionless final-score processing",
    complete: "Winners and rewards are final",
    cancelled: "Entry bonds can be reclaimed",
  }[phase];
}

function transitionLabel(phase: StrategySnapshot["epoch"]["phase"]): string {
  return {
    registration: "OPEN REVEAL / CANCEL",
    reveal: "SEAL RANDOMNESS / CANCEL",
    planning: "START ACTIVE PLAY",
    active: "BEGIN SETTLEMENT",
    settling: "COMPLETE EPOCH",
    complete: "OPEN NEXT EPOCH",
    cancelled: "OPEN NEXT EPOCH",
  }[phase];
}

function relativeTime(timestamp: number): string {
  const seconds = Math.max(0, Math.floor(Date.now() / 1_000) - timestamp);
  if (seconds < 60) return `${seconds}s ago`;
  if (seconds < 3_600) return `${Math.floor(seconds / 60)}m ago`;
  return `${Math.floor(seconds / 3_600)}h ago`;
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

void new StrategyRoom().start();
