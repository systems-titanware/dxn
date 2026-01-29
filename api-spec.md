# DXN API Specification

This document provides a comprehensive API specification for the DXN server, designed for mobile client integration.

**Base URL**: `http://127.0.0.1:8080`

---

## Table of Contents

1. [Server Routes](#server-routes)
2. [Function Routes](#function-routes)
3. [Data Routes](#data-routes)
4. [Config Routes](#config-routes)
5. [Common Patterns](#common-patterns)
6. [Error Handling](#error-handling)

---

## Server Routes

### Overview
Server routes are dynamically generated from the `config.json` file's `server.public` configuration. These routes serve HTML content and support nested/hierarchical routing.

### Route Pattern
```
GET /server/{route_path}
```

### Description
- Routes are defined hierarchically in the config (parent routes can have child routes)
- Each route maps to an HTML file in the `dxn-files/routes/` directory
- Routes can optionally specify a layout template
- Routes are flattened at runtime (e.g., `/server/test/baby-test`)

### Example Routes
Based on the config structure:
- `GET /server/test` → serves `home.html` with `global.layout.html` layout
- `GET /server/test/baby-test` → serves `subpage.html` with `global.layout.html` layout
- `GET /server/tester` → serves `tester.html` with `global.layout.html` layout

### Response
- **Content-Type**: `text/html`
- **Status Codes**:
  - `200 OK`: HTML content returned
  - `404 Not Found`: Route not found (returns HTML 404 page)
  - `500 Internal Server Error`: Server error (returns HTML 500 page)

### Notes
- Routes are case-sensitive
- Trailing slashes are handled automatically
- Layout templates use Handlebars for variable substitution

---

## Function Routes

### Overview
Function routes allow execution of server-side functions defined in the `config.json` file's `functions.public` configuration.

### Execute Function
```
POST /api/function/{function_name}
POST /api/function/{function_name}/
```

### Path Parameters
- `function_name` (string, required): Name of the function to execute (as defined in config.json)

### Request Body
JSON object where keys are parameter names and values are the parameter values. The values will be converted to an array in the order they appear in the HashMap.

**Content-Type**: `application/json`

**Example Request**:
```json
{
  "address": "0x1234...",
  "network": "mainnet",
  "balance": 1000
}
```

### Response
**Success (200 OK)**:
```json
{
  // Function-specific result (varies by function)
}
```

**Error Responses**:
- `400 Bad Request`: Function execution returned an error
  ```json
  {
    "error": "Error message from function"
  }
  ```
- `404 Not Found`: Function not found
  ```json
  {
    "error": "Function '{function_name}' not found"
  }
  ```
- `500 Internal Server Error`: Function execution error
  ```json
  {
    "error": "Function execution error: {details}"
  }
  ```

### Available Function Types
Functions can be of type:
- `wasm`: WebAssembly functions
- `native`: Native library functions
- `remote`: Remote service functions
- `script`: Script-based functions (TypeScript, JavaScript)

### Example
```bash
POST /api/function/wallet_get_balance_wasm
Content-Type: application/json

{
  "address": "0x1234567890abcdef"
}
```

---

## Data Routes

### Overview
Data routes provide CRUD operations for data models defined in `config.json`'s `data.public` configuration. Each model gets its own set of routes.

### Route Pattern
All routes follow the pattern: `/api/data/{model_name}/{action}`

Where `{model_name}` is the name of the data model (e.g., "profile", "wallet").

### List Records
```
GET /api/data/{model_name}/list
GET /api/data/{model_name}/list/
```

### Query Parameters
- `page_size` (u8, optional): Number of records per page (default: 10)
- `page` (u8, optional): Page number (default: 10)
- `query` (string, optional): Search query (currently reserved for future use)

### Response
```json
[
  {
    "id": 1,
    "email": "user@example.com",
    "phone": "+1234567890"
  },
  {
    "id": 2,
    "email": "user2@example.com",
    "phone": "+0987654321"
  }
]
```

### Get Single Record
```
GET /api/data/{model_name}/{id}
GET /api/data/{model_name}/{id}/
```

### Path Parameters
- `id` (u32, required): Record ID

### Response
```json
{
  "id": 1,
  "email": "user@example.com",
  "phone": "+1234567890"
}
```

**Error**: Returns error message as plain text if record not found

### Create Record
```
POST /api/data/{model_name}/
POST /api/data/{model_name}
```

### Request Body
JSON object with field names and values matching the model's field definitions.

**Content-Type**: `application/json`

**Example**:
```json
{
  "email": "newuser@example.com",
  "phone": "+1111111111"
}
```

### Response
**Success**: Plain text confirmation message

### Update Record
```
PUT /api/data/{model_name}/{id}
PUT /api/data/{model_name}/{id}/
```

### Path Parameters
- `id` (string, required): Record ID

### Request Body
JSON object with fields to update.

**Example**:
```json
{
  "email": "updated@example.com"
}
```

### Response
**Success**: Plain text confirmation message

### Delete Record
```
DELETE /api/data/{model_name}/{id}
DELETE /api/data/{model_name}/{id}/
```

### Path Parameters
- `id` (u32, required): Record ID

### Response
**Success**: Plain text confirmation message

---

## Migration Routes

### Overview
Migration routes allow management of database schema migrations.

### List Migrations
```
GET /api/data/migrate/list
GET /api/data/migrate/list/
```

### Query Parameters
- `db_name` (string, optional): Database name (default: "public")

### Response
```json
{
  "db_name": "public",
  "migrations": [
    {
      "id": "migration_001",
      "description": "Add users table",
      "created_at": "2024-01-01T00:00:00Z",
      "applied": true,
      "requires_approval": false
    }
  ],
  "summary": {
    "total": 10,
    "applied": 8,
    "pending": 2
  }
}
```

### Apply Single Migration
```
POST /api/data/migrate/{migration_id}
POST /api/data/migrate/{migration_id}/
```

### Path Parameters
- `migration_id` (string, required): Migration ID

### Request Body
```json
{
  "db_name": "public",
  "force": false
}
```

### Response
**Success (200 OK)**:
```json
{
  "status": "success",
  "message": "Migration 'migration_001' applied successfully",
  "migration_id": "migration_001"
}
```

**Requires Approval (400 Bad Request)**:
```json
{
  "status": "requires_approval",
  "message": "Reason for approval requirement",
  "migration_id": "migration_001",
  "hint": "Set 'force': true in request body to apply this migration"
}
```

**Failed (500 Internal Server Error)**:
```json
{
  "status": "failed",
  "error": "Error details",
  "migration_id": "migration_001"
}
```

### Apply All Pending Migrations
```
POST /api/data/migrate/all
POST /api/data/migrate/all/
```

### Request Body
```json
{
  "db_name": "public",
  "force": false
}
```

### Response
```json
{
  "status": "completed",
  "applied": ["migration_001", "migration_002"],
  "requires_approval": [
    {
      "migration_id": "migration_003",
      "reason": "Potentially destructive operation"
    }
  ],
  "failed": [
    {
      "migration_id": "migration_004",
      "error": "Error message"
    }
  ],
  "summary": {
    "total": 4,
    "applied": 2,
    "requires_approval": 1,
    "failed": 1
  }
}
```

---

## Config Routes

### Overview
Config routes provide read-only access to the server's configuration, including function models, data models, and server routes. All routes support pagination.

### Get Function Models
```
GET /api/config/functions
GET /api/config/functions/
```

### Query Parameters
- `page_size` (u8, optional): Number of items per page (default: 10)
- `page` (u8, optional): Page number (default: 1)

### Response
```json
{
  "data": [
    {
      "name": "wallet_get_balance_wasm",
      "functionType": "wasm",
      "version": 1,
      "path": "./dxn-wasm-wallet/target/wasm32-unknown-unknown/release/dxn_wasm_wallet.wasm",
      "functionName": "get_balance",
      "parameters": ["String"],
      "return": "String"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 10,
    "total": 7,
    "total_pages": 1
  }
}
```

### Get Data Models
```
GET /api/config/data
GET /api/config/data/
```

### Query Parameters
- `page_size` (u8, optional): Number of items per page (default: 10)
- `page` (u8, optional): Page number (default: 1)

### Response
```json
{
  "data": [
    {
      "name": "profile",
      "version": 1,
      "db": "public",
      "fields": [
        {
          "name": "email",
          "datatype": "text",
          "value": "{vault.profile.email}"
        },
        {
          "name": "phone",
          "datatype": "text",
          "value": "{vault.profile.phone}"
        }
      ]
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 10,
    "total": 2,
    "total_pages": 1
  }
}
```

### Get Server Routes
```
GET /api/config/server
GET /api/config/server/
```

### Query Parameters
- `page_size` (u8, optional): Number of items per page (default: 10)
- `page` (u8, optional): Page number (default: 1)

### Response
```json
{
  "data": [
    {
      "name": "test",
      "file": "home.html",
      "layout": "global.layout.html",
      "full_path": "test",
      "url": "http://127.0.0.1:8080/server/test"
    }
  ],
  "pagination": {
    "page": 1,
    "page_size": 10,
    "total": 2,
    "total_pages": 1
  }
}
```

---

## Common Patterns

### Pagination
Several endpoints support pagination using query parameters:
- `page_size`: Number of items per page (default: 10)
- `page`: Page number, 1-indexed (default: 1)

**Pagination Response Format**:
```json
{
  "data": [...],
  "pagination": {
    "page": 1,
    "page_size": 10,
    "total": 50,
    "total_pages": 5
  }
}
```

### Query Parameters
Query parameters should be URL-encoded when necessary. Boolean and numeric values should be passed as strings in query parameters.

**Example**:
```
GET /api/config/functions?page=2&page_size=20
```

---

## Error Handling

### Standard Error Responses

#### 400 Bad Request
Returned when:
- Function execution returns an error
- Migration requires approval

**Format**:
```json
{
  "error": "Error message",
  // Additional fields may be present
}
```

#### 404 Not Found
Returned when:
- Function not found
- Route not found
- Record not found

**Format**:
- JSON endpoints: `{"error": "Error message"}`
- HTML endpoints: HTML 404 page

#### 500 Internal Server Error
Returned when:
- Server-side errors occur
- Database errors
- Function execution errors

**Format**:
- JSON endpoints: `{"error": "Error details"}`
- HTML endpoints: HTML 500 page

### Error Response Best Practices
- Always check the HTTP status code first
- Parse error messages from the response body
- Handle network errors separately from API errors
- Implement retry logic for transient errors (5xx status codes)

---

## Authentication & Authorization

**Note**: The current API specification does not include authentication/authorization details. These may be implemented at a higher layer or via middleware. Mobile clients should be prepared to handle:
- Authentication tokens (if implemented)
- Session management
- Rate limiting responses

---

## Rate Limiting

**Note**: Rate limiting is not currently documented. Mobile clients should implement:
- Exponential backoff for retries
- Request queuing for high-frequency operations
- Caching strategies where appropriate

---

## Versioning

**Current Version**: Not explicitly versioned in the URL path. The API version may be inferred from:
- Function `version` fields in config responses
- Data model `version` fields in config responses

---

## Mobile Client Integration Notes

### Recommended Practices

1. **Discovery**: Use `/api/config/*` endpoints to discover available:
   - Functions (for dynamic function execution UI)
   - Data models (for dynamic CRUD UI)
   - Server routes (for navigation)

2. **Caching**: Consider caching config responses as they change infrequently.

3. **Error Handling**: Implement comprehensive error handling for all endpoints.

4. **Pagination**: Always implement pagination for list endpoints to handle large datasets.

5. **Type Safety**: Use the config endpoints to generate type-safe models in your mobile client.

### Example Integration Flow

1. **App Startup**:
   - Fetch `/api/config/functions` to discover available functions
   - Fetch `/api/config/data` to discover available data models
   - Cache these for the session

2. **User Actions**:
   - Use discovered functions to build dynamic UI
   - Use data models to build dynamic forms
   - Execute functions via `/api/function/{name}`
   - Perform CRUD operations via `/api/data/{model}`

3. **Error Recovery**:
   - Implement retry logic for failed requests
   - Show user-friendly error messages
   - Log errors for debugging

---

## Changelog

- **2026-01-27**: Initial API specification document created

---

## Support

For issues or questions regarding the API, please refer to the project documentation or contact the development team.
