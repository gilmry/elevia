# Multi-stage build: compile the static Astro/Svelte PWA, serve it with nginx.

FROM node:22-slim AS builder
WORKDIR /app

ARG PUBLIC_API_URL=http://localhost:8080
ENV PUBLIC_API_URL=${PUBLIC_API_URL}

ARG PUBLIC_CURRENCY_SYMBOL=FCFA
ENV PUBLIC_CURRENCY_SYMBOL=${PUBLIC_CURRENCY_SYMBOL}

COPY package.json package-lock.json* ./
RUN npm install

COPY . .
RUN npm run build

# Pre-compress once at build time instead of re-gzipping the same immutable,
# content-hashed assets on every request forever - the box serving this only
# has 1 vCPU to spare. nginx serves the .gz sibling directly (gzip_static).
RUN find dist -type f \( -name '*.js' -o -name '*.css' -o -name '*.html' \
      -o -name '*.svg' -o -name '*.json' \) -exec gzip -9 -k {} \;

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
