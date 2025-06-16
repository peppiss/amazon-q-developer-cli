# Amazon Q Developer API Server

This crate provides a REST API server for the Amazon Q Developer CLI, allowing remote access to Amazon Q's capabilities.

## Features

- REST API for Amazon Q chat conversations
- Support for both streaming and non-streaming responses
- Conversation management
- Context file support

## API Endpoints

### Chat

- `POST /api/chat`: Send a message to Amazon Q and receive a complete response
- `POST /api/chat/stream`: Send a message to Amazon Q and receive a streaming response
- `GET /api/chat/conversations`: Get all active conversations

## Request/Response Format

### Chat Request

```json
{
  "message": "How do I create an S3 bucket using AWS CDK?",
  "conversation_id": "optional-conversation-id",
  "context_files": [
    {
      "name": "example.ts",
      "content": "// TypeScript code here"
    }
  ]
}
```

### Chat Response

```json
{
  "conversation_id": "conversation-uuid",
  "message": "To create an S3 bucket using AWS CDK...",
  "code_snippets": [
    {
      "language": "typescript",
      "code": "import * as cdk from 'aws-cdk-lib';\nimport * as s3 from 'aws-cdk-lib/aws-s3';\n\nexport class MyStack extends cdk.Stack {\n  constructor(scope: cdk.App, id: string, props?: cdk.StackProps) {\n    super(scope, id, props);\n\n    new s3.Bucket(this, 'MyBucket', {\n      versioned: true,\n      removalPolicy: cdk.RemovalPolicy.DESTROY,\n      autoDeleteObjects: true\n    });\n  }\n}"
    }
  ],
  "citations": [
    {
      "source": "AWS CDK Documentation",
      "url": "https://docs.aws.amazon.com/cdk/api/v2/docs/aws-cdk-lib.aws_s3.Bucket.html"
    }
  ]
}
```

### Streaming Response Format

Each chunk is a JSON object:

```json
{
  "conversation_id": "conversation-uuid",
  "chunk_type": "text",
  "content": "To create an S3 bucket",
  "is_final": false
}
```

## Prerequisites

### Installing Rust and Cargo on Ubuntu WSL2

1. Update your package list:
```bash
sudo apt update
```

2. Install required dependencies:
```bash
sudo apt install -y build-essential curl
```

3. Install Rust and Cargo using rustup:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

4. Follow the on-screen instructions (typically select option 1 for default installation)

5. Load Rust environment variables:
```bash
source $HOME/.cargo/env
```

6. Verify the installation:
```bash
rustc --version
cargo --version
```

## Running the Server

```bash
cd amazon-q-developer-cli
cargo run --bin amazon-q-developer-api
```

By default, the server runs on `127.0.0.1:8080`. You can configure the host and port using environment variables:

```bash
HOST=0.0.0.0 PORT=3000 cargo run --bin amazon-q-developer-api
```

## Authentication

This implementation does not include authentication. For production use, you should add authentication middleware to protect the API endpoints.
