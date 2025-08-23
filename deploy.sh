#!/bin/bash

# Tatteau Web Application Deployment Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Docker is installed
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker first."
    exit 1
fi

# Check if Docker Compose is installed
if ! command -v docker-compose &> /dev/null; then
    print_error "Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Parse command line arguments
ENVIRONMENT="development"
BUILD_FRESH=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --env)
            ENVIRONMENT="$2"
            shift 2
            ;;
        --fresh-build)
            BUILD_FRESH=true
            shift
            ;;
        --help)
            echo "Usage: $0 [--env development|production] [--fresh-build] [--help]"
            echo ""
            echo "Options:"
            echo "  --env            Set environment (development or production)"
            echo "  --fresh-build    Force rebuild without cache"
            echo "  --help          Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

print_status "Deploying Tatteau Web Application..."
print_status "Environment: $ENVIRONMENT"

# Check if database exists
if [ ! -f "tatteau.db" ]; then
    print_warning "Database file 'tatteau.db' not found. The application may not work properly."
    print_warning "Make sure to create and populate the database before running the application."
fi

# Build and start services
if [ "$BUILD_FRESH" = true ]; then
    print_status "Building application with fresh cache..."
    docker-compose build --no-cache
else
    print_status "Building application..."
    docker-compose build
fi

# Stop any running containers
print_status "Stopping existing containers..."
docker-compose down

# Start services based on environment
if [ "$ENVIRONMENT" = "production" ]; then
    print_status "Starting production services (with nginx)..."
    docker-compose --profile production up -d
else
    print_status "Starting development services..."
    docker-compose up -d tatteau-web
fi

# Wait for services to be healthy
print_status "Waiting for services to be ready..."
sleep 10

# Check if the application is running
if curl -f http://localhost:3000/ > /dev/null 2>&1; then
    print_status "✅ Application is running successfully!"
    echo ""
    print_status "Access the application at:"
    if [ "$ENVIRONMENT" = "production" ]; then
        echo "  🌐 http://localhost (via nginx)"
        echo "  🔧 http://localhost:3000 (direct)"
    else
        echo "  🌐 http://localhost:3000"
    fi
    echo ""
    print_status "To view logs: docker-compose logs -f"
    print_status "To stop: docker-compose down"
else
    print_error "❌ Application failed to start or is not responding"
    print_status "Check logs with: docker-compose logs tatteau-web"
    exit 1
fi