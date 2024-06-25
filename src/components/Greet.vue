<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const greetMsg = ref("");
const invite_code = ref("fed11qgqpw9thwvaz7te3xgmjuvpwxqhrzw3j8yurjvf0qqqjqvxv6zvkg0lpak7lnr4s0azaqwyhu22kfurpw9sdlnze687s8xfu7649y9");
const admin_password = ref("pass");
const peer_id = ref("0");

async function greet() {
  console.log("greeting");
  console.log(`${invite_code.value}, ${admin_password.value}, ${peer_id.value}`)
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsg.value = await invoke("plugin:roastr|join_federation_as_admin", {
    inviteCode: invite_code.value,
    adminPassword: admin_password.value,
    peerId: peer_id.value,
  });
}
</script>

<template>
  <form class="row" @submit.prevent="greet">
    <input
      id="invite-code-input"
      v-model="invite_code"
      placeholder="Enter an invite code..."
    />
    <input
      id="admin-password-input"
      v-model="admin_password"
      placeholder="Enter admin password..."
    />
    <input
      id="peer-id-input"
      v-model="peer_id"
      placeholder="Enter Peer ID..."
    />
    <button type="submit">Send it</button>
  </form>

  <p>return: {{ greetMsg }}</p>
</template>
