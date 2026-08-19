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

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf

EXPOSE 80
