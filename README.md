# Rugistry - Dynamic Registry Service

A modern, real-time registry service for managing dynamic configuration values across microservices and applications. Built with Rust (backend) and Flowbite Svelte (frontend).

## Features

- **Key-Value Management**: Store and retrieve configuration values by key in isolated spaces
- **Real-time Updates**: WebSocket support for live configuration changes
- **Multi-tenancy**: Separate namespaces (spaces) for different applications/microservices
- **Modern UI**: Beautiful, responsive interface built with Flowbite Svelte
- **Type Safety**: Strongly typed values (string, number, boolean, JSON, TOML, YAML, HOCON)

## Quick Start

### Prerequisites
- Docker and Docker Compose (recommended)
- OR manually: Rust 1.75+, Node.js 20+, PostgreSQL 16+

### Using Docker Compose (Recommended)

```bash
# Start all services (PostgreSQL, Backend, Frontend)
docker-compose up -d

# Or use Makefile shortcuts:
make start          # Start all services
make status         # Show service status and URLs
make logs           # View all logs
make test           # Run API tests
make stop           # Stop all services

# The API will be available at http://localhost:3000
# The UI will be available at http://localhost:5173
```

Available Makefile commands:
```bash
make help           # Show all available commands
make start          # Start all services
make stop           # Stop all services
make restart        # Restart services
make logs           # View all logs
make build          # Build Docker images
make test           # Run automated tests
make clean          # Remove containers and volumes
make db-connect     # Connect to PostgreSQL
make status         # Show service status
```

### Manual Setup

#### Backend Setup

```bash
cd rugistry-service

# Start PostgreSQL (or use your own instance)
docker run -d \
  --name rugistry-postgres \
  -e POSTGRES_DB=rugistry \
  -e POSTGRES_USER=rugistry_user \
  -e POSTGRES_PASSWORD=rugistry_pass \
  -p 5432:5432 \
  postgres:16-alpine

# Set environment variable
export DATABASE_URL=postgresql://rugistry_user:rugistry_pass@localhost:5432/rugistry

# Build and run
cargo build --release
cargo run

# The API will be available at http://localhost:3000
```

#### Frontend Setup

```bash
cd rugistry-frontend

# Install dependencies
npm install

# Run development server
npm run dev

# The UI will be available at http://localhost:5173
```

## API Documentation

### Spaces

#### Create Space
```http
POST /api/spaces
Content-Type: application/json

{
  "name": "profile-service",
  "description": "Configuration for profile service"
}
```

#### List Spaces
```http
GET /api/spaces
```

#### Get Space by ID
```http
GET /api/spaces/{id}
```

#### Get Space by Name
```http
GET /api/spaces/by-name/{name}
```

#### Update Space
```http
PUT /api/spaces/{id}
Content-Type: application/json

{
  "name": "updated-name",
  "description": "Updated description"
}
```

#### Delete Space
```http
DELETE /api/spaces/{id}
```

### Registry Entries

#### Create Entry
```http
POST /api/entries
Content-Type: application/json

{
  "space_id": "uuid",
  "key": "user_statuses",
  "value": "[\"active\", \"inactive\", \"pending\"]",
  "value_type": "json",
  "description": "Available user statuses"
}
```

#### Get Entry by ID
```http
GET /api/entries/{id}
```

#### Get Entry by Key
```http
GET /api/spaces/{space_id}/entries/by-key?key={key}
```

#### List Entries by Space
```http
GET /api/spaces/{space_id}/entries
```

#### Update Entry
```http
PUT /api/entries/{id}
Content-Type: application/json

{
  "value": "[\"active\", \"inactive\", \"suspended\", \"pending\"]",
  "description": "Updated user statuses"
}
```

#### Delete Entry
```http
DELETE /api/entries/{id}
```

### WebSocket

Connect to receive real-time updates for a specific space:

```
ws://localhost:3000/api/ws/{space_id}
```

Message format:
```json
{
  "event_type": "created|updated|deleted",
  "space_id": "uuid",
  "entry_id": "uuid",
  "key": "string",
  "timestamp": "ISO8601"
}
```

## Usage Example

### 1. Create a Space
```bash
curl -X POST http://localhost:3000/api/spaces \
  -H "Content-Type: application/json" \
  -d '{"name": "profile-service", "description": "Profile service config"}'
```

### 2. Add Configuration Values
```bash
curl -X POST http://localhost:3000/api/entries \
  -H "Content-Type: application/json" \
  -d '{
    "space_id": "{space_id}",
    "key": "user_statuses",
    "value": "[\"active\", \"inactive\", \"pending\"]",
    "value_type": "json"
  }'
```

### 3. Retrieve Values
```bash
# By space
curl http://localhost:3000/api/spaces/{space_id}/entries

# By key
curl "http://localhost:3000/api/spaces/{space_id}/entries/by-key?key=user_statuses"
```

### 4. Subscribe to Changes (WebSocket)
```javascript
const ws = new WebSocket('ws://localhost:3000/api/ws/{space_id}');
ws.onmessage = (event) => {
  const notification = JSON.parse(event.data);
  console.log('Configuration changed:', notification);
};
```

## Database

The service uses PostgreSQL for production-grade performance and reliability.

### Schema

- **spaces**: Namespace isolation for different applications
- **registry_entries**: Key-value configuration entries
- **users**: User management (prepared for Keycloak integration)

### Database Configuration

Set the `DATABASE_URL` environment variable:
```bash
DATABASE_URL=postgresql://user:password@localhost:5432/rugistry
```

The migrations run automatically on startup.

## Keycloak Integration (TODO)

The architecture is prepared for Keycloak SSO integration:

1. Configure Keycloak realm and client
2. Add JWT validation middleware
3. Map Keycloak user IDs to internal users
4. Implement role-based access control

Configuration placeholders are in:
- Backend: Domain/User entity with `keycloak_id` field
- Frontend: Ready for auth provider integration

## Development

### Backend Tests
```bash
cd rugistry-service
cargo test
```

### Frontend Development
```bash
cd rugistry-frontend
npm run dev
```

### Build for Production
```bash
# Backend

Full Docker support with docker-compose for easy deployment:

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f backend

# Stop all services
docker-compose down

# Rebuild after code changes
docker-compose up -d --build
```

The docker-compose setup includes:
- PostgreSQL 16 with persistent volume
- Rust backend service
- Svelte frontend with Nginx
- Automatic health checks and restarts run build
```

## Docker Support (Coming Soon)

```dockerfile
# Backend Dockerfile
# Frontend Dockerfile
# docker-compose.yml
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## License

MIT License

## Roadmap

- [x] Core API functionality
- [x] WebSocket real-time updates
- [x] Frontend UI with Flowbite
- [ ] Keycloak SSO integration
- [ ] Rate limiting
- [ ] Audit logging
- [ ] Backup/restore functionality
- [ ] Import/export configuration
- [ ] API key authentication
- [ ] Role-based access control
