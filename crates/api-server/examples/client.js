// Example JavaScript client for the Amazon Q Developer API

// Regular chat request
async function sendChatRequest() {
  const response = await fetch('http://localhost:8080/api/chat', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      message: 'How do I create an S3 bucket using AWS CDK?',
    }),
  });
  
  const data = await response.json();
  console.log('Chat response:', data);
  return data;
}

// Streaming chat request
async function sendStreamingChatRequest() {
  const response = await fetch('http://localhost:8080/api/chat/stream', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      message: 'How do I create an S3 bucket using AWS CDK?',
    }),
  });
  
  const reader = response.body.getReader();
  const decoder = new TextDecoder();
  let fullResponse = '';
  
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    
    const chunk = decoder.decode(value);
    try {
      const jsonChunk = JSON.parse(chunk);
      console.log('Received chunk:', jsonChunk);
      
      if (jsonChunk.chunk_type === 'text') {
        fullResponse += jsonChunk.content;
      }
      
      if (jsonChunk.is_final) {
        console.log('Stream completed');
        break;
      }
    } catch (e) {
      console.error('Error parsing chunk:', e);
    }
  }
  
  console.log('Full response:', fullResponse);
}

// Continue a conversation
async function continueConversation(conversationId) {
  const response = await fetch('http://localhost:8080/api/chat', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      message: 'Can you explain more about the versioned property?',
      conversation_id: conversationId,
    }),
  });
  
  const data = await response.json();
  console.log('Follow-up response:', data);
}

// Get all conversations
async function getConversations() {
  const response = await fetch('http://localhost:8080/api/chat/conversations');
  const data = await response.json();
  console.log('Conversations:', data);
}

// Example usage
async function main() {
  try {
    // Send a regular chat request
    const chatResponse = await sendChatRequest();
    
    // Continue the conversation
    if (chatResponse.conversation_id) {
      await continueConversation(chatResponse.conversation_id);
    }
    
    // Send a streaming chat request
    await sendStreamingChatRequest();
    
    // Get all conversations
    await getConversations();
  } catch (error) {
    console.error('Error:', error);
  }
}

main();
