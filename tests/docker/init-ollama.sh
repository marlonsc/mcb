#!/bin/sh

# Wait for Ollama to be ready
until curl -s http://localhost:11434/api/tags >/dev/null; do
	echo "Waiting for Ollama..."
	sleep 2
done

echo "Pulling nomic-embed-text model..."
ollama pull nomic-embed-text

echo "Ollama initialized!"
