## DXN Data API Contract – Mobile Client Guide

This document summarizes the **data API** (`/api/data/...`) as implemented in `dxn-core`, with a focus on how the **mobile client** should consume responses and handle success / error cases.

Base URL (local dev): `http://127.0.0.1:8080`

All data endpoints return **JSON** with a consistent envelope.

---

## 1. Common Response Envelope

Every `/api/data/...` endpoint now returns a JSON object of the form:

```json
{
  "success": true,
  "data": { },
  "error": null,
  "meta": null
}
```

### 1.1 Fields

- **`success`**: `boolean`
  - `true` if the operation completed successfully
  - `false` if there was any error (client or server side)

- **`data`**: `object | array | null`
  - For `GET` (single): the record object
  - For `GET /list`: an array of record objects
  - For `POST` / `PUT` / `DELETE`: a small object describing the result (see per‑endpoint sections)
  - `null` when `success = false`

- **`error`**: `object | null`
  - Present when `success = false`
  - Shape:
    ```json
    {
      "code": "not_found",
      "message": "Record not found",
      "details": null
    }
    ```
  - **`code`**: stable string for programmatic handling. Current values:
    - `"not_found"` – resource does not exist
    - `"internal_error"` – unexpected server/database error
    - (Future: `"validation_error"`, etc.)
  - **`message`**: human‑readable, safe to display to users
  - **`details`**: reserved for structured error details (currently `null`)

- **`meta`**: `object | null`
  - Used mainly for list/pagination responses
  - Shape:
    ```json
    {
      "page": 1,
      "page_size": 10,
      "total": 42,
      "total_pages": 5
    }
    ```
  - All fields are optional and may be `null` if not applicable.

### 1.2 HTTP Status Codes

The data controller now uses **meaningful HTTP status codes**:

- `200 OK` – successful read, update, or delete
- `201 Created` – successful create
- `404 Not Found` – record not found (id is valid type, but no row)
- `500 Internal Server Error` – unexpected server/database errors

The mobile client **MUST**:

- Check the **HTTP status code** first
- Then inspect the **`success`** and **`error`** fields

---

## 2. Endpoints and Payloads

All routes are mounted under `/api/data/{model_name}` where `{model_name}` is defined in `config.json` under `data.public` (e.g. `profile`, `wallet`).

### 2.1 List Records

**Route**

```http
GET /api/data/{model_name}/list
GET /api/data/{model_name}/list/
```

**Query Params**

- `page_size` (optional, integer; default: `10`)
- `page` (optional, integer; default: `1`)
- `query` (optional, string; currently not used by the backend for filtering)

**Success (200)**

```json
{
  "success": true,
  "data": [
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
  ],
  "error": null,
  "meta": {
    "page": 1,
    "page_size": 10,
    "total": 2,
    "total_pages": 1
  }
}
```

> Note: `total` and `total_pages` are computed from the currently returned list length. The backend’s SQL layer does not (yet) perform real OFFSET/LIMIT pagination; treat this as **client‑side convenience metadata**, not strong guarantees of total row count.

**Error (500)**

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "internal_error",
    "message": "An internal error occurred while listing records",
    "details": null
  },
  "meta": null
}
```

**Mobile client handling**

- On `success: true`:
  - Use `data` as an array of arbitrary objects (keys/fields defined by the model).
  - Use `meta.page`, `meta.page_size` to drive pagination UI.
- On `success: false`:
  - Treat as a hard failure; show a generic error using `error.message`.
  - Log `error.code` for debugging/telemetry.

---

### 2.2 Get Single Record

**Route**

```http
GET /api/data/{model_name}/{id}
GET /api/data/{model_name}/{id}/
```

**Path Params**

- `id` – integer primary key

**Success (200)**

```json
{
  "success": true,
  "data": {
    "id": 1,
    "email": "user@example.com",
    "phone": "+1234567890"
  },
  "error": null,
  "meta": null
}
```

**Not Found (404)**

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "not_found",
    "message": "Record not found",
    "details": null
  },
  "meta": null
}
```

**Internal Error (500)**

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "internal_error",
    "message": "An internal error occurred while retrieving the record",
    "details": null
  },
  "meta": null
}
```

**Mobile client handling**

- `404` / `error.code = "not_found"`:
  - Show “not found” UI; safe to allow user to navigate back or remove local cache for that id.
- `500` / `internal_error`:
  - Show generic “something went wrong” and provide retry option.

---

### 2.3 Create Record

**Route**

```http
POST /api/data/{model_name}/
POST /api/data/{model_name}
```

**Body**

JSON object whose keys match the model’s fields (e.g. from `/api/config/data`).

Example (for `profile`):

```json
{
  "email": "newuser@example.com",
  "phone": "+1111111111"
}
```

**Success (201 Created)**

```json
{
  "success": true,
  "data": {
    "id": 123,
    "object": "profile",
    "attributes": {
      "email": "newuser@example.com",
      "phone": "+1111111111"
    }
  },
  "error": null,
  "meta": null
}
```

Notes:

- `id` is the database primary key generated by SQLite.
- `object` is the model name (e.g. `"profile"`, `"wallet"`).
- `attributes` echoes back the payload.

**Error (500)**

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "internal_error",
    "message": "An internal error occurred while creating the record",
    "details": null
  },
  "meta": null
}
```

> At the moment, all DB failures are mapped to `internal_error`. Future versions may introduce `validation_error` codes for constraint violations.

**Mobile client handling**

- On success:
  - Store the new record using `data.id` as the canonical identifier.
  - Use `data.attributes` as the created object’s fields.
- On error:
  - Show generic failure; optionally allow retry.

---

### 2.4 Update Record

**Route**

```http
PUT /api/data/{model_name}/{id}
PUT /api/data/{model_name}/{id}/
```

**Path Params**

- `id` – record identifier as string in the URL, but treated as the primary key in the DB.

**Body**

JSON object with the fields to update:

```json
{
  "email": "updated@example.com"
}
```

**Success (200)**

```json
{
  "success": true,
  "data": {
    "id": "123",
    "object": "profile",
    "updated": true
  },
  "error": null,
  "meta": null
}
```

**Not Found (404)**

If no rows were affected by the update:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "not_found",
    "message": "Record not found",
    "details": null
  },
  "meta": null
}
```

**Internal Error (500)**

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "internal_error",
    "message": "An internal error occurred while updating the record",
    "details": null
  },
  "meta": null
}
```

**Mobile client handling**

- Treat `updated: true` as confirmation that the backend accepted the changes.
- For `404/not_found`, you should:
  - Remove the record from local state (it no longer exists) or
  - Show a “record no longer available” message.

---

### 2.5 Delete Record

**Route**

```http
DELETE /api/data/{model_name}/{id}
DELETE /api/data/{model_name}/{id}/
```

**Path Params**

- `id` – integer primary key

**Success (200)**

```json
{
  "success": true,
  "data": {
    "id": 123,
    "object": "profile",
    "deleted": true
  },
  "error": null,
  "meta": null
}
```

**Not Found (404)**

If there was no record with that `id`:

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "not_found",
    "message": "Record not found",
    "details": null
  },
  "meta": null
}
```

**Internal Error (500)**

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "internal_error",
    "message": "An internal error occurred while deleting the record",
    "details": null
  },
  "meta": null
}
```

**Mobile client handling**

- On success:
  - Remove the record from local state and any cached lists.
- On `404/not_found`:
  - Record already gone; safe to treat as deleted on the client as well.

---

## 3. Client‑Side Best Practices

1. **Always branch on HTTP status code first**, then check `body.success` / `body.error`.
2. **Do not rely on specific record fields** for all models:
   - For model‑agnostic UI, treat `data` as opaque maps keyed by field names discovered via `/api/config/data`.
3. **Use `error.code` for programmatic flows**:
   - Example: `if (error.code === "not_found") { showNotFound(); }`
4. **Be defensive about `meta` values**:
   - `total`/`total_pages` are helpful hints but not hard guarantees (pagination is not fully implemented at the SQL layer yet).
5. **Log `error` objects for observability**:
   - Especially important for `internal_error` to aid backend debugging.

---

## 4. Summary of Changes vs Previous Contract

If you integrated with the earlier version of the API:

- **Plain text responses are gone** – everything is now JSON.
- **CRUD endpoints share a unified envelope**:
  - `success`, `data`, `error`, `meta`
- **HTTP status codes are meaningful**:
  - `201` for create, `404` for missing records, `500` for server errors.
- **Delete/Update now tell you if the record existed** via:
  - `error.code = "not_found"` when no rows were affected.

The mobile client should be updated to:

- Expect JSON envelopes on all data routes
- Handle `success` / `error` as described above
- Update any code that assumed plain string responses for create/update/delete

