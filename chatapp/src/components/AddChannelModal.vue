<template>
  <div class="fixed inset-0 flex items-center justify-center bg-black bg-opacity-50 z-50">
    <div class="bg-white p-4 rounded shadow-lg w-96">
      <h2 class="text-lg font-bold mb-4">Add Channel</h2>
      <div class="mb-4">
        <label class="block text-sm font-medium text-gray-700">Channel Name</label>
        <input v-model="channelName" type="text"
               class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"/>
      </div>
      <div class="mb-4">
        <label class="block text-sm font-medium text-gray-700">Select Users</label>
        <select v-model="selectedUserIds" multiple
                class="mt-1 block w-full border border-gray-300 rounded-md shadow-sm focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm">
          <option v-for="user in filteredUsers" :key="user.id" :value="user.id">{{ user.fullname }}</option>
        </select>
      </div>
      <div class="mb-4">
        <label class="block text-sm font-medium text-gray-700">Public</label>
        <input type="checkbox" v-model="isPublic" class="mt-1"/>
      </div>
      <div class="flex justify-end">
        <button @click="$emit('close')"
                class="mr-2 bg-gray-300 hover:bg-gray-400 text-gray-800 font-bold py-2 px-4 rounded">Cancel
        </button>
        <button @click="addChannel" class="bg-blue-500 hover:bg-blue-700 text-white font-bold py-2 px-4 rounded">Add
        </button>
      </div>
    </div>
  </div>
</template>

<script>
export default {
  data() {
    return {
      channelName: '',
      selectedUserIds: [],
      isPublic: false,
    };
  },
  computed: {
    users() {
      return Object.values(this.$store.state.users);
    },
    filteredUsers() {
      return this.users.filter(user => user.id !== this.$store.state.user.id);
    },
  },
  methods: {
    async addChannel() {
      if (this.selectedUserIds.length > 0) {
        try {
          const newChannel = await this.$store.dispatch('createChannel', {
            name: this.channelName,
            members: [this.$store.state.user.id, ...this.selectedUserIds],
            isPublic: this.isPublic,
          });
          this.$emit('add-channel', newChannel);
          this.$emit('close'); // Close the modal after adding the channel
        } catch (error) {
          console.error('Failed to create channel:', error);
        }
      }
    },
  },
};
</script>