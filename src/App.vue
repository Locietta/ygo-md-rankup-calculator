<script setup lang="ts">
import Calculator from "./components/Calculator.vue";
import { createVuetify } from "vuetify";

createVuetify();
</script>

<template>
  <v-app>
    <v-main>
      <v-container>
        <v-row justify="center">
          <v-card class="pa-4">
            <Calculator />
          </v-card>
        </v-row>
      </v-container>
    </v-main>
  </v-app>
</template>
