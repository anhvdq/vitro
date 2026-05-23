# API Documentation

This document defines a REST API for the core MVP entities:

- **Datasets**
- **Assets**
- **Tasks**
- **Task Runs**

Base URL: `/`

> Note: you may see `/api/v1` used in examples elsewhere; this backend currently serves routes without a version prefix (e.g. `/datasets`, `/assets`).

Auth: Bearer token (Vitro backend JWT)

- Clients authenticate with the Vitro backend using:
  - `POST /auth/signup` (email/password registration)
  - `POST /auth/login` (email/password login)
- Provider login route exists as placeholder:
  - `POST /auth/exchange` accepts `provider=google` and currently returns `501 Not Implemented`.

Common headers:

```
Authorization: Bearer <access_token>
Content-Type: application/json
X-Workspace-Id: <workspace_id>
```

> Note: `X-Workspace-Id` is required for workspace-scoped routes (`/datasets`, `/assets`, `/tasks`, ...). Auth routes do not require it.

## Auth API

### Signup with email/password

`POST /auth/signup`

Create a user with email/password and return app access token.

#### Request

```json
{
  "email": "john@example.com",
  "password": "secret123",
  "name": "John Doe"
}
```

#### Response `201`

```json
{
  "data": {
    "access_token": "<backend_access_token>",
    "user": {
      "id": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
      "name": "John Doe",
      "age": null,
      "email": "john@example.com",
      "default_workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10"
    }
  }
}
```

### Login with email/password

`POST /auth/login`

Login existing user and return app access token.

#### Request

```json
{
  "email": "john@example.com",
  "password": "secret123"
}
```

#### Response `200`

```json
{
  "data": {
    "access_token": "<backend_access_token>",
    "user": {
      "id": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
      "name": "John Doe",
      "age": null,
      "email": "john@example.com",
      "default_workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10"
    }
  }
}
```

### Exchange provider token (placeholder)

`POST /auth/exchange`

Provider-based login placeholder for future Google support.

#### Request

```json
{
  "provider": "google",
  "token": "<provider_access_token>"
}
```

#### Response `501`

Provider login is not implemented yet.

### Get current user

`GET /auth/me`

Return the current user details for the provided backend access token.

#### Headers

```
Authorization: Bearer <backend_access_token>
```

#### Response `200`

```json
{
  "data": {
    "id": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
    "name": "John Doe",
    "age": null,
    "email": "john@example.com",
    "default_workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10"
  }
}
```

---

## Conventions

### IDs & timestamps

- All IDs are UUID strings.
- Timestamps are ISO-8601 UTC strings, e.g. `2025-03-18T14:21:00Z`.

### Pagination (list endpoints)

Query params:

- `page` (default 1)
- `page_size` (default 20, max 100)
- `sort` (e.g. `created_at:desc`)

Response shape:

```json
{
  "data": [],
  "page": 1,
  "page_size": 20,
  "total": 0
}
```

### Standard error response

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "name is required",
    "details": { "field": "name" },
    "correlation_id": "c2a5b3f6-9e5b-4b2b-a66f-7b3c4c7c0001"
  }
}
```

The same `correlation_id` is also returned in the `X-Correlation-Id` response header.

### HTTP status codes

`200`: Success
`201`: Resource created successfully
`401`: Unauthorized, unknown user
`404`: Not Found, resource not existed
`410`: Gone, the resource has already archived, return when call `DELETE` on an archived resource
`500`: Internal server error

### Error codes (`error.code`)

- `BAD_REQUEST` (400): malformed params/headers/body
- `VALIDATION_ERROR` (400): request validation failed
- `UNAUTHORIZED` (401): missing/invalid/expired bearer token
- `NOT_FOUND` (404): resource not found in workspace scope
- `CONFLICT` (409): uniqueness constraint violated (e.g. duplicate name)
- `GONE` (410): resource already archived
- `INTERNAL_SERVER_ERROR` (500): unexpected server error

---

# 1) Datasets API

## Dataset fields

| Field | Description | Data type | Allowed methods |
| --- | --- | --- | --- |
| `id` | Dataset identifier | UUID | GET |
| `workspace_id` | Tenant/workspace identifier | UUID | GET |
| `name` | Dataset name, unique, snake_case | string | GET, POST |
| `display_name` | Dataset display name, human readable | string | GET, POST, PATCH |
| `description` | Dataset description | string | null | GET, POST, PATCH |
| `owner_user_id` | Owner user ID | UUID | GET, PATCH |
| `owner_user_name` | Owner display name | string | GET |
| `tags` | Labels for search/filter | string[] | GET, POST, PATCH |
| `status` | Lifecycle state | enum(`ACTIVE`,`ARCHIVED`) | GET |
| `created_at` | Creation time | timestamp | GET |
| `updated_at` | Last update time | timestamp | GET |
| `archived_at` | Archive time, null if active | timestamp | null | GET |

## Create dataset

`POST /datasets`

Create a new dataset, the owner is set to the one who send the request

### Request

```json
{
  "name": "marketing_data",
  "display_name": "Marketing Data",
  "description": "Data used for marketing department",
  "tags": ["ecommerce", "sales", "finance"]
}
```

### Response `201`

Create dataset successfully

```json
{
  "data": {
    "id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10",
    "name": "marketing_data",
    "display_name": "Marketing Data",
    "description": "Data used for marketing department",
    "owner_user_id": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
    "owner_user_name": "John Doe",
    "tags": ["ecommerce", "sales", "finance"],
    "status": "ACTIVE",
    "created_at": "2025-01-05T09:12:44Z",
    "updated_at": "2025-01-05T09:12:44Z",
    "archived_at": null
  }
}
```

## List datasets

`GET /datasets?page=1&page_size=20&status=ACTIVE&search=Sales&tags=kpi,sales`

Query list of datasets, support search by status, display name and tags

### Response `200`

```json
{
  "data": [
    {
      "id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
      "workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10",
		  "name": "marketing_data",
      "display_name": "Marketing Data",
      "description": "Data used for marketing department",
      "owner_user_id": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
      "owner_user_name": "John Doe",
      "tags": ["ecommerce", "sales", "finance"],
      "status": "ACTIVE",
      "created_at": "2025-01-05T09:12:44Z",
      "updated_at": "2025-03-18T14:22:10Z",
      "archived_at": null
    }
  ],
  "page": 1,
  "page_size": 20,
  "total": 1
}
```

## Get dataset by id

`GET /datasets/{dataset_id}`

Query dataset details by id

### Response `200`

```json
{
  "data": {
    "id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10",
    "name": "marketing_data",
    "display_name": "Marketing Data",
    "description": "Data used for marketing department",
    "owner_user_id": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
    "owner_user_name": "John Doe",
    "tags": ["ecommerce", "sales", "finance"],
    "status": "ACTIVE",
    "created_at": "2025-01-05T09:12:44Z",
    "updated_at": "2025-03-18T14:22:10Z",
    "archived_at": null
  }
}
```

## Update dataset

`PATCH /datasets/{dataset_id}`

Update dataset details, set `owner_user_id` to a new user ID to transfer ownership

### Request

```json
{
  "display_name": "New Marketing Data Name",
  "description": "Marketing data description updated",
  "owner_user_id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
  "tags": ["ecommerce", "sales", "kpi"]
}
```

### Response `200`

```json
{
  "data": {
    "id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10",
    "name": "marketing_data",
	  "display_name": "New Marketing Data Name",
	  "description": "Marketing data description updated",
	  "owner_user_id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "owner_user_name": "Jane Lee",
	  "tags": ["ecommerce", "sales", "kpi"],
    "status": "ACTIVE",
    "created_at": "2025-01-05T09:12:44Z",
    "updated_at": "2025-03-18T14:22:10Z",
    "archived_at": null
  }
}
```

## Archive dataset

DELETE `/datasets/{dataset_id}`

Archive a dataset

### Response `200`

```json
{
  "data": {
    "id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10",
    "name": "marketing_data",
    "display_name": "Marketing Data",
    "description": "Data used for marketing department",
    "owner_user_id": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
    "owner_user_name": "John Doe",
    "tags": ["ecommerce", "sales", "finance"],
    "status": "ARCHIVED",
    "created_at": "2025-01-05T09:12:44Z",
    "updated_at": "2025-03-26T07:12:00Z",
    "archived_at": "2025-03-26T07:12:00Z"
  }
}
```

---

# 2) Assets API

## Asset fields

| Field | Description | Data type | Allowed methods |
| --- | --- | --- | --- |
| `id` | Asset identifier | UUID | GET |
| `workspace_id` | Tenant/workspace identifier | UUID | GET |
| `dataset_id` | Parent dataset ID | UUID | GET, POST |
| `name` | Asset name (unique across all assets), snake_case | string | GET, POST |
| `display_name` | Asset display name, human readable | string | GET, POST, PATCH |
| `description` | Asset description | string | null | GET, POST, PATCH |
| `owner_user_id` | Owner user ID | UUID | GET, PATCH |
| `owner_user_name` | Owner display name | string | GET |
| `tags` | Labels for search/filter | string[] | GET, POST, PATCH |
| `connector` | Bound connector, define  | JSON Object | GET |
| `connector.id` | Bound connector ID | UUID | GET |
| `connector.type` | Connector type, default to `UPLOAD` type | enum(`UPLOAD`) | GET |
| `connector.storage_prefix` | Storage prefix (immutable) | string | GET |
| `connector.allowed_formats` | Allowed file formats | enum[] (`CSV`) | GET |
| `status` | Lifecycle state | enum(`ACTIVE`,`ARCHIVED`) | GET |
| `created_at` | Creation time | timestamp | GET |
| `updated_at` | Last update time | timestamp | GET |
| `archived_at` | Archive time | timestamp | null | GET |

## Create asset

`POST /assets`

Create a new asset

### Request

```json
{
  "dataset_id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
  "name": "sales_data_daily",
  "display_name": "Sales data daily",
  "description": "Daily order-level sales extracts (one row per order)",
  "tags": ["ecommerce", "sales", "facts"]
}
```

### Response `201`

```json
{
  "data": {
    "id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10",
    "dataset_id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "name": "sales_data_daily",
    "display_name": "Sales data daily",
    "description": "Daily order-level sales extracts (one row per order)",
    "owner_user_id": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
    "owner_user_name": "John Doe",
    "tags": ["ecommerce", "sales", "facts"],
    "connector": {
      "id": "9a8b7c6d-1001-4f01-9a01-111111111111",
      "type": "UPLOAD",
      "storage_prefix": "s3://idea-data-assets/ws/b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10/datasets/2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1/assets/1b2c3d4e-0001-4a11-8b9a-001122334455/",
      "allowed_formats": ["CSV"]
    }
  }
}
```

## List assets

`GET /assets?page=1&page_size=20&dataset_id={dataset_id}&status=ACTIVE&search=Sales&tags=kpi,sales`

Query list of assets, dataset_id must be provided, support search by status, display name and tags

### Response `200`

```json
{
  "data": [
    {
      "id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
      "workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10",
      "dataset_id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
      "name": "sales_data_daily",
      "display_name": "Sales data daily",
      "description": "Daily order-level sales extracts (one row per order)",
      "owner_user_id": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
      "owner_user_name": "John Doe",
      "tags": ["ecommerce", "sales", "facts"],
      "connector": {
        "id": "9a8b7c6d-1001-4f01-9a01-111111111111",
        "type": "UPLOAD",
        "storage_prefix": "s3://idea-data-assets/ws/b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10/datasets/2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1/assets/1b2c3d4e-0001-4a11-8b9a-001122334455/",
        "allowed_formats": ["CSV"]
      }
    }
  ],
  "page": 1,
  "page_size": 20,
  "total": 1
}
```

## Get asset by id

`GET /assets/{asset_id}`

Get asset details by id

### Response `200`

```json
{
  "data": {
    "id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10",
    "dataset_id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "name": "sales_data_daily",
    "display_name": "Sales data daily",
    "description": "Daily order-level sales extracts (one row per order)",
    "owner_user_id": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
    "owner_user_name": "John Doe",
    "tags": ["ecommerce", "sales", "facts"],
    "connector": {
      "id": "9a8b7c6d-1001-4f01-9a01-111111111111",
      "type": "UPLOAD",
      "storage_prefix": "s3://idea-data-assets/ws/b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10/datasets/2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1/assets/1b2c3d4e-0001-4a11-8b9a-001122334455/",
      "allowed_formats": ["CSV"]
    }
  }
}
```

## Update asset

`PATCH /assets/{asset_id}`

Update asset details, set `owner_user_id` to a new user ID to transfer ownership

### Request

```json
{
  "display_name": "Sales data daily",
  "description": "Daily order-level sales extracts (one row per order)",
  "owner_user_id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
  "tags": ["ecommerce", "sales", "facts"]
}
```

### Response `200`

```json
{
  "data": {
    "id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10",
    "dataset_id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "name": "sales_data_daily",
    "display_name": "Sales data daily",
    "description": "Daily order-level sales extracts (one row per order)",
    "owner_user_id": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
    "owner_user_name": "John Doe",
    "tags": ["ecommerce", "sales", "facts"],
    "connector": {
      "id": "9a8b7c6d-1001-4f01-9a01-111111111111",
      "type": "UPLOAD",
      "storage_prefix": "s3://idea-data-assets/ws/b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10/datasets/2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1/assets/1b2c3d4e-0001-4a11-8b9a-001122334455/",
      "allowed_formats": ["CSV"]
    }
  }
}
```

## Archive asset

`DELETE /assets/{asset_id}`

Archive an asset

### Response `200`

```json
{
  "data": {
    "id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10",
    "dataset_id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "name": "sales_data_daily",
    "display_name": "Sales data daily",
    "description": "Daily order-level sales extracts (one row per order)",
    "owner_user_id": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
    "owner_user_name": "John Doe",
    "tags": ["ecommerce", "sales", "facts"],
    "connector": {
      "id": "9a8b7c6d-1001-4f01-9a01-111111111111",
      "type": "UPLOAD",
      "storage_prefix": "ws/b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10/datasets/2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1/assets/1b2c3d4e-0001-4a11-8b9a-001122334455",
      "allowed_formats": ["CSV"]
    },
    "status": "ARCHIVED",
    "created_at": "2025-01-05T09:12:44Z",
    "updated_at": "2025-03-26T07:12:00Z",
    "archived_at": "2025-03-26T07:12:00Z"
  }
}
```

## Upload a file for an asset

`POST /assets/{asset_id}/upload`

Upload a CSV file for an asset using multipart/form-data.

### Request

Content-Type: `multipart/form-data`

Form fields:
- `file` (required): the CSV file to upload

### Response `200`

```json
{
  "data": {
    "path": "./data/ws/<workspace_id>/datasets/<dataset_id>/assets/<asset_id>/<file_name>.csv",
    "file_name": "sales_data_daily.csv",
    "size_bytes": 123456,
    "content_type": "text/csv"
  }
}
```

---

# 3) Tasks API

## Task fields

| Field | Description | Data type | Allowed methods |
| --- | --- | --- | --- |
| `id` | Task identifier | UUID | GET |
| `workspace_id` | Tenant/workspace identifier | UUID | GET |
| `dataset_id` | Target dataset ID | UUID | GET |
| `asset_id` | Target asset ID | UUID | GET, POST |
| `task_type` | Task operation type | enum(`FULL_ANALYSIS`) | GET, POST |
| `status` | Current state | enum(`QUEUED`,`RUNNING`,`SUCCEEDED`,`FAILED`,`CANCELED`) | GET |
| `configs` | Task configuration | JSON Object | GET, POST |
| `correlation_id` | Trace ID for logs | UUID | string | GET |
| `created_at` | Creation time | timestamp | GET |
| `created_by` | Requester user ID | UUID | string | GET |
| `created_by_name` | Requester display name | string | GET |
| `updated_at` | Last update time | timestamp | GET |

## Create task

`POST /tasks`

Create a new task

### Request

```json
{
  "asset_id": "1b2c3d4e-0001-4a11-8b9a-001122334455",
  "task_type": "FULL_ANALYSIS",
  "configs": {
    "mode": "AUTO",
    "profiling": { "sampling": "SAMPLE", "sample_rows": 500000, "seed": 42 },
    "generate": [
      "DESCRIPTION",
      "SCHEMA",
      "PROFILE",
      "DQ_BASELINE",
      "FILE_PATTERN",
      "GRAIN_HINTS"
    ]
  }
}
```

### Response `201`

```json
{
  "data": {
    "id": "3f2a1c0e-0001-4b2c-9a10-111111111111",
    "workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10",
    "dataset_id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "asset_id": "1b2c3d4e-0001-4a11-8b9a-001122334455",
    "task_type": "FULL_ANALYSIS",
    "status": "QUEUED",
    "configs": {
      "mode": "AUTO",
      "profiling": { "sampling": "SAMPLE", "sample_rows": 500000, "seed": 42 },
      "generate": [
        "DESCRIPTION",
        "SCHEMA",
        "PROFILE",
        "DQ_BASELINE",
        "FILE_PATTERN",
        "GRAIN_HINTS"
      ]
    },
    "created_by": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
    "created_by_name": "John Doe",
    "created_at": "2025-03-18T14:21:00Z",
    "updated_at": "2025-03-18T14:21:00Z"
  }
}
```

## List tasks

`GET /tasks?page=1&page_size=20&asset_id={asset_id}&status=QUEUED&task_type=FULL_ANALYSIS`

Query list of tasks, support search by asset id, status, display name and task type

### Response `200`

```json
{
  "data": [
    {
      "id": "3f2a1c0e-0001-4b2c-9a10-111111111111",
      "workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10",
      "dataset_id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
      "asset_id": "1b2c3d4e-0001-4a11-8b9a-001122334455",
      "task_type": "FULL_ANALYSIS",
      "status": "QUEUED",
      "configs": {
        "mode": "AUTO",
        "profiling": {
          "sampling": "SAMPLE",
          "sample_rows": 500000,
          "seed": 42
        },
        "generate": [
          "DESCRIPTION",
          "SCHEMA",
          "PROFILE",
          "DQ_BASELINE",
          "FILE_PATTERN",
          "GRAIN_HINTS"
        ]
      },
      "created_by": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
      "created_by_name": "John Doe",
      "created_at": "2025-03-18T14:21:00Z",
      "updated_at": "2025-03-18T14:21:00Z"
    }
  ],
  "page": 1,
  "page_size": 20,
  "total": 1
}
```

## Get task by id

`GET /tasks/{task_id}`

Get task details by id

### Response `200`

```json
{
  "data": {
    "id": "3f2a1c0e-0001-4b2c-9a10-111111111111",
    "workspace_id": "b0a6f0c2-7d1d-4f2d-9c2a-1b3f8b2f2e10",
    "dataset_id": "2f1a7e58-0a45-4a2b-9b9e-9f6d0a6cf0a1",
    "asset_id": "1b2c3d4e-0001-4a11-8b9a-001122334455",
    "task_type": "FULL_ANALYSIS",
    "status": "QUEUED",
    "configs": {
      "mode": "AUTO",
      "profiling": { "sampling": "SAMPLE", "sample_rows": 500000, "seed": 42 },
      "generate": [
        "DESCRIPTION",
        "SCHEMA",
        "PROFILE",
        "DQ_BASELINE",
        "FILE_PATTERN",
        "GRAIN_HINTS"
      ]
    },
    "created_by": "6b0d6e6b-5c0e-4f3a-9f4c-6a1b3d2c4e01",
    "created_by_name": "John Doe",
    "created_at": "2025-03-18T14:21:00Z",
    "updated_at": "2025-03-18T14:21:00Z"
  }
}
```

---

# 4) Task Runs API

## Task Run fields

| Field | Description | Data type | Allowed methods |
| --- | --- | --- | --- |
| `id` | Run identifier | UUID | GET |
| `task_id` | Parent task ID | UUID | GET |
| `attempt` | Attempt number (1..n) | integer | GET |
| `status` | Run status | enum(`RUNNING`,`SUCCEEDED`,`FAILED`,`CANCELED`) | GET |
| `error_message` | Human-readable error summary | string | null | GET |
| `started_at` | Execution start time | timestamp | null | GET |
| `finished_at` | Execution end time | timestamp | null | GET |
| `metrics` | Execution metrics | JSON object | GET |
| `created_at` | Run record creation time | timestamp | GET |

## List task runs by task

`GET /tasks/{task_id}/runs?page=1&page_size=20`

### Response `200`

```json
{
  "data": [
    {
      "id": "7d9a2c1e-0001-4f10-9a01-aaaaaaaa0001",
      "task_id": "3f2a1c0e-0001-4b2c-9a10-111111111111",
      "attempt": 1,
      "status": "SUCCEEDED",
      "error_message": null,
      "started_at": "2025-03-18T14:21:08Z",
      "finished_at": "2025-03-18T14:24:21Z",
      "metrics": {
        "file_count": 3,
        "rows_processed": 2548932,
        "columns_processed": 42,
        "duration_ms": 193000,
        "sampled": true,
        "sample_rows": 500000
      },
      "created_at": "2025-03-18T14:21:07Z"
    }
  ],
  "page": 1,
  "page_size": 20,
  "total": 1
}
```

## Get task run by id

`GET /task-runs/{run_id}`

Get task run detail by id

### Response `200`

```json
{
  "data": {
    "id": "7d9a2c1e-0001-4f10-9a01-aaaaaaaa0001",
    "task_id": "3f2a1c0e-0001-4b2c-9a10-111111111111",
    "attempt": 1,
    "status": "SUCCEEDED",
    "error_message": null,
    "started_at": "2025-03-18T14:21:08Z",
    "finished_at": "2025-03-18T14:24:21Z",
    "metrics": {
      "file_count": 3,
      "rows_processed": 2548932,
      "columns_processed": 42,
      "duration_ms": 193000,
      "sampled": true,
      "sample_rows": 500000
    },
    "created_at": "2025-03-18T14:21:07Z"
  }
}
```
