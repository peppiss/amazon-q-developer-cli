#!/bin/bash
# Example curl commands for the Amazon Q Developer API

# Base URL
API_BASE_URL="http://localhost:8080/api"

# Send a regular chat request
echo "Sending regular chat request..."
RESPONSE=$(curl -s -X POST "${API_BASE_URL}/chat" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "How do I create an S3 bucket using AWS CDK?"
  }')

echo "Response: $RESPONSE"

# Extract conversation ID
CONVERSATION_ID=$(echo $RESPONSE | jq -r '.conversation_id')
echo "Conversation ID: $CONVERSATION_ID"

# Continue the conversation
if [ ! -z "$CONVERSATION_ID" ]; then
  echo -e "\nContinuing conversation..."
  FOLLOW_UP=$(curl -s -X POST "${API_BASE_URL}/chat" \
    -H "Content-Type: application/json" \
    -d "{
      \"message\": \"Can you explain more about the versioned property?\",
      \"conversation_id\": \"$CONVERSATION_ID\"
    }")
  
  echo "Follow-up response: $FOLLOW_UP"
fi

# Send a streaming chat request
echo -e "\nSending streaming chat request..."
echo "Note: This will stream responses. Press Ctrl+C to stop."
curl -N -X POST "${API_BASE_URL}/chat/stream" \
  -H "Content-Type: application/json" \
  -d '{
    "message": "What are the best practices for S3 bucket security?"
  }'

# Get all conversations
echo -e "\nGetting all conversations..."
curl -s "${API_BASE_URL}/chat/conversations" | jq .
