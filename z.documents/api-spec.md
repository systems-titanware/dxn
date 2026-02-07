# DXN API Reference

Base URL: `http://127.0.0.1:8080`

---

## Data API

CRUD operations for data models defined in config or created via Schema API.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/data/{model}` | List records (paginated) |
| GET | `/api/data/{model}/{id}` | Get record by ID |
| POST | `/api/data/{model}` | Create record |
| PUT | `/api/data/{model}/{id}` | Update record |
| DELETE | `/api/data/{model}/{id}` | Delete record |

**Query Parameters:**
- `page` - Page number (default: 1)
- `page_size` - Records per page (default: 20)
- `query` - Search filter

**Note:** Operations on soft-deleted schemas return `410 Gone` with instructions to restore.

---

## Schema API

Manage data model definitions at runtime.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/schema` | List all schemas |
| GET | `/api/schema/{name}` | Get schema |
| POST | `/api/schema` | Create schema |
| PUT | `/api/schema/{name}` | Update schema |
| DELETE | `/api/schema/{name}` | Soft delete schema (restorable) |
| DELETE | `/api/schema/{name}?cascade=true` | Hard delete schema AND data |
| PUT | `/api/schema/{name}/restore` | Restore soft-deleted schema |

**Query Parameters (GET list):**
- `page` - Page number (default: 1)
- `page_size` - Items per page (default: 10)
- `include_deleted` - Include soft-deleted schemas (default: false)

**Query Parameters (DELETE):**
- `cascade` - If true, permanently deletes schema AND drops data table (default: false)

**Create/Update Schema Body:**
```json
{
    "name": "orders",
    "db": "public",
    "public": true,
    "icon": "📦",
    "fields": [
        {"name": "id", "datatype": "integer", "primary": true},
        {"name": "total", "datatype": "number"}
    ]
}
```

**Schema Response (includes status):**
```json
{
    "name": "orders",
    "version": 1,
    "db": "public",
    "public": true,
    "icon": "📦",
    "status": "active",
    "deletedAt": null,
    "fields": [...]
}
```

**Soft Delete Response:**
```json
{
    "data": {
        "deleted": true,
        "name": "orders",
        "cascade": false,
        "restorable": true,
        "note": "Schema soft-deleted. Data preserved."
    }
}
```

**Hard Delete Response (cascade=true):**
```json
{
    "data": {
        "deleted": true,
        "name": "orders",
        "cascade": true,
        "table_dropped": true,
        "permanent": true
    }
}
```

---

## Events API

Query the event store and manage event sourcing.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/events` | List recent events |
| GET | `/api/events/{id}` | Get event by ID |
| GET | `/api/events/aggregate/{id}` | Events for entity |
| GET | `/api/events/schema/{name}` | Events for schema |
| GET | `/api/events/replay/{aggregate_id}` | Replay aggregate state |
| POST | `/api/events/rebuild/{schema}` | Rebuild schema from events |

**Query Parameters:**
- `limit` - Max events to return
- `since` - Filter events after timestamp
- `event_type` - Filter by type (created, updated, deleted)

---

## Files API

Manage file directories and file operations.

### Directory Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/files/directories` | List directories |
| GET | `/api/files/{name}` | Get directory config |
| POST | `/api/files/directories` | Create directory |
| PUT | `/api/files/{name}` | Update directory |
| DELETE | `/api/files/{name}` | Delete directory config |

**Create Directory Body:**
```json
{
    "name": "uploads",
    "provider": "local",
    "path": "/_files/uploads",
    "icon": "📁"
}
```

### File Operations

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/files` | List files by directory (query: `directory`, optional `path`) |
| GET | `/api/files/{name}/list` | List files at root |
| GET | `/api/files/{name}/list/{path}` | List files in path |
| GET | `/api/files/{name}/read/{path}` | Read file (returns content) |
| POST | `/api/files/{name}/write/{path}` | Write file (body = content) |
| POST | `/api/files/upload` | Upload file via multipart/form-data |
| DELETE | `/api/files/{name}/delete/{path}` | Delete file |
| POST | `/api/files/{name}/mkdir/{path}` | Create directory |
| GET | `/api/files/{name}/metadata/{path}` | Get file metadata |

**File List Response:**
```json
{
    "directory": "uploads",
    "path": "images",
    "entries": [
        {
            "name": "photo.jpg",
            "path": "images/photo.jpg",
            "isDirectory": false,
            "size": 102400,
            "mimeType": "image/jpeg"
        }
    ],
    "total": 1
}
```

**File Upload (multipart)**

- **Endpoint:** `POST /api/files/upload`
- **Content-Type:** `multipart/form-data`

**Form fields:**

- `directory` (string, required): Directory name (matches `SystemFileDirectory.name`)
- `path` (string, optional): Relative path inside directory (e.g. `"images/photo.jpg"`)
- `file` (file, required): File blob (the uploaded file)

If `path` is omitted, the server will use the filename from the `file` field.

---

## Config API

Read server configuration.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/config` | Get full config |
| GET | `/api/config/data` | Get data config |
| GET | `/api/config/functions` | Get functions config |

---

## Function API

Execute functions.

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/function/{name}` | Execute function |

**Request Body:**
```json
{
    "params": [30, 12]
}
```

---

## Response Envelope

All responses follow a standard envelope:

**Success:**
```json
{
    "data": { ... },
    "meta": {
        "page": 1,
        "pageSize": 20,
        "total": 100,
        "totalPages": 5
    }
}
```

**Error:**
```json
{
    "error": "not_found",
    "message": "Record not found",
    "details": null
}
```
