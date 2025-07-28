# Multi-stage build for DevOps 1.0 application
FROM node:18-alpine AS base

# Set working directory
WORKDIR /app

# Copy package files
COPY package*.json ./

# Install dependencies
RUN npm ci --only=production && npm cache clean --force

# Development stage
FROM base AS development
RUN npm ci
COPY . .
EXPOSE 3000
CMD ["npm", "run", "dev"]

# Build stage
FROM base AS build
COPY . .
RUN npm run build

# Production stage
FROM node:18-alpine AS production

# Create app user
RUN addgroup -g 1001 -S nodejs
RUN adduser -S nextjs -u 1001

# Set working directory
WORKDIR /app

# Copy built application
COPY --from=build --chown=nextjs:nodejs /app/.next ./.next
COPY --from=build --chown=nextjs:nodejs /app/public ./public
COPY --from=build --chown=nextjs:nodejs /app/package*.json ./
COPY --from=build --chown=nextjs:nodejs /app/node_modules ./node_modules

# Switch to non-root user
USER nextjs

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:3000/health || exit 1

# Start application
CMD ["npm", "start"]

# Security scanning stage
FROM production AS security-scan
USER root
RUN apk add --no-cache curl
USER nextjs

# Testing stage
FROM base AS testing
COPY . .
RUN npm ci
RUN npm run test
RUN npm run test:coverage

# Linting stage
FROM base AS linting
COPY . .
RUN npm ci
RUN npm run lint
RUN npm run lint:fix

# Performance testing stage
FROM grafana/k6:latest AS performance
COPY performance-tests /scripts
WORKDIR /scripts
CMD ["k6", "run", "load-test.js"] 