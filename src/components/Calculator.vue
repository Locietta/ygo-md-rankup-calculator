<template>
  <v-container class="calculator-page py-6 py-md-10">
    <section class="hero mb-6 mb-md-8">
      <p class="hero-kicker">Yu-Gi-Oh! Master Duel</p>
      <h1 class="hero-title">段位升降与大段期望计算器</h1>
      <p class="hero-subtitle">
        单小段结果采用闭式公式，大段结果基于递推/DP。输入当前小段、位置和胜率后，一次给出四项指标。
      </p>
    </section>

    <v-row class="ga-4 ga-md-6" align="stretch">
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
              label="当前小段"
              variant="outlined"
              density="comfortable"
              class="mb-4"
              prepend-inner-icon="mdi-ladder"
            />

            <v-text-field
              v-model.number="currentNetWins"
              label="当前小段位置（净胜局）"
              type="number"
              :rules="currentNetWinsRules"
              variant="outlined"
              density="comfortable"
              class="mb-3"
              prepend-inner-icon="mdi-chart-line-variant"
              hint="范围为 -2 到 K-1，K 由大段类型决定"
              persistent-hint
            />

            <v-card variant="tonal" class="pa-4 mb-2 winrate-box" rounded="lg">
              <div class="d-flex justify-space-between align-center mb-2">
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
              />
              <v-text-field
                v-model.number="winRate"
                type="number"
                suffix="%"
                :rules="winRateRules"
                variant="outlined"
                density="compact"
                hide-details="auto"
                class="mt-3"
              />
            </v-card>

            <v-alert
              type="info"
              variant="tonal"
              density="comfortable"
              rounded="lg"
              icon="mdi-information-outline"
              class="my-5"
            >
              DP 模型将每个小段视为一个递推状态：升段进入下一小段，降段回到上一小段（V 视作地板）。
            </v-alert>

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
            <v-row class="metric-grid" dense>
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
                  <p class="metric-value">{{ formatMatches(results.expectedToNextBigTierV) }}</p>
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
      elevation="6"
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
import { computed, onMounted, ref } from "vue";
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

const rankTypeOptions: Array<{ label: string; value: RankK }> = [
  { label: "白金/钻石系（单小段净胜 4 场升段）", value: 4 },
  { label: "大师系（单小段净胜 5 场升段）", value: 5 },
];

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
const solverMode = ref<"WASM整链路" | "JS整链路">("WASM整链路");
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
    solverMode.value = "WASM整链路";
  } catch (error) {
    console.error("WASM init failed, fallback to JS formulas", error);
    wasmReady.value = false;
    solverMode.value = "JS整链路";
    showSnackbar("WASM 加载失败，已自动切换 JS 整链路", "warning", "mdi-alert");
  }
});

const maxNetWins = computed(() => rankType.value - 1);

const currentSubtierLabel = computed(
  () => subtierOptions.find((option) => option.value === currentSubtier.value)?.label ?? "-"
);

const winRateRules = [
  (value: unknown) => value !== null || "胜率不能为空",
  (value: unknown) => Number(value) >= 0 || "胜率不能低于 0",
  (value: unknown) => Number(value) <= 100 || "胜率不能高于 100",
];

const currentNetWinsRules = computed(() => [
  (value: unknown) => value !== null || "净胜局不能为空",
  (value: unknown) => Number.isInteger(Number(value)) || "净胜局必须是整数",
  (value: unknown) => Number(value) >= -2 || "净胜局不能小于 -2",
  (value: unknown) => Number(value) <= maxNetWins.value || `净胜局不能大于 ${maxNetWins.value}`,
]);

const clamp = (value: number, min: number, max: number): number => {
  if (value < min) return min;
  if (value > max) return max;
  return value;
};

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

const calculateK4ExpectedMatches = (
  p: number,
  p2: number,
  p3: number,
  p4: number,
  p5: number,
  p6: number,
  currentNetWinsValue: number
): number => {
  const denom = p6 - 5.0 * p5 + 11.0 * p4 - 13.0 * p3 + 11.0 * p2 - 5.0 * p + 1.0;

  switch (currentNetWinsValue) {
    case -2:
      return (2.0 * p5 - 4.0 * p4 - p3 + 10.0 * p2 - 6.0 * p + 3.0) / denom;
    case -1:
      return (-p5 + 5.0 * p4 - 11.0 * p3 + 15.0 * p2 - 9.0 * p + 4.0) / denom;
    case 0:
      return (p5 - 2.0 * p4 - 3.0 * p3 + 13.0 * p2 - 12.0 * p + 5.0) / denom;
    case 1:
      return (-2.0 * p4 + 2.0 * p3 + 5.0 * p2 - 3.0 * p + 2.0) / denom;
    case 2:
      return (2.0 * p4 + 2.0 * p3 + p2 - p + 1.0) / denom;
    case 3:
      return (-2.0 * p5 + 12.0 * p4 - 29.0 * p3 + 36.0 * p2 - 22.0 * p + 6.0) / denom;
    default:
      return Number.POSITIVE_INFINITY;
  }
};

const calculateK5ExpectedMatches = (
  p: number,
  p2: number,
  p3: number,
  p4: number,
  p5: number,
  p6: number,
  currentNetWinsValue: number
): number => {
  const denom = 2.0 * p6 - 10.0 * p5 + 22.0 * p4 - 24.0 * p3 + 16.0 * p2 - 6.0 * p + 1.0;

  switch (currentNetWinsValue) {
    case -2:
      return (3.0 * p4 + 3.0 * p3 - 2.0 * p + 1.0) / denom;
    case -1:
      return (3.0 * p6 - 6.0 * p4 + 3.0 * p3 - 8.0 * p2 + 5.0 * p - 2.0) / denom;
    case 0:
      return (3.0 * p6 - 9.0 * p5 + 12.0 * p4 - 11.0 * p3 + 16.0 * p2 - 9.0 * p + 3.0) / denom;
    case 1:
      return (-2.0 * p5 + 10.0 * p4 - 19.0 * p3 + 24.0 * p2 - 13.0 * p + 4.0) / denom;
    case 2:
      return (3.0 * p6 - 14.0 * p5 + 29.0 * p4 - 35.0 * p3 + 32.0 * p2 - 17.0 * p + 5.0) / denom;
    case 3:
      return (-p5 + 8.0 * p4 - 22.0 * p3 + 32.0 * p2 - 21.0 * p + 6.0) / denom;
    case 4:
      return (3.0 * p6 - 19.0 * p5 + 52.0 * p4 - 78.0 * p3 + 69.0 * p2 - 33.0 * p + 7.0) / denom;
    default:
      return Number.POSITIVE_INFINITY;
  }
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

  const p2 = pValue * pValue;
  const p3 = p2 * pValue;
  const p4 = p3 * pValue;
  const p5 = p4 * pValue;
  const p6 = p5 * pValue;

  const expectedMatches =
    kValue === 4
      ? calculateK4ExpectedMatches(pValue, p2, p3, p4, p5, p6, netWinsValue)
      : calculateK5ExpectedMatches(pValue, p2, p3, p4, p5, p6, netWinsValue);

  return {
    expectedMatches: Math.max(expectedMatches, 0),
    promotionProbability: clamp(promotionProbability, 0, 1),
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
  baseStats: SegmentStats
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

    const stats = tier === currentTier ? currentStats : baseStats;
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
  baseStats: SegmentStats
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

    const stats = tier === currentTier ? currentStats : baseStats;
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
  const currentStats = calculateSegmentStatsJs(pValue, kValue, currentNetWinsValue);
  const baseStats = calculateSegmentStatsJs(pValue, kValue, 0);

  return {
    leaveCurrentSegmentExpected: currentStats.expectedMatches,
    currentSegmentPromotionProbability: currentStats.promotionProbability,
    expectedToCurrentTierI: computeExpectedToCurrentTierI(currentTier, currentStats, baseStats),
    expectedToNextBigTierV: computeExpectedToNextBigTierV(currentTier, currentStats, baseStats),
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

        solverMode.value = "WASM整链路";
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
        solverMode.value = "JS整链路";
        results.value = calculateRankProgressStatsJs(p, k, currentNet, currentTier);
      }
    } else {
      solverMode.value = "JS整链路";
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

.hero-subtitle {
  margin: 0;
  max-width: 780px;
  font-size: 0.98rem;
  line-height: 1.6;
  opacity: 0.95;
}

.panel {
  height: 100%;
  padding: 1.2rem;
  background: rgba(255, 255, 255, 0.9);
  border: 1px solid rgba(255, 255, 255, 0.72);
  backdrop-filter: blur(8px);
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
  background: linear-gradient(150deg, rgba(38, 114, 231, 0.11), rgba(46, 181, 139, 0.13));
}

.winrate-label {
  font-size: 0.93rem;
  color: #17324a;
}

.winrate-value {
  font-size: 1.08rem;
  color: #0d5b73;
}

.metric-grid {
  margin-top: 0.1rem;
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

@media (max-width: 960px) {
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
</style>
