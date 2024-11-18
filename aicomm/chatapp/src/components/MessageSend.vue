<template>
  <div class="flex flex-col bg-gray-100 border-t border-gray-200 relative bottom-0">
    <div class="flex items-center">
      <button class="p-2 mr-2 text-gray-600 hover:text-blue-600 focus:outline-none" @click="triggerFileUpload">
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
          <path
              d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
              stroke-linecap="round" stroke-linejoin="round"
              stroke-width="2"/>
        </svg>
      </button>
      <input
          ref="fileInput"
          accept="image/*"
          class="hidden"
          multiple
          type="file"
          @change="handleFileUpload"
      />
      <!-- Add more image buttons here if needed -->
    </div>

    <div>
      <textarea
          v-model="message"
          class="w-full px-4 text-sm bg-gray-100 border-none rounded-lg focus:outline-none resize-none"
          placeholder="Type a message..."
          rows="3"
          @keyup.enter="sendMessage"
      ></textarea>
    </div>

    <div v-if="files.length > 0" class="flex flex-wrap p-2">
      <img
          v-for="file in files"
          :key="file.path"
          :src="file.fullUrl"
          alt="Uploaded image"
          class="h-64 object-cover rounded mr-2 mb-2"
      />
    </div>

    <button
        class="absolute bottom-4 right-4 p-1 text-white bg-blue-600 rounded-full hover:bg-blue-700 focus:outline-none"
        @click="sendMessage">
      <svg
          class="w-5 h-5"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          xmlns="http://www.w3.org/2000/svg"
      >
        <path
            d="M5 12h14M12 5l7 7-7 7"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
        />
      </svg>
    </button>
  </div>
</template>

<script>
export default {
  data() {
    return {
      message: '',
      files: [],
    };
  },
  computed: {
    userId() {
      return this.$store.state.user.id;
    },
    activeChannelId() {
      const channel = this.$store.state.activeChannel;
      if (!channel) {
        return null;
      }
      return channel.id;
    },
  },
  methods: {
    sendMessage() {
      if (this.message.trim() === '') return;

      const payload = {
        chatId: this.activeChannelId,
        content: this.message,
        files: this.files.map(file => file.path),
      };

      console.log('Sending message:', payload);

      this.$store.dispatch('messageSent', {
        chatId: payload.chatId,
        type: "text",
        size: payload.content.length,
        totalFiles: payload.files.length
      });

      try {
        this.$store.dispatch('sendMessage', payload);
        this.message = ''; // Clear the input after sending
        this.files = []; // Clear the files after sending
      } catch (error) {
        console.error('Failed to send message:', error);
      }
    },
    triggerFileUpload() {
      this.$refs.fileInput.click();
    },
    async handleFileUpload(event) {
      const files = Array.from(event.target.files);
      if (files.length === 0) return;

      try {
        const uploadedFiles = await this.$store.dispatch('uploadFiles', files);
        this.files = [...this.files, ...uploadedFiles];
      } catch (error) {
        console.error('Failed to upload files:', error);
      }
    },
  },
};
</script>