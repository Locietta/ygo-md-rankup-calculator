<script setup lang="ts">
import Calculator from "./components/Calculator.vue";
</script>

<template>
  <v-app>
    <v-main class="app-main">
      <Calculator />
    </v-main>
  </v-app>
</template>

<style scoped>
.app-main {
  min-height: 100vh;
}
</style>
