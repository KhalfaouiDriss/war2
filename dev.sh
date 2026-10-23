#!/bin/bash
# War2 Development Startup Script

set -e

echo "🚀 Starting War2 development environment..."

# Check for .env file
if [ ! -f .env ]; then
    echo "📝 No .env file found, copying from .env.example..."
    cp .env.example .env
    echo "⚠️  Please edit .env with your settings (especially JWT_SECRET)"
fi

# Load environment variables
export $(grep -v '^#' .env | xargs)

# Build WASM if needed
if [ ! -d "client/pkg" ]; then
    echo "🔨 Building WASM..."
    make wasm
fi

# Start services
echo "🐳 Starting Docker services..."
docker compose up -d --build

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 10

# Check health
echo "🏥 Checking service health..."
if curl -sf http://localhost:8080/health > /dev/null; then
    echo "✅ Gateway is healthy"
else
    echo "⚠️  Gateway health check failed"
fi

echo ""
echo "🎮 War2 development environment is ready!"
echo ""
echo "Services:"
echo "  - Gateway (WebSocket):  ws://localhost:8080"
echo "  - Client (Vite):        http://localhost:3000"
echo "  - Shard 1 (TCP):        localhost:9001"
echo "  - Shard 2 (TCP):        localhost:9002"
echo "  - Redis:                localhost:6379"
echo "  - PostgreSQL:           localhost:5432"
echo ""
echo "Commands:"
echo "  make logs    - View logs"
echo "  make down    - Stop services"
echo "  make test    - Run tests"
echo "  make lint    - Run linter"