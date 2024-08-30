import { defineConfig } from 'astro/config';

// https://astro.build/config
export default defineConfig({
    output: 'static',
    server: {
        port: 1420,
        host: '0.0.0.0'
    },
    build: { 
        assets: "assets"
    },
    devToolbar: {
        enabled: false
    }
});
