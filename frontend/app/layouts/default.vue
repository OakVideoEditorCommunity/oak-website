<template>
  <div class="min-h-screen flex flex-col bg-forest-950 text-emerald-50">
    <AppHeader />
    <main class="flex-1">
      <slot />
    </main>
    <AppFooter />
  </div>
</template>
