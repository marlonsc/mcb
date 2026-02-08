#!/bin/bash
set -euo pipefail

MAX_RETRIES=60
RETRY_INTERVAL=1

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
	echo -e "${GREEN}[✓]${NC} $1"
}

log_error() {
	echo -e "${RED}[✗]${NC} $1"
}

check_service() {
	local service_name=$1
	local port=$2
	local health_cmd=$3
	local retries=0

	echo -n "Checking $service_name (port $port)... "

	while [ $retries -lt $MAX_RETRIES ]; do
		if eval "$health_cmd" >/dev/null 2>&1; then
			log_info "$service_name is healthy"
			return 0
		fi

		retries=$((retries + 1))
		if [ $((retries % 10)) -eq 0 ]; then
			echo -n "."
		fi
		sleep $RETRY_INTERVAL
	done

	log_error "$service_name failed to become healthy after ${MAX_RETRIES}s"
	return 1
}

main() {
	echo "Starting health checks for MCB Docker infrastructure..."
	echo ""

	local failed=0

	if ! check_service "Ollama" "11434" "curl -sf http://localhost:11434/api/tags > /dev/null"; then
		failed=$((failed + 1))
	fi

	if ! check_service "Milvus" "19530" "curl -sf http://localhost:9091/healthz > /dev/null"; then
		failed=$((failed + 1))
	fi

	if ! check_service "Redis" "6379" "redis-cli -p 6379 ping > /dev/null"; then
		failed=$((failed + 1))
	fi

	if ! check_service "NATS" "4222" "nc -z localhost 4222 2>/dev/null"; then
		failed=$((failed + 1))
	fi

	if ! check_service "PostgreSQL" "5432" "pg_isready -h localhost -p 5432 -U mcb_user > /dev/null 2>&1"; then
		failed=$((failed + 1))
	fi

	echo ""
	if [ $failed -eq 0 ]; then
		log_info "All services are healthy!"
		echo ""
		echo "Service endpoints:"
		echo "  - Ollama:     http://localhost:11434"
		echo "  - Milvus:     http://localhost:19530"
		echo "  - Redis:      localhost:6379"
		echo "  - NATS:       localhost:4222"
		echo "  - PostgreSQL: localhost:5432 (mcb_user)"
		echo ""
		return 0
	else
		log_error "$failed service(s) failed health check"
		echo ""
		echo "Debugging steps:"
		echo "  1. Check service logs: docker-compose logs <service>"
		echo "  2. List running containers: docker ps"
		echo "  3. Check network: docker network ls"
		echo ""
		return 1
	fi
}

main "$@"
