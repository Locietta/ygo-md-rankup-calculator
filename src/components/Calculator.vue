<template>
  <v-responsive class="text-center">
    <h1 class="text-h4 font-weight-bold my-5 text-primary">游戏王MD段位计算器</h1>
  </v-responsive>

  <v-container>
    <v-card class="pa-md-6 pa-4" elevation="4" rounded="xl">
      <v-card-title class="text-h5 font-weight-bold text-center mb-2">
        <v-icon start color="primary" size="large">mdi-sword-cross</v-icon>
        MD 段位升降计算器
      </v-card-title>
      <v-card-subtitle class="text-center mb-6 text-medium-emphasis">
        输入您的数据以预估段位变动情况
      </v-card-subtitle>

      <v-form ref="formRef" @submit.prevent="handleCalculate">
        <v-text-field
          v-model.number="winRate"
          label="当前胜率 (%)"
          type="number"
          placeholder="例如: 55"
          variant="outlined"
          density="comfortable"
          :rules="winRateRules"
          class="mb-4"
          prepend-inner-icon="mdi-percent-outline"
          clearable
          color="primary"
        />

        <v-select
          v-model="targetK"
          :items="kOptions"
          item-title="label"
          item-value="value"
          label="目标段位类型"
          variant="outlined"
          density="comfortable"
          class="mb-4"
          prepend-inner-icon="mdi-trophy-variant-outline"
          color="primary"
        />

        <v-text-field
          v-model.number="currentNetWins"
          label="当前净胜局"
          type="number"
          placeholder="例如: 0, 1, -1"
          variant="outlined"
          density="comfortable"
          :rules="currentNetWinsRules"
          class="mb-2"
          prepend-inner-icon="mdi-chart-line"
          clearable
          color="primary"
        />

        <v-alert
          density="compact"
          variant="tonal"
          type="info"
          icon="mdi-information-outline"
          class="mb-5 mt-3 pa-3"
          border="start"
          border-color="info"
          rounded="lg"
        >
          <div class="text-caption" style="line-height: 1.65">
            <strong class="d-block mb-1 text-info-darken-1">提示：</strong>
            <ul class="pl-4" style="list-style-type: disc">
              <li>净胜0局时，连败3局将导致掉段。</li>
              <li>"-1" 表示在0净胜局基础上连败1局；"-2" 表示连败2局。</li>
              <li>掉段保护中（如-1、-2状态），赢1局会直接变为净胜1局。</li>
            </ul>
          </div>
        </v-alert>

        <v-btn
          :loading="isLoading"
          type="submit"
          color="primary"
          block
          size="large"
          class="mt-4 py-3"
          elevation="2"
          rounded="lg"
        >
          <v-icon start size="default">mdi-calculator-variant-outline</v-icon>
          开始计算
        </v-btn>
      </v-form>

      <template v-if="resultsVisible">
        <v-divider class="my-6"></v-divider>

        <div class="text-h6 mb-4 text-center font-weight-medium">
          <v-icon start color="secondary">mdi-poll</v-icon>
          计算结果
        </div>

        <v-alert
          type="info"
          variant="tonal"
          class="mb-4 pa-4"
          icon="mdi-clock-fast"
          border="start"
          border-color="info"
          density="comfortable"
          closable
          rounded="lg"
          elevation="1"
        >
          <div class="text-subtitle-1 font-weight-medium">下次段位变动期望场次</div>
          大约需要
          <strong class="text-info-darken-2">{{
            expectedMatches !== null ? expectedMatches.toFixed(2) : "N/A"
          }}</strong>
          场
        </v-alert>

        <v-alert
          type="success"
          variant="tonal"
          class="pa-4"
          icon="mdi-arrow-up-bold-hexagon-outline"
          border="start"
          border-color="success"
          density="comfortable"
          closable
          rounded="lg"
          elevation="1"
        >
          <div class="text-subtitle-1 font-weight-medium">升段概率</div>
          您本次升段的概率为
          <strong class="text-success-darken-2">{{
            promotionProbability !== null ? (promotionProbability * 100).toFixed(2) + "%" : "N/A"
          }}</strong>
        </v-alert>
      </template>
    </v-card>

    <v-snackbar
      v-model="snackbar.visible"
      :timeout="snackbar.timeout"
      :color="snackbar.color"
      location="top right"
      multi-line
      rounded="lg"
      elevation="6"
    >
      <v-icon start class="mr-2">{{ snackbar.icon }}</v-icon>
      {{ snackbar.text }}
      <template v-slot:actions>
        <v-btn
          variant="text"
          @click="snackbar.visible = false"
          :color="snackbar.color ? 'white' : undefined"
        >
          关闭
        </v-btn>
      </template>
    </v-snackbar>
  </v-container>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import init, { calculate_rank_stats_formulas } from "wasm";

// WASM initialization
let wasmInitialized = false;

onMounted(async () => {
  try {
    await init();
    wasmInitialized = true;
    console.log("WASM module loaded successfully");
  } catch (error) {
    console.error("Failed to load WASM module:", error);
    showSnackbar("WASM模块加载失败，将使用JavaScript版本", "warning", "mdi-alert", 3000);
  }
});

// --- Reactive State for Snackbar ---
interface SnackbarState {
  visible: boolean;
  text: string;
  timeout: number;
  color: string;
  icon: string;
}
const snackbar = ref<SnackbarState>({
  visible: false,
  text: "",
  timeout: 4000,
  color: "info",
  icon: "mdi-information",
});

const showSnackbar = (
  text: string,
  color: string = "info",
  icon: string = "mdi-information",
  timeout: number = 4000
) => {
  snackbar.value.text = text;
  snackbar.value.color = color;
  snackbar.value.icon = icon;
  snackbar.value.timeout = timeout;
  snackbar.value.visible = true;
};

// --- Form and Input State ---
const formRef = ref<any>(null); // For VForm validation: ref="formRef"
const winRate = ref<number | null>(55);
const targetK = ref<number>(4);
const currentNetWins = ref<number | null>(0);

const isLoading = ref(false);
const resultsVisible = ref(false);
const expectedMatches = ref<number | null>(null);
const promotionProbability = ref<number | null>(null);

// --- Options & Computed ---
const kOptions = [
  { label: "白金/钻石 (净胜4场)", value: 4 },
  { label: "大师 (净胜5场)", value: 5 },
];

const maxNetWins = computed(() => targetK.value - 1);

// --- Validation Rules for Vuetify ---
const winRateRules = [
  (v: any) => (v !== null && v !== "") || "胜率是必填项",
  (v: any) => (Number(v) >= 0 && Number(v) <= 100) || "胜率必须在 0 到 100 之间",
];

const currentNetWinsRules = computed(() => [
  (v: any) => (v !== null && v !== "") || "当前净胜局是必填项",
  (v: any) =>
    (Number(v) >= -2 && Number(v) <= maxNetWins.value) ||
    `净胜局需在 -2 和 ${maxNetWins.value} 之间 (K=${targetK.value})`,
  (v: any) => Number(v) < targetK.value || `净胜局 (${v}) 已达或超过目标 (${targetK.value})`,
]);

// --- Methods ---

const calculate_rank_stats_formulas_js = (pValue: number, kValue: number, netWinsValue: number) => {
  // Early validation and edge cases first
  if (kValue !== 4 && kValue !== 5) {
    return {
      expected_matches: Infinity,
      promotion_probability: NaN,
    };
  }

  // Handle edge cases for p=0 and p=1 first (most intuitive and avoids polynomial calculations)
  // also treat any negative p as 0, and any p > 1 as 1
  if (pValue <= 0.0) {
    const expected = netWinsValue >= -2 ? 3.0 + netWinsValue : 0.0; // 3 losses from 0, + netWins to reach 0
    return {
      expected_matches: expected,
      promotion_probability: 0.0,
    };
  }

  if (pValue >= 1.0) {
    let expected: number;
    if (netWinsValue < kValue) {
      if (netWinsValue < 0) {
        // 1 win to get to +1 state, then (K-1) more wins
        expected = 1.0 + (kValue - 1);
      } else {
        expected = kValue - netWinsValue;
      }
    } else {
      expected = 0.0; // Already at or above K
    }
    return {
      expected_matches: expected,
      promotion_probability: 1.0,
    };
  }

  const createMartingaleFn = (p: number) => {
    return (n: number): number => {
      if (n >= 0) {
        if (p === 0.5) {
          return n;
        } else {
          const factor = p / (2.0 * p - 1.0);
          const ratio = (1.0 - p) / p;
          return factor * (1.0 - Math.pow(ratio, n));
        }
      } else {
        return 1.0 - Math.pow(1.0 - p, n);
      }
    };
  };

  const martingale = createMartingaleFn(pValue);

  const promotionProbability =
    (martingale(netWinsValue) - martingale(-3)) / (martingale(kValue) - martingale(-3));

  // Pre-calculate powers of p only once
  const p2 = pValue * pValue;
  const p3 = p2 * pValue;
  const p4 = p3 * pValue;
  const p5 = p4 * pValue;
  const p6 = p5 * pValue;

  const calculateK4Stats = (
    p: number,
    p2: number,
    p3: number,
    p4: number,
    p5: number,
    p6: number,
    currentNetWinsVal: number
  ): number => {
    const denom = p6 - 5.0 * p5 + 11.0 * p4 - 13.0 * p3 + 11.0 * p2 - 5.0 * p + 1.0;

    switch (currentNetWinsVal) {
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
        return Infinity;
    }
  };

  const calculateK5Stats = (
    p: number,
    p2: number,
    p3: number,
    p4: number,
    p5: number,
    p6: number,
    currentNetWinsVal: number
  ): number => {
    const denom = 2.0 * p6 - 10.0 * p5 + 22.0 * p4 - 24.0 * p3 + 16.0 * p2 - 6.0 * p + 1.0;

    switch (currentNetWinsVal) {
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
        return Infinity;
    }
  };

  const expectedMatches =
    kValue === 4
      ? calculateK4Stats(pValue, p2, p3, p4, p5, p6, netWinsValue)
      : calculateK5Stats(pValue, p2, p3, p4, p5, p6, netWinsValue);

  return {
    expected_matches: Math.max(expectedMatches, 0.0),
    promotion_probability: Math.max(0.0, Math.min(1.0, promotionProbability)),
  };
};

const handleCalculate = async () => {
  if (!formRef.value) return;
  const { valid } = await formRef.value.validate();
  if (!valid) {
    showSnackbar("请检查并修正表单中的错误！", "error", "mdi-alert-circle-outline");
    return;
  }

  isLoading.value = true;
  resultsVisible.value = false;

  try {
    // Ensure winRate.value and currentNetWins.value are not null before passing
    const pValue = (winRate.value ?? 0) / 100;
    const kValue = targetK.value;
    const netWinsValue = currentNetWins.value ?? 0;

    let result: { expected_matches: number; promotion_probability: number };

    if (wasmInitialized) {
      try {
        const wasmResult = calculate_rank_stats_formulas(pValue, kValue, netWinsValue);
        result = {
          expected_matches: wasmResult.expected_matches,
          promotion_probability: wasmResult.promotion_probability,
        };
        console.log("Used WASM calculation");
      } catch (wasmError) {
        console.warn("WASM calculation failed, falling back to JavaScript:", wasmError);
        result = calculate_rank_stats_formulas_js(pValue, kValue, netWinsValue);
        console.log("Used JavaScript fallback calculation");
      }
    } else {
      result = calculate_rank_stats_formulas_js(pValue, kValue, netWinsValue);
      console.log("Used JavaScript calculation (WASM not initialized)");
    }

    expectedMatches.value = result.expected_matches;
    promotionProbability.value = result.promotion_probability;
    resultsVisible.value = true;

    showSnackbar("计算完成！结果已更新", "success", "mdi-check-circle-outline");
  } catch (error: any) {
    console.error("Calculation error:", error);
    showSnackbar(
      `计算失败: ${error.message || "未知错误，请稍后重试。"}`,
      "error",
      "mdi-close-octagon-outline",
      5000
    );
    expectedMatches.value = null;
    promotionProbability.value = null;
  } finally {
    isLoading.value = false;
  }
};
</script>

<style scoped>
/* Page title specific style if needed, but Vuetify classes handle a lot */
.text-primary {
  /* Vuetify should provide this if primary color is set */
  color: rgb(var(--v-theme-primary));
}
.text-info-darken-1 {
  color: rgb(
    var(--v-theme-info-darken-1)
  ); /* You might need to define info-darken-1 in your theme */
}
.text-info-darken-2 {
  color: rgb(var(--v-theme-info-darken-2));
}
.text-success-darken-2 {
  color: rgb(var(--v-theme-success-darken-2));
}

/* For better list presentation inside alert */
.v-alert ul {
  margin-bottom: 0; /* Remove default bottom margin from ul */
}
.v-alert li {
  /* align to left */
  text-align: left;

  margin-bottom: 4px; /* Add a little space between list items */
}
.v-alert li:last-child {
  margin-bottom: 0;
}
</style>
