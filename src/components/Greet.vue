<script setup lang="ts">
import { Ref, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";

const log: Ref<String[]> = ref([]);
const invite_code = ref("fed11qgqpw9thwvaz7te3xgmjuvpwxqhrzw3jxcun2vp0qqqjq5sd54ldl2pzu5qwx5j9cyu38wpre4xzupwr7lzk7tsymquun5pv2rg5ql");
const admin_password = ref("pass");
const peer_id = ref("0");
const note_text = ref("note text goes here");
const current_event = ref("");

enum Invocations {
  JoinFederation = "plugin:roastr|join_federation_as_admin",
  CreateNote = "plugin:roastr|create_note",
  SigningSessions = "plugin:roastr|get_signing_sessions"
}

async function join_federation_as_admin() {
  console.log("Joining a federation...");
  console.log(`${invite_code.value}, ${admin_password.value}, ${peer_id.value}`)
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  log.value.push(await invoke(Invocations.JoinFederation, {
    inviteCode: invite_code.value,
    adminPassword: admin_password.value,
    peerId: peer_id.value,
  }));
  console.log("Federation joined.")
}

async function create_note() {
  console.log("creating note");
  let note_id = await invoke(Invocations.CreateNote, { noteText: note_text.value });
  log.value.push(note_id as String);
  console.log("note created");
}

async function get_signing_sessions() {
  console.log("getting sessions");
  let sessions = await invoke(Invocations.SigningSessions, { eventId: current_event.value })
  log.value.push(JSON.stringify(sessions));
}
</script>

<template>
  <p>latest functions returned: {{ log }}</p>
  <form class="row" @submit.prevent="join_federation_as_admin">
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
  <form class="row" @submit.prevent="create_note">
    <input
      id="note-creation-text"
      v-model="note_text"
      placeholder="Enter note text here"
    />
    <button type="submit">Create Note</button>
  </form>
  <form class="row" @submit.prevent="get_signing_sessions">
    <input id="event-id" v-model="current_event" />
    <button type="submit">Get Signing Sessions</button>
  </form>

</template>
