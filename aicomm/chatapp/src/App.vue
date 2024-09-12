<!--<template>-->
<!--  <div class="h-full flex flex-col overflow-hidden">-->
<!--    <router-view />-->
<!--  </div>-->
<!--</template>-->

<!--<script>-->
<!--export default {-->
<!--  name: 'App',-->
<!--};-->
<!--</script>-->
<template>
  <div class="h-full flex flex-col overflow-hidden">
    <router-view/>
    <AddChannelModal v-if="isAddChannelModalVisible" @close="isAddChannelModalVisible = false"
                     @add-channel="addChannel"/>
  </div>
</template>

<script>
import AddChannelModal from './components/AddChannelModal.vue';

export default {
  name: 'App',
  components: {
    AddChannelModal,
  },
  data() {
    return {
      isAddChannelModalVisible: false,
    };
  },
  methods: {
    addChannel({name, userIds}) {
      const newChannel = {
        id: Date.now().toString(),
        name,
        members: [this.$store.state.user.id, ...userIds],
        type: 'single',
      };
      this.$store.dispatch('addChannel', newChannel);
      this.isAddChannelModalVisible = false;
    },
  },
};
</script>