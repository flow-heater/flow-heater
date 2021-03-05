<template>
  <v-row justify="center" align="center">
    <v-col cols="12" sm="8" md="6">
      <v-card>
        <v-card-title class="headline">
          Welcome to the Vuetify + Nuxt.js template
        </v-card-title>
        <v-card-text v-if="$auth.loggedIn">
          Hello {{ $auth.loggedIn ? $auth.user.nickname : "Freund" }}! Do you
          want to <a @click="$auth.logout()" class="link">sign out</a>?
          <FhApp />
        </v-card-text>
        <v-card-text v-else>
          You're not authenticated yet. Maybe you want to
          <a @click="$auth.login()" class="link">sign in</a>?
        </v-card-text>
      </v-card>
    </v-col>
  </v-row>
</template>

<script>
import Logo from "~/components/Logo.vue";
import VuetifyLogo from "~/components/VuetifyLogo.vue";
import FhApp from "~/components/FhApp.vue";

export default {
  components: {
    Logo,
    VuetifyLogo,
    FhApp,
  },
};
</script>
