#!/usr/bin/env python3
"""
Example Python client for the Amazon Q Developer API
"""

import json
import requests
import sseclient  # pip install sseclient-py

API_BASE_URL = "http://localhost:8080/api"

def send_chat_request(message, conversation_id=None):
    """Send a regular chat request"""
    url = f"{API_BASE_URL}/chat"
    payload = {
        "message": message
    }
    
    if conversation_id:
        payload["conversation_id"] = conversation_id
        
    response = requests.post(url, json=payload)
    response.raise_for_status()
    return response.json()

def send_streaming_chat_request(message, conversation_id=None):
    """Send a streaming chat request"""
    url = f"{API_BASE_URL}/chat/stream"
    payload = {
        "message": message
    }
    
    if conversation_id:
        payload["conversation_id"] = conversation_id
        
    response = requests.post(url, json=payload, stream=True)
    response.raise_for_status()
    
    full_response = ""
    client = sseclient.SSEClient(response)
    
    for event in client.events():
        try:
            chunk = json.loads(event.data)
            print(f"Received chunk: {chunk}")
            
            if chunk["chunk_type"] == "text":
                full_response += chunk["content"]
                
            if chunk["is_final"]:
                print("Stream completed")
                break
        except json.JSONDecodeError:
            print(f"Error parsing chunk: {event.data}")
    
    return full_response

def get_conversations():
    """Get all conversations"""
    url = f"{API_BASE_URL}/chat/conversations"
    response = requests.get(url)
    response.raise_for_status()
    return response.json()

def main():
    """Main function"""
    try:
        # Send a regular chat request
        print("Sending regular chat request...")
        chat_response = send_chat_request("How do I create an S3 bucket using AWS CDK?")
        print(f"Chat response: {json.dumps(chat_response, indent=2)}")
        
        # Continue the conversation
        if "conversation_id" in chat_response:
            print("\nContinuing conversation...")
            follow_up = send_chat_request(
                "Can you explain more about the versioned property?",
                chat_response["conversation_id"]
            )
            print(f"Follow-up response: {json.dumps(follow_up, indent=2)}")
        
        # Send a streaming chat request
        print("\nSending streaming chat request...")
        full_response = send_streaming_chat_request("What are the best practices for S3 bucket security?")
        print(f"Full streaming response: {full_response}")
        
        # Get all conversations
        print("\nGetting all conversations...")
        conversations = get_conversations()
        print(f"Conversations: {json.dumps(conversations, indent=2)}")
        
    except Exception as e:
        print(f"Error: {e}")

if __name__ == "__main__":
    main()
