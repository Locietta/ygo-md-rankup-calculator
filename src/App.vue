<script setup lang="ts">
import Calculator from "./components/Calculator.vue";
</script>

<template>
  <v-app>
    <v-main class="app-main">
      <Calculator />
      <div class="art-credit">
        BG art from
        <a
          href="https://x.com/marronpan160"
          target="_blank"
          rel="noopener noreferrer"
          >@marronpan160</a
        >
      </div>
    </v-main>
  </v-app>
</template>

<style scoped>
.app-main {
  min-height: 100vh;
  background: url("/bg-dmg.webp") center 25% / cover no-repeat fixed;
}

.app-main::before {
  content: "";
  position: fixed;
  inset: 0;
  background: rgba(240, 244, 248, 0.6);
  pointer-events: none;
  z-index: 0;
}
.art-credit {
  position: fixed;
  bottom: 8px;
  right: 12px;
  font-size: 0.75rem;
  color: rgba(0, 0, 0, 0.45);
  z-index: 1;
}

.art-credit a {
  color: inherit;
  text-decoration: underline;
}

.art-credit a:hover {
  color: rgba(0, 0, 0, 0.7);
}
</style>
