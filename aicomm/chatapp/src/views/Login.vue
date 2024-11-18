<template>
  <div class="flex items-center justify-center min-h-screen bg-gray-100">
    <div class="w-full max-w-md p-8 space-y-8 bg-white rounded-xl shadow-2xl">
      <h1 class="text-3xl font-bold text-center text-gray-800">Welcome Back</h1>
      <p class="text-center text-gray-600">Please login to your account</p>
      <form class="mt-8 space-y-6" @submit.prevent="login">
        <div>
          <label class="block text-sm font-medium text-gray-700" for="email">Email</label>
          <input id="email" v-model="email" class="mt-1 block w-full px-3 py-2 bg-gray-50 border border-gray-300 rounded-md text-sm shadow-sm placeholder-gray-400
                        focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500" placeholder="Enter your email" required
                 type="email"/>
        </div>

        <div>
          <label class="block text-sm font-medium text-gray-700" for="password">Password</label>
          <input id="password" v-model="password" class="mt-1 block w-full px-3 py-2 bg-gray-50 border border-gray-300 rounded-md text-sm shadow-sm placeholder-gray-400
                        focus:outline-none focus:border-blue-500 focus:ring-1 focus:ring-blue-500" placeholder="Enter your password" required
                 type="password"/>
        </div>

        <button class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 transition duration-150 ease-in-out"
                type="submit">
          Login
        </button>
      </form>

      <p class="mt-2 text-center text-sm text-gray-600">
        Don't have an account?
        <router-link class="font-medium text-blue-600 hover:text-blue-500" to="/register">
          Register here
        </router-link>
        .
      </p>
    </div>
  </div>
</template>

<script>
export default {
  data() {
    return {
      email: '',
      password: '',
    };
  },
  methods: {
    async login() {
      try {
        this.$store.dispatch('userLogin', {email: this.email});

        const user = await this.$store.dispatch('signin', {
          email: this.email,
          password: this.password,
        });

        console.log('Signin successful, user:', user);
        this.$router.push('/'); // Redirect to chat after successful signup
      } catch (error) {
        console.error('Signin failed:', error);
        // Handle signin failure, show error message to user, etc.
      }
    },
  },
};
</script>