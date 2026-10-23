FROM node:20-alpine AS dev
WORKDIR /app
RUN apk add --no-cache git
COPY client/package*.json ./
RUN npm ci
COPY client/ .
EXPOSE 3000
CMD ["npm", "run", "dev"]