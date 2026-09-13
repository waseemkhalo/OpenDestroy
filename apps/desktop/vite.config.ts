import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { fileURLToPath, URL } from 'node:url';
export default defineConfig({plugins:[svelte()],resolve:{alias:{$lib:fileURLToPath(new URL('./src/lib',import.meta.url))}},clearScreen:false,server:{strictPort:true,port:1422},test:{environment:'node',include:['src/**/*.test.ts']}});
