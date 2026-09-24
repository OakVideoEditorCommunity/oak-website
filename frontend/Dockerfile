FROM node:22-slim AS builder

WORKDIR /app

COPY package*.json ./
RUN npm install

COPY . .

# The site is fully static (nuxi generate): public runtime config is baked
# into the output here, at build time. Override with --build-arg if needed.
ARG NUXT_PUBLIC_SITE_URL=https://www.oakvideoeditor.org
ARG NUXT_PUBLIC_API_BASE_URL=
ARG NUXT_PUBLIC_CDN_DOMAIN=
ENV NUXT_PUBLIC_SITE_URL=$NUXT_PUBLIC_SITE_URL \
    NUXT_PUBLIC_API_BASE_URL=$NUXT_PUBLIC_API_BASE_URL \
    NUXT_PUBLIC_CDN_DOMAIN=$NUXT_PUBLIC_CDN_DOMAIN

RUN npm run generate

FROM nginx:alpine

COPY --from=builder /app/.output/public /usr/share/nginx/html
COPY nginx.conf.template /etc/nginx/templates/default.conf.template

# The nginx entrypoint envsubstitutes this into /etc/nginx/conf.d/default.conf
# (see nginx.conf.template). Compose DNS name of the backend by default.
ENV API_UPSTREAM=http://backend:8080

EXPOSE 3000
