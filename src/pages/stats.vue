<script setup lang="ts">
  import { useFileStore } from '@/stores/fileStore';

  const store = useFileStore();
  const { query, searchHistory } = storeToRefs(store);

  const running = ref(false);
  const hasRun = ref(false);
  const photoCount = ref<number | null>(null);

  interface Metrics {
    count: number;
    dateSpan: string;
    rated: number;
    tagged: number;
  }
  const metrics = ref<Metrics | null>(null);

  async function runStats() {
    if (running.value) return;
    running.value = true;
    // TODO: replace with invoke('get_stats', { query: query.value })
    await new Promise(resolve => setTimeout(resolve, 700));
    photoCount.value = 412;
    metrics.value = {
      count: 412,
      dateSpan: 'Mar 2021 – Jun 2024',
      rated: 87,
      tagged: 394,
    };
    hasRun.value = true;
    running.value = false;
  }

  function applyHistory(entry: string[]) {
    query.value = [...entry];
    store.pushHistory(entry);
  }

  const comboItems = computed(() =>
    searchHistory.value.map(entry => ({ type: 'history', value: entry })),
  );

  // ── Stub chart data ────────────────────────────────────────────────────────

  const MONTH_LABELS = [
    'Jan',
    'Feb',
    'Mar',
    'Apr',
    'May',
    'Jun',
    'Jul',
    'Aug',
    'Sep',
    'Oct',
    'Nov',
    'Dec',
  ];
  const SHOT_DATA = [12, 8, 24, 36, 18, 45, 52, 38, 29, 14, 8, 20];
  const RATING_DATA = [32, 71, 125, 95, 89]; // counts for 1★ – 5★

  const maxShots = Math.max(...SHOT_DATA);
  const maxRating = Math.max(...RATING_DATA);

  // viewBox "0 0 480 150" — bar area y 0–115, baseline y 115, labels y 133
  const shotBars = SHOT_DATA.map((val, i) => {
    const slotW = 40;
    const barW = 22;
    const areaH = 110;
    const barH = Math.round((val / maxShots) * areaH);
    return {
      x: i * slotW + (slotW - barW) / 2,
      y: 115 - barH,
      w: barW,
      h: barH,
      labelX: i * slotW + slotW / 2,
      label: MONTH_LABELS[i]!,
    };
  });

  // viewBox "0 0 200 150"
  const ratingBars = RATING_DATA.map((val, i) => {
    const slotW = 40;
    const barW = 24;
    const areaH = 110;
    const barH = Math.round((val / maxRating) * areaH);
    return {
      x: i * slotW + (slotW - barW) / 2,
      y: 115 - barH,
      w: barW,
      h: barH,
      labelX: i * slotW + slotW / 2,
      stars: i + 1,
    };
  });
</script>

<template>
  <div class="stats-page">
    <!-- ── Scope Panel ──────────────────────────────────────────────────────── -->
    <div class="scope-panel">
      <span class="scope-label">SCOPE</span>

      <v-combobox
        v-model="query"
        chips
        class="scope-field"
        clearable
        density="compact"
        hide-details
        :items="comboItems"
        multiple
        placeholder="All photos"
        variant="outlined"
      >
        <template #item="{ props, item }">
          <v-list-item v-bind="props" title="" @click="applyHistory(item.value)">
            <template #prepend>
              <v-icon size="16">mdi-history</v-icon>
            </template>
            <v-chip-group>
              <v-chip v-for="(token, j) in item.value" :key="j" size="small">
                {{ token }}
              </v-chip>
            </v-chip-group>
          </v-list-item>
        </template>
      </v-combobox>

      <div class="scope-count">
        <template v-if="!running && photoCount !== null">
          <span class="count-number">{{ photoCount.toLocaleString() }}</span>
          <span class="count-unit">{{ photoCount === 1 ? 'photo' : 'photos' }}</span>
        </template>
        <span v-else class="count-dash">—</span>
      </div>

      <v-btn
        class="run-btn"
        color="primary"
        :loading="running"
        variant="flat"
        @click="runStats"
      >
        Run Statistics
      </v-btn>
    </div>

    <!-- ── Zero-results notice ────────────────────────────────────────────── -->
    <div v-if="hasRun && photoCount === 0" class="scope-empty">No photos match this scope.</div>

    <!-- ── Metrics Strip ──────────────────────────────────────────────────── -->
    <div class="metrics-strip">
      <div class="metric-block">
        <span class="metric-value">
          <template v-if="hasRun">{{ metrics!.count.toLocaleString() }}</template>
          <span v-else class="metric-skel" style="width: 52px" />
        </span>
        <span class="metric-label">in scope</span>
      </div>

      <div class="metric-divider" />

      <div class="metric-block">
        <span class="metric-value metric-value--date">
          <template v-if="hasRun">{{ metrics!.dateSpan }}</template>
          <span v-else class="metric-skel" style="width: 118px" />
        </span>
        <span class="metric-label">date range</span>
      </div>

      <div class="metric-divider" />

      <div class="metric-block">
        <span class="metric-value">
          <template v-if="hasRun">{{ metrics!.rated }}</template>
          <span v-else class="metric-skel" style="width: 40px" />
        </span>
        <span class="metric-label">rated</span>
      </div>

      <div class="metric-divider" />

      <div class="metric-block">
        <span class="metric-value">
          <template v-if="hasRun">{{ metrics!.tagged }}</template>
          <span v-else class="metric-skel" style="width: 40px" />
        </span>
        <span class="metric-label">tagged</span>
      </div>
    </div>

    <!-- ── Charts ─────────────────────────────────────────────────────────── -->
    <div class="charts-grid">
      <section class="chart-panel">
        <h2 class="chart-title">Shots per month</h2>
        <div class="chart-body">
          <div v-if="!hasRun" class="chart-skel" />
          <svg
            v-else
            class="bar-chart"
            preserveAspectRatio="none"
            viewBox="0 0 480 148"
            xmlns="http://www.w3.org/2000/svg"
          >
            <!-- Baseline -->
            <line
              stroke="oklch(30% 0.008 245)"
              stroke-width="0.75"
              x1="0"
              x2="480"
              y1="115"
              y2="115"
            />
            <!-- Mid grid -->
            <line
              stroke="oklch(25% 0.006 245)"
              stroke-dasharray="4 4"
              stroke-width="0.5"
              x1="0"
              x2="480"
              y1="60"
              y2="60"
            />
            <!-- Bars -->
            <rect
              v-for="bar in shotBars"
              :key="bar.label"
              fill="oklch(65% 0.14 245 / 0.52)"
              :height="bar.h"
              rx="2"
              :width="bar.w"
              :x="bar.x"
              :y="bar.y"
            />
            <!-- Month labels -->
            <text
              v-for="bar in shotBars"
              :key="`lbl-${bar.label}`"
              fill="oklch(55% 0.006 245)"
              font-family="Roboto, sans-serif"
              font-size="11"
              text-anchor="middle"
              :x="bar.labelX"
              y="135"
            >
              {{ bar.label }}
            </text>
          </svg>
        </div>
      </section>

      <section class="chart-panel">
        <h2 class="chart-title">Rating distribution</h2>
        <div class="chart-body">
          <div v-if="!hasRun" class="chart-skel" />
          <svg
            v-else
            class="bar-chart"
            preserveAspectRatio="none"
            viewBox="0 0 200 148"
            xmlns="http://www.w3.org/2000/svg"
          >
            <!-- Baseline -->
            <line
              stroke="oklch(30% 0.008 245)"
              stroke-width="0.75"
              x1="0"
              x2="200"
              y1="115"
              y2="115"
            />
            <!-- Mid grid -->
            <line
              stroke="oklch(25% 0.006 245)"
              stroke-dasharray="4 4"
              stroke-width="0.5"
              x1="0"
              x2="200"
              y1="60"
              y2="60"
            />
            <!-- Bars — increasing opacity toward 5★ -->
            <rect
              v-for="bar in ratingBars"
              :key="bar.stars"
              :fill="`oklch(65% 0.14 245 / ${(0.28 + (bar.stars / 5) * 0.55).toFixed(2)})`"
              :height="bar.h"
              rx="2"
              :width="bar.w"
              :x="bar.x"
              :y="bar.y"
            />
            <!-- Star labels -->
            <text
              v-for="bar in ratingBars"
              :key="`lbl-${bar.stars}`"
              fill="oklch(55% 0.006 245)"
              font-family="Roboto, sans-serif"
              font-size="11"
              text-anchor="middle"
              :x="bar.labelX"
              y="135"
            >
              {{ bar.stars }}★
            </text>
          </svg>
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
  .stats-page {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
  }

  /* ── Scope Panel ─────────────────────────────────────────────────────────── */

  .scope-panel {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 16px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
    position: sticky;
    top: 0;
    z-index: 2;
  }

  .scope-label {
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.1em;
    color: var(--color-text-secondary);
    white-space: nowrap;
    flex-shrink: 0;
    user-select: none;
  }

  .scope-field {
    flex: 1;
    min-width: 0;
  }

  .scope-count {
    display: flex;
    align-items: baseline;
    gap: 4px;
    flex-shrink: 0;
    white-space: nowrap;
    min-width: 80px;
    justify-content: flex-end;
  }

  .count-number {
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text-primary);
  }

  .count-unit {
    font-size: 12px;
    color: var(--color-text-secondary);
  }

  .count-dash {
    font-size: 14px;
    color: var(--color-text-secondary);
  }

  .run-btn {
    flex-shrink: 0;
    letter-spacing: 0.04em;
    font-size: 13px;
  }

  /* ── Zero-results ────────────────────────────────────────────────────────── */

  .scope-empty {
    padding: 10px 24px;
    font-size: 13px;
    color: var(--color-text-secondary);
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  /* ── Metrics Strip ───────────────────────────────────────────────────────── */

  .metrics-strip {
    display: flex;
    align-items: center;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .metric-block {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 5px;
    padding: 20px 16px;
  }

  .metric-value {
    font-size: 20px;
    font-weight: 700;
    line-height: 1.2;
    color: var(--color-text-primary);
    min-height: 26px;
    display: flex;
    align-items: center;
  }

  .metric-value--date {
    font-size: 14px;
    font-weight: 500;
    letter-spacing: 0.01em;
  }

  .metric-label {
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--color-text-secondary);
  }

  .metric-divider {
    width: 1px;
    height: 40px;
    background: var(--color-border);
    flex-shrink: 0;
  }

  /* Skeleton shimmer */
  .metric-skel {
    display: inline-block;
    height: 22px;
    border-radius: 3px;
    background: linear-gradient(
      90deg,
      oklch(21% 0.006 245) 25%,
      oklch(27% 0.007 245) 50%,
      oklch(21% 0.006 245) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s ease-in-out infinite;
  }

  @keyframes shimmer {
    from {
      background-position: 200% 0;
    }
    to {
      background-position: -200% 0;
    }
  }

  /* ── Charts ──────────────────────────────────────────────────────────────── */

  .charts-grid {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 16px;
    padding: 24px;
  }

  .chart-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .chart-title {
    font-size: 14px;
    font-weight: 500;
    line-height: 1.4;
    letter-spacing: 0.01em;
    color: var(--color-text-primary);
    margin: 0;
  }

  .chart-body {
    flex: 1;
  }

  .chart-skel {
    width: 100%;
    height: 160px;
    border-radius: 4px;
    background: linear-gradient(
      90deg,
      oklch(21% 0.006 245) 25%,
      oklch(27% 0.007 245) 50%,
      oklch(21% 0.006 245) 75%
    );
    background-size: 200% 100%;
    animation: shimmer 1.5s ease-in-out infinite;
  }

  .bar-chart {
    display: block;
    width: 100%;
    height: 160px;
  }
</style>
