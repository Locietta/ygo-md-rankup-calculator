<template>
  <v-container class="calculator-page py-6 py-md-10">
    <section class="hero mb-3 mb-md-4">
      <p class="hero-kicker">Yu-Gi-Oh! Master Duel</p>
      <h1 class="hero-title">段位升降与大段期望计算器</h1>
    </section>

    <v-alert
      class="intro-alert mb-6 mb-md-8"
      variant="tonal"
      density="compact"
      icon="mdi-information-outline"
      color="info"
    >
      <p class="intro-alert-text">
        原理可以参看我的博客文章：
        <a
          href="https://blog.locietta.xyz/posts/possibility-calculation-2"
          target="_blank"
          rel="noopener noreferrer"
        >
          游戏王MD中的概率问题（二）：Bo2与升降段
        </a>
      </p>
    </v-alert>

    <v-row class="panel-row align-stretch">
      <v-col cols="12" lg="5">
        <v-sheet class="panel panel-input" rounded="xl" elevation="3">
          <div class="panel-header">
            <v-icon size="22" color="primary">mdi-tune-variant</v-icon>
            <h2>参数输入</h2>
          </div>

          <v-form ref="formRef" @submit.prevent="handleCalculate">
            <v-select
              v-model="rankType"
              :items="rankTypeOptions"
              item-title="label"
              item-value="value"
              :menu-props="selectMenuProps"
              label="当前大段类型"
              variant="outlined"
              density="comfortable"
              class="mb-4"
              prepend-inner-icon="mdi-castle"
            />

            <v-select
              v-model="currentSubtier"
              :items="subtierOptions"
              item-title="label"
              item-value="value"
              :menu-props="selectMenuProps"
              label="当前小段"
              variant="outlined"
              density="comfortable"
              class="mb-4"
              prepend-inner-icon="mdi-ladder"
            />

            <v-text-field
              v-model.number="currentNetWins"
              label="当前小段位置（胜场数）"
              type="number"
              :min="minNetWins"
              :max="maxNetWins"
              :step="1"
              :rules="currentNetWinsRules"
              variant="outlined"
              density="comfortable"
              class="mb-3"
              prepend-inner-icon="mdi-chart-line-variant"
              :hint="currentNetWinsHint"
              persistent-hint
            />

            <v-card variant="flat" class="mb-2 winrate-box" rounded="lg">
              <div class="d-flex justify-space-between align-center mb-2 winrate-head">
                <label class="winrate-label">平均胜率</label>
                <strong class="winrate-value">{{ winRate.toFixed(1) }}%</strong>
              </div>
              <v-slider
                v-model="winRate"
                :min="0"
                :max="100"
                :step="0.1"
                color="primary"
                hide-details
                class="winrate-slider"
              />
              <v-text-field
                v-model.number="winRate"
                type="number"
                suffix="%"
                :rules="winRateRules"
                variant="outlined"
                density="compact"
                hide-details="auto"
                class="mt-3 winrate-input"
              />
            </v-card>

            <v-btn
              type="submit"
              block
              size="large"
              rounded="lg"
              color="primary"
              :loading="isLoading"
              class="text-none"
            >
              <v-icon start>mdi-calculator-variant-outline</v-icon>
              开始计算
            </v-btn>
          </v-form>
        </v-sheet>
      </v-col>

      <v-col cols="12" lg="7">
        <v-sheet class="panel panel-output" rounded="xl" elevation="3">
          <div class="panel-header">
            <v-icon size="22" color="secondary">mdi-chart-areaspline</v-icon>
            <h2>结果面板</h2>
          </div>

          <template v-if="results">
            <v-row class="metric-grid" density="compact">
              <v-col cols="12" sm="6">
                <v-card class="metric metric-a" rounded="lg" variant="tonal">
                  <p class="metric-title">离开当前小段期望局数</p>
                  <p class="metric-value">{{ formatMatches(results.leaveCurrentSegmentExpected) }}</p>
                </v-card>
              </v-col>
              <v-col cols="12" sm="6">
                <v-card class="metric metric-b" rounded="lg" variant="tonal">
                  <p class="metric-title">当前小段升段概率</p>
                  <p class="metric-value">{{ formatPercent(results.currentSegmentPromotionProbability) }}</p>
                </v-card>
              </v-col>
              <v-col cols="12" sm="6">
                <v-card class="metric metric-c" rounded="lg" variant="tonal">
                  <p class="metric-title">到当前大段 I 的期望局数</p>
                  <p class="metric-value">{{ formatMatches(results.expectedToCurrentTierI) }}</p>
                </v-card>
              </v-col>
              <v-col cols="12" sm="6">
                <v-card class="metric metric-d" rounded="lg" variant="tonal">
                  <p class="metric-title">到下一个大段 V 的期望局数</p>
                  <p class="metric-value">
                    {{
                      isHighestBigTier
                        ? "已是最高段位"
                        : formatMatches(results.expectedToNextBigTierV)
                    }}
                  </p>
                </v-card>
              </v-col>
            </v-row>

            <div class="meta mt-4">
              <v-chip size="small" color="primary" variant="outlined" class="mr-2 mb-2">
                当前小段：{{ currentSubtierLabel }}
              </v-chip>
              <v-chip size="small" color="primary" variant="outlined" class="mr-2 mb-2">
                K = {{ rankType }}
              </v-chip>
              <v-chip size="small" color="primary" variant="outlined" class="mr-2 mb-2">
                胜率 = {{ winRate.toFixed(1) }}%
              </v-chip>
              <v-chip size="small" color="secondary" variant="tonal" class="mb-2">
                求解后端：{{ solverMode }}
              </v-chip>
            </div>
          </template>

          <template v-else>
            <v-card class="empty-state" rounded="lg" variant="outlined">
              <v-icon size="42" color="medium-emphasis">mdi-target-variant</v-icon>
              <p>输入参数后点击“开始计算”，这里会显示四项指标。</p>
            </v-card>
          </template>
        </v-sheet>
      </v-col>
    </v-row>

    <v-snackbar
      v-model="snackbar.visible"
      :timeout="snackbar.timeout"
      :color="snackbar.color"
      location="top right"
      rounded="lg"
      elevation="5"
    >
      <v-icon start class="mr-2">{{ snackbar.icon }}</v-icon>
      {{ snackbar.text }}
      <template #actions>
        <v-btn variant="text" @click="snackbar.visible = false">关闭</v-btn>
      </template>
    </v-snackbar>
  </v-container>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import init, { calculate_rank_progress_stats } from "wasm";

type RankK = 4 | 5;
type SubtierValue = 0 | 1 | 2 | 3 | 4; // 0:V, 4:I

interface SnackbarState {
  visible: boolean;
  text: string;
  timeout: number;
  color: string;
  icon: string;
}

interface SegmentStats {
  expectedMatches: number;
  promotionProbability: number;
}

interface CalculationResult {
  leaveCurrentSegmentExpected: number;
  currentSegmentPromotionProbability: number;
  expectedToCurrentTierI: number;
  expectedToNextBigTierV: number;
}

interface WasmRankProgressStats {
  leave_current_segment_expected: number;
  current_segment_promotion_probability: number;
  expected_to_current_tier_i: number;
  expected_to_next_big_tier_v: number;
}

interface FormValidator {
  validate: () => Promise<{ valid: boolean }>;
}

const rankTypeOptions: Array<{ label: string; value: RankK; isHighestBigTier: boolean }> = [
  { label: "白金/钻石段（ 4 胜场升段）", value: 4, isHighestBigTier: false },
  { label: "大师段（ 5 胜场升段）", value: 5, isHighestBigTier: true },
];

const defaultRankTypeConfig = rankTypeOptions[0] ?? {
  label: "白金/钻石段（ 4 胜场升段）",
  value: 4 as RankK,
  isHighestBigTier: false,
};

const selectMenuProps = {
  contentClass: "rank-select-menu",
};

const subtierOptions: Array<{ label: string; value: SubtierValue }> = [
  { label: "V", value: 0 },
  { label: "IV", value: 1 },
  { label: "III", value: 2 },
  { label: "II", value: 3 },
  { label: "I", value: 4 },
];

const EPS = 1e-12;

const formRef = ref<FormValidator | null>(null);
const rankType = ref<RankK>(4);
const currentSubtier = ref<SubtierValue>(0);
const currentNetWins = ref<number>(0);
const winRate = ref<number>(55);

const isLoading = ref(false);
const solverMode = ref<"WASM" | "JS">("WASM");
const wasmReady = ref(false);
const results = ref<CalculationResult | null>(null);

const snackbar = ref<SnackbarState>({
  visible: false,
  text: "",
  timeout: 4000,
  color: "info",
  icon: "mdi-information",
});

const showSnackbar = (
  text: string,
  color = "info",
  icon = "mdi-information",
  timeout = 4000
): void => {
  snackbar.value = {
    visible: true,
    text,
    timeout,
    color,
    icon,
  };
};

onMounted(async () => {
  try {
    await init();
    wasmReady.value = true;
    solverMode.value = "WASM";
  } catch (error) {
    console.error("WASM init failed, fallback to JS formulas", error);
    wasmReady.value = false;
    solverMode.value = "JS";
    showSnackbar("WASM 加载失败，已自动切换 JS ", "warning", "mdi-alert");
  }
});

const maxNetWins = computed(() => rankType.value - 1);
const minNetWins = computed(() => (currentSubtier.value === 0 ? 0 : -2));
const currentRankTypeConfig = computed(
  () => rankTypeOptions.find((option) => option.value === rankType.value) ?? defaultRankTypeConfig
);
const isHighestBigTier = computed(() => currentRankTypeConfig.value.isHighestBigTier);
const currentNetWinsHint = computed(() => {
  const upperBound = maxNetWins.value;

  if (currentSubtier.value === 0) {
    return `范围为 0 到 ${upperBound}`; // V 是当前大段最低段位，失败不会有惩罚
  }

  return `范围为 -2 到 ${upperBound}`; // IV、III、II、I 失败会掉段，最差会掉到 V 段的 0 胜
});

const currentSubtierLabel = computed(
  () => subtierOptions.find((option) => option.value === currentSubtier.value)?.label ?? "-"
);

const winRateRules = [
  (value: unknown) => value !== null || "胜率不能为空",
  (value: unknown) => Number(value) >= 0 || "胜率不能低于 0",
  (value: unknown) => Number(value) <= 100 || "胜率不能高于 100",
];

const currentNetWinsRules = computed(() => [
  (value: unknown) => value !== null || "胜场数不能为空",
  (value: unknown) => Number.isInteger(Number(value)) || "胜场数必须是整数",
  (value: unknown) =>
    Number(value) >= minNetWins.value ||
    (currentSubtier.value === 0 ? "V 段位胜场数不能小于 0" : "胜场数不能小于 -2"),
  (value: unknown) => Number(value) <= maxNetWins.value || `胜场数不能大于 ${maxNetWins.value}`,
]);

const clamp = (value: number, min: number, max: number): number => {
  if (value < min) return min;
  if (value > max) return max;
  return value;
};

watch([currentSubtier, rankType], () => {
  const numeric = Math.trunc(Number(currentNetWins.value));
  if (!Number.isFinite(numeric)) {
    currentNetWins.value = minNetWins.value;
    return;
  }

  currentNetWins.value = clamp(numeric, minNetWins.value, maxNetWins.value);
});

watch(currentNetWins, (value) => {
  const numeric = Math.trunc(Number(value));
  if (!Number.isFinite(numeric)) {
    return;
  }

  const bounded = clamp(numeric, minNetWins.value, maxNetWins.value);
  if (bounded !== value) {
    currentNetWins.value = bounded;
  }
});

const createMartingaleFn = (p: number) => {
  return (n: number): number => {
    if (n >= 0) {
      if (p === 0.5) {
        return n;
      }

      const factor = p / (2.0 * p - 1.0);
      const ratio = (1.0 - p) / p;
      return factor * (1.0 - ratio ** n);
    }

    return 1.0 - (1.0 - p) ** n;
  };
};

const calculateNonFloorSegmentExpectedJs = (
  pValue: number,
  kValue: RankK,
  netWinsValue: number
): number => {
  const start = Math.floor(clamp(netWinsValue, -2, kValue - 1));
  const qValue = 1 - pValue;

  const states: number[] = [];
  for (let state = -2; state <= kValue - 1; state += 1) {
    states.push(state);
  }

  const size = states.length;
  const matrix: number[][] = Array.from({ length: size }, () => Array(size).fill(0));
  const vector: number[] = Array(size).fill(1);

  for (let row = 0; row < size; row += 1) {
    const rowRef = matrix[row];
    const state = states[row];
    if (!rowRef || state === undefined) {
      return Number.POSITIVE_INFINITY;
    }

    rowRef[row] = 1;

    const nextWin = state < 0 ? 1 : state + 1;
    if (nextWin >= -2 && nextWin <= kValue - 1) {
      rowRef[nextWin + 2] = (rowRef[nextWin + 2] ?? 0) - pValue;
    }

    const nextLose = state - 1;
    if (nextLose >= -2 && nextLose <= kValue - 1) {
      rowRef[nextLose + 2] = (rowRef[nextLose + 2] ?? 0) - qValue;
    }
  }

  const solved = solveLinearSystem(matrix, vector);
  if (!solved) {
    return Number.POSITIVE_INFINITY;
  }

  return solved[start + 2] ?? Number.POSITIVE_INFINITY;
};

const calculateSegmentStatsJs = (pRaw: number, kValue: RankK, netWinsValue: number): SegmentStats => {
  const pValue = clamp(pRaw, 0, 1);

  if (pValue <= 0.0) {
    return {
      expectedMatches: netWinsValue >= -2 ? 3.0 + netWinsValue : 0,
      promotionProbability: 0,
    };
  }

  if (pValue >= 1.0) {
    if (netWinsValue >= kValue) {
      return { expectedMatches: 0, promotionProbability: 1 };
    }

    if (netWinsValue < 0) {
      return { expectedMatches: kValue as number, promotionProbability: 1 };
    }

    return {
      expectedMatches: kValue - netWinsValue,
      promotionProbability: 1,
    };
  }

  const martingale = createMartingaleFn(pValue);
  const promotionProbability =
    (martingale(netWinsValue) - martingale(-3)) / (martingale(kValue) - martingale(-3));

  const expectedMatches = calculateNonFloorSegmentExpectedJs(pValue, kValue, netWinsValue);

  return {
    expectedMatches: Math.max(expectedMatches, 0),
    promotionProbability: clamp(promotionProbability, 0, 1),
  };
};

const calculateVFloorSegmentStatsJs = (
  pRaw: number,
  kValue: RankK,
  netWinsValue: number
): SegmentStats => {
  const pValue = clamp(pRaw, 0, 1);
  const qValue = 1 - pValue;
  const target = Number(kValue);
  const start = Math.floor(clamp(netWinsValue, 0, target - 1));

  if (pValue <= 0) {
    return {
      expectedMatches: Number.POSITIVE_INFINITY,
      promotionProbability: 0,
    };
  }

  if (pValue >= 1) {
    return {
      expectedMatches: target - start,
      promotionProbability: 1,
    };
  }

  const size = target; // unknowns E0..E(k-1), Ek=0
  const matrix: number[][] = Array.from({ length: size }, () => Array(size).fill(0));
  const vector: number[] = Array(size).fill(1);

  for (let i = 0; i < size; i += 1) {
    const row = matrix[i];
    if (!row) {
      return {
        expectedMatches: Number.POSITIVE_INFINITY,
        promotionProbability: 1,
      };
    }

    if (i === 0) {
      row[0] = pValue;
      if (size > 1) {
        row[1] = -pValue;
      }
      continue;
    }

    row[i] = 1;
    row[i - 1] = -qValue;
    if (i + 1 < size) {
      row[i + 1] = -pValue;
    }
  }

  const solved = solveLinearSystem(matrix, vector);
  const expectedMatches = solved?.[start] ?? Number.POSITIVE_INFINITY;

  return {
    expectedMatches,
    promotionProbability: 1,
  };
};

const solveLinearSystem = (matrix: number[][], vector: number[]): number[] | null => {
  const n = matrix.length;
  const a = matrix.map((row) => [...row]);
  const b = [...vector];

  for (let col = 0; col < n; col += 1) {
    let pivot = col;
    for (let row = col + 1; row < n; row += 1) {
      const rowVal = a[row]?.[col] ?? 0;
      const pivotVal = a[pivot]?.[col] ?? 0;
      if (Math.abs(rowVal) > Math.abs(pivotVal)) {
        pivot = row;
      }
    }

    const pivotCandidate = a[pivot]?.[col] ?? 0;
    if (Math.abs(pivotCandidate) < EPS) {
      return null;
    }

    const colRow = a[col];
    const pivotRow = a[pivot];
    if (!colRow || !pivotRow) {
      return null;
    }
    a[col] = pivotRow;
    a[pivot] = colRow;

    const colValue = b[col];
    const pivotVectorValue = b[pivot];
    if (colValue === undefined || pivotVectorValue === undefined) {
      return null;
    }
    b[col] = pivotVectorValue;
    b[pivot] = colValue;

    const pivotMatrixValue = a[col]?.[col];
    if (pivotMatrixValue === undefined || Math.abs(pivotMatrixValue) < EPS) {
      return null;
    }

    for (let j = col; j < n; j += 1) {
      const rowRef = a[col];
      const cell = rowRef?.[j];
      if (!rowRef || cell === undefined) {
        return null;
      }
      rowRef[j] = cell / pivotMatrixValue;
    }

    const bColNormalized = b[col];
    if (bColNormalized === undefined) {
      return null;
    }
    b[col] = bColNormalized / pivotMatrixValue;

    for (let row = 0; row < n; row += 1) {
      if (row === col) continue;

      const factor = a[row]?.[col];
      if (factor === undefined) {
        return null;
      }
      if (Math.abs(factor) < EPS) continue;

      for (let j = col; j < n; j += 1) {
        const targetRow = a[row];
        const sourceRow = a[col];
        const targetCell = targetRow?.[j];
        const sourceCell = sourceRow?.[j];
        if (!targetRow || !sourceRow || targetCell === undefined || sourceCell === undefined) {
          return null;
        }
        targetRow[j] = targetCell - factor * sourceCell;
      }

      const bRow = b[row];
      const bCol = b[col];
      if (bRow === undefined || bCol === undefined) {
        return null;
      }
      b[row] = bRow - factor * bCol;
    }
  }

  return b;
};

const computeExpectedToCurrentTierI = (
  currentTier: SubtierValue,
  currentStats: SegmentStats,
  baseStats: SegmentStats,
  vFloorBaseStats: SegmentStats
): number => {
  if (currentTier === 4) {
    return 0;
  }

  if (baseStats.promotionProbability <= EPS) {
    return Number.POSITIVE_INFINITY;
  }

  const size = 4; // V, IV, III, II (I is target)
  const matrix: number[][] = Array.from({ length: size }, () => Array(size).fill(0));
  const vector: number[] = Array(size).fill(0);

  for (let tier = 0; tier < size; tier += 1) {
    const row = matrix[tier];
    if (!row) {
      return Number.POSITIVE_INFINITY;
    }

    const stats = tier === currentTier ? currentStats : tier === 0 ? vFloorBaseStats : baseStats;
    const up = stats.promotionProbability;
    const down = 1 - up;

    vector[tier] = stats.expectedMatches;

    if (tier === 0) {
      row[tier] = up;
      row[tier + 1] = -up;
      continue;
    }

    row[tier] = 1;
    row[tier - 1] = -down;
    if (tier + 1 < size) {
      row[tier + 1] = -up;
    }
  }

  const solved = solveLinearSystem(matrix, vector);
  if (!solved) {
    return Number.POSITIVE_INFINITY;
  }

  return solved[currentTier] ?? Number.POSITIVE_INFINITY;
};

const computeExpectedToNextBigTierV = (
  currentTier: SubtierValue,
  currentStats: SegmentStats,
  baseStats: SegmentStats,
  vFloorBaseStats: SegmentStats
): number => {
  if (baseStats.promotionProbability <= EPS) {
    return Number.POSITIVE_INFINITY;
  }

  const size = 5; // V, IV, III, II, I
  const matrix: number[][] = Array.from({ length: size }, () => Array(size).fill(0));
  const vector: number[] = Array(size).fill(0);

  for (let tier = 0; tier < size; tier += 1) {
    const row = matrix[tier];
    if (!row) {
      return Number.POSITIVE_INFINITY;
    }

    const stats = tier === currentTier ? currentStats : tier === 0 ? vFloorBaseStats : baseStats;
    const up = stats.promotionProbability;
    const down = 1 - up;

    vector[tier] = stats.expectedMatches;

    if (tier === 0) {
      row[tier] = up;
      row[tier + 1] = -up;
      continue;
    }

    row[tier] = 1;
    row[tier - 1] = -down;
    if (tier < 4) {
      row[tier + 1] = -up;
    }
  }

  const solved = solveLinearSystem(matrix, vector);
  if (!solved) {
    return Number.POSITIVE_INFINITY;
  }

  return solved[currentTier] ?? Number.POSITIVE_INFINITY;
};

const calculateRankProgressStatsJs = (
  pValue: number,
  kValue: RankK,
  currentNetWinsValue: number,
  currentTier: SubtierValue
): CalculationResult => {
  const currentStats =
    currentTier === 0
      ? calculateVFloorSegmentStatsJs(pValue, kValue, currentNetWinsValue)
      : calculateSegmentStatsJs(pValue, kValue, currentNetWinsValue);
  const baseStats = calculateSegmentStatsJs(pValue, kValue, 0);
  const vFloorBaseStats = calculateVFloorSegmentStatsJs(pValue, kValue, 0);

  return {
    leaveCurrentSegmentExpected: currentStats.expectedMatches,
    currentSegmentPromotionProbability: currentStats.promotionProbability,
    expectedToCurrentTierI: computeExpectedToCurrentTierI(
      currentTier,
      currentStats,
      baseStats,
      vFloorBaseStats
    ),
    expectedToNextBigTierV: computeExpectedToNextBigTierV(
      currentTier,
      currentStats,
      baseStats,
      vFloorBaseStats
    ),
  };
};

const formatMatches = (value: number): string => {
  if (!Number.isFinite(value)) {
    return "∞";
  }
  return value.toFixed(2);
};

const formatPercent = (value: number): string => {
  if (!Number.isFinite(value)) {
    return "∞";
  }
  return `${(value * 100).toFixed(2)}%`;
};

const handleCalculate = async (): Promise<void> => {
  if (!formRef.value) return;

  const validation = await formRef.value.validate();
  if (!validation.valid) {
    showSnackbar("请先修正输入参数", "error", "mdi-alert-circle-outline");
    return;
  }

  isLoading.value = true;
  try {
    const p = clamp(winRate.value / 100, 0, 1);
    const k = rankType.value;
    const currentNet = currentNetWins.value;
    const currentTier = currentSubtier.value;

    if (wasmReady.value) {
      try {
        const wasmResult = calculate_rank_progress_stats(
          p,
          k,
          currentNet,
          currentTier
        ) as WasmRankProgressStats;

        solverMode.value = "WASM";
        results.value = {
          leaveCurrentSegmentExpected: Math.max(wasmResult.leave_current_segment_expected, 0),
          currentSegmentPromotionProbability: clamp(
            wasmResult.current_segment_promotion_probability,
            0,
            1
          ),
          expectedToCurrentTierI: wasmResult.expected_to_current_tier_i,
          expectedToNextBigTierV: wasmResult.expected_to_next_big_tier_v,
        };
      } catch (error) {
        console.warn("WASM full pipeline failed, fallback to JS full pipeline", error);
        solverMode.value = "JS";
        results.value = calculateRankProgressStatsJs(p, k, currentNet, currentTier);
      }
    } else {
      solverMode.value = "JS";
      results.value = calculateRankProgressStatsJs(p, k, currentNet, currentTier);
    }

    showSnackbar("计算完成", "success", "mdi-check-circle-outline", 2200);
  } catch (error) {
    console.error("Calculation failed", error);
    showSnackbar("计算失败，请稍后重试", "error", "mdi-close-octagon-outline", 3600);
  } finally {
    isLoading.value = false;
  }
};
</script>

<style scoped>
.calculator-page {
  max-width: 1240px;
}

.hero {
  position: relative;
  overflow: hidden;
  padding: 1.6rem 1.2rem;
  border-radius: 1.2rem;
  background: linear-gradient(120deg, rgba(8, 65, 92, 0.95), rgba(7, 126, 105, 0.9));
  color: #f7fffe;
}

.hero::after {
  content: "";
  position: absolute;
  right: -90px;
  top: -120px;
  width: 260px;
  height: 260px;
  border-radius: 50%;
  background: radial-gradient(circle at 30% 30%, rgba(255, 255, 255, 0.35), rgba(255, 255, 255, 0.08));
}

.hero-kicker {
  margin: 0;
  font-size: 0.86rem;
  text-transform: uppercase;
  letter-spacing: 0.12em;
  opacity: 0.9;
}

.hero-title {
  margin: 0.4rem 0 0.3rem;
  font-size: clamp(1.7rem, 3vw, 2.4rem);
  line-height: 1.2;
  font-family: "Noto Serif SC", "Source Han Serif SC", serif;
}

.intro-alert-text {
  margin: 0;
  font-size: 0.88rem;
  line-height: 1.5;
}

.intro-alert a {
  color: #0b3f75;
  text-decoration: underline;
  text-decoration-thickness: 2px;
  text-underline-offset: 3px;
  transition: color 0.2s ease;
  font-weight: 600;
}

.intro-alert a:hover {
  color: #7a2f00;
}

.panel {
  padding: 1.2rem;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid rgba(255, 255, 255, 0.72);
  backdrop-filter: blur(8px);
}

.panel-input {
  height: 100%;
}

.panel-header {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  margin-bottom: 1rem;
}

.panel-header h2 {
  margin: 0;
  font-size: 1.05rem;
  font-weight: 700;
}

.winrate-box {
  padding: 1rem 1rem 0.95rem;
  background: linear-gradient(148deg, rgba(255, 255, 255, 0.92), rgba(236, 251, 247, 0.96));
  border: 1px solid rgba(106, 181, 209, 0.3);
  box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.65);
}

.winrate-head {
  padding: 0.1rem 0.2rem 0.35rem;
}

.winrate-label {
  font-size: 0.96rem;
  font-weight: 600;
  color: #24566f;
  letter-spacing: 0.01em;
}

.winrate-value {
  font-size: 1.02rem;
  font-weight: 700;
  color: #1f5f80;
}

.winrate-slider {
  padding-inline: 0.28rem;
}

.winrate-input {
  padding-inline: 0.2rem;
}

.metric-grid {
  margin-top: 0.1rem;
}

.panel-row {
  row-gap: 1rem;
}

.metric {
  height: 100%;
  padding: 0.95rem;
}

.metric-title {
  margin: 0;
  font-size: 0.84rem;
  color: #385468;
}

.metric-value {
  margin: 0.55rem 0 0;
  font-size: 1.52rem;
  font-weight: 700;
  line-height: 1.2;
  color: #052c3d;
}

.metric-a {
  background: linear-gradient(145deg, rgba(82, 176, 255, 0.18), rgba(62, 129, 221, 0.2));
}

.metric-b {
  background: linear-gradient(145deg, rgba(69, 194, 141, 0.2), rgba(50, 160, 122, 0.22));
}

.metric-c {
  background: linear-gradient(145deg, rgba(255, 186, 77, 0.2), rgba(246, 149, 66, 0.22));
}

.metric-d {
  background: linear-gradient(145deg, rgba(255, 145, 128, 0.22), rgba(244, 108, 108, 0.22));
}

.empty-state {
  min-height: 250px;
  display: grid;
  place-items: center;
  text-align: center;
  gap: 0.65rem;
  color: #526778;
}

.meta {
  display: flex;
  flex-wrap: wrap;
}

@media (max-width: 840px) {
  .panel {
    padding: 1rem;
  }

  .hero {
    padding: 1.2rem 1rem;
  }

  .metric-value {
    font-size: 1.38rem;
  }
}

@media (min-width: 840px) {
  .panel-row {
    row-gap: 1.5rem;
  }
}

</style>

<style>
.rank-select-menu .v-list-item--active {
  background: rgba(66, 165, 245, 0.22) !important;
}

.rank-select-menu .v-list-item {
  color: #0d3550 !important;
}

.rank-select-menu .v-list-item--active .v-list-item-title {
  color: #0d3550 !important;
  font-weight: 700;
}

.rank-select-menu .v-list-item:hover .v-list-item__overlay,
.rank-select-menu .v-list-item:focus-visible .v-list-item__overlay,
.rank-select-menu .v-list-item--active .v-list-item__overlay,
.rank-select-menu .v-list-item--active:hover .v-list-item__overlay {
  background: #42a5f5 !important;
  opacity: 0.14 !important;
}

.rank-select-menu .v-ripple__animation {
  color: #42a5f5 !important;
  opacity: 0.22 !important;
}
</style>
