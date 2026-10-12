---
title: "Platform Architecture: Simple Single-Region Topology"
aliases:
  - "Simple Architecture"
  - "Single-Region Architecture"
  - "Local Development Topology"
tags:
  - architecture
  - topology
  - single-region
  - k3d
  - kubernetes
author: "Platform Engineering & Architecture Team"
date_created: 2026-09-25
status: "Living Document"
version: "2.5.0"
---

# 💻 Chapter 3: Mode 1 - Simple Single-Region Topology

> [!info] Navigation
> - ⬅️ Back to: [[02_CLEAN_ARCHITECTURE_RUST|Clean Architecture in Rust]]
> - ➡️ Next: [[04_MULTI_REGION_TOPOLOGY|Multi-Region Distributed Architecture]]
> - 🧭 [[00_INDEX|Master Index]]

---

## 1. Overview & Objectives

The **Simple Architecture** provides a self-contained, low-overhead environment running on a single Kubernetes cluster (or locally via `k3d` on rootless Podman/Docker). 

It is ideal for:
- Local development and testing.
- Single-VPC production deployments with low-to-medium traffic.
- Continuous Integration (CI) and automated end-to-end testing pipelines.

---

## 2. Architecture Diagram

```mermaid
flowchart TD
    classDef client fill:#f9f0ff,stroke:#702963,stroke-width:2px;
    classDef edge fill:#e8f4f8,stroke:#1b4965,stroke-width:2px;
    classDef app fill:#eef9f0,stroke:#2d6a4f,stroke-width:2px;
    classDef data fill:#fff4e6,stroke:#b07d62,stroke-width:2px;
    classDef obs fill:#f8f9fa,stroke:#495057,stroke-width:2px;

    Client["💻 Client Browser\n(Rust Leptos WASM SPA + shadcn/ui)"]:::client

    subgraph EdgeTier ["🚪 Ingress & Gateway Layer"]
        Traefik["Traefik / Ingress Gateway\n(Ports: 80 / 443 / 8080)"]:::edge
    end

    subgraph AppTier ["⚙️ Application Compute Tier (Rust Axum)"]
        direction TB
        AuthMW["Kratos Session Middleware\n(axum-extra: Cookie / Bearer)"]:::app
        APIMaster["api-master :8000\n(Master Data Engine)"]:::app
        APIReport["api-report :8001\n(Report Worker & Storage)"]:::app
        APINotification["api-notification :8002\n(Realtime SSE & Alerts)"]:::app
    end

    subgraph IdentityTier ["🔐 Identity & Access Management"]
        Kratos["Ory Kratos 1.3\n(Public: 4433 | Admin: 4434)"]:::edge
    end

    subgraph DataTier ["💾 Persistence, Caching & Streaming"]
        direction TB
        Postgres[("pgEdge / PostgreSQL 17\n• UUIDv7 Native\n• ICU th-TH Collation\n(Port: 5432)")]:::data
        Valkey[("Valkey 8 Cache\n• In-Memory RESP3\n(Port: 6379)")]:::data
        Garage[("Garage S3 Storage\n• Local Chunks & Metadata\n(Port: 3900)")]:::data
        Redpanda[("Redpanda Broker\n• Topic: report.requests\n• Topic: notifications\n• Topic: audit.events\n(Port: 9092)")]:::data
    end

    subgraph ObsTier ["📊 Observability & Auditing"]
        direction TB
        Quickwit[("Quickwit Search Engine\n• REST: 7280\n• OTLP gRPC: 7281")]:::obs
        Console["Redpanda Console UI\n(Port: 8080)"]:::obs
    end

    Client -->|"1. Load WebAssembly SPA"| Traefik
    Client -->|"2. Self-Service Login / Register"| Kratos
    Client -->|"3. Authorized API Requests"| Traefik
    Client <-.->|"4. Real-time SSE Stream"| APINotification

    Traefik --> AuthMW
    AuthMW -->|"Validate Session"| Kratos
    AuthMW --> APIMaster
    AuthMW --> APIReport
    AuthMW --> APINotification

    APIMaster -->|"Read / Write Entities"| Postgres
    APIMaster -->|"Cache Aside & Invalidate"| Valkey
    APIMaster -->|"Queue Export Requests"| Redpanda

    APIReport <-->|"Consume Requests / Publish Noti"| Redpanda
    APIReport -->|"Store CSV / Typst Exports"| Garage
    APIReport -->|"Update Job Status (rp_report_job)"| Postgres

    APINotification <-->|"Consume Notifications"| Redpanda
    APINotification -->|"Persist Alerts (nt_notification)"| Postgres

    Redpanda -->|"Native Kafka Indexing"| Quickwit
    APIMaster -.->|"Direct OTLP Traces / Logs"| Quickwit
    Console <--> Redpanda
    Quickwit -.->|"Store Index Splits"| Garage
```

---

## 3. Communication Protocols & Port Mappings

All services in the simple local architecture are bound to fixed NodePorts and exposed directly to `localhost` through the `k3d` direct server binding:

| Service | Internal DNS | NodePort | Host Port (`localhost`) | Function |
| :--- | :--- | :--- | :--- | :--- |
| **Axum API Master** | `api-master.arch.svc:8000` | - | `8000` | Master data, IAM, and menu API |
| **Axum API Report** | `api-report.arch.svc:8001` | - | `8001` | Background report worker & signed download API |
| **Axum API Notification**| `api-notification.arch.svc:8002`| - | `8002` | SSE push streaming & notification API |
| **PostgreSQL 17** | `pgedge.arch.svc:5432` | `30432` | `5432` | Relational storage (UUIDv7, ICU th-TH) |
| **Valkey 8** | `valkey.arch.svc:6379` | `30379` | `6379` | In-memory key-value cache |
| **Redpanda Kafka** | `redpanda.arch.svc:9092` | `30092` | `9092` | Event streaming broker |
| **Redpanda Admin** | `redpanda.arch.svc:9644` | `30644` | `9644` | Broker administration API |
| **Redpanda Console**| `redpanda-console.arch.svc:8080`| `30080` | `8080` | Web UI for topics, consumer groups |
| **Ory Kratos Public**| `kratos.arch.svc:4433` | `30433` | `4433` | User self-service login/signup API |
| **Ory Kratos Admin** | `kratos.arch.svc:4434` | `30434` | `4434` | Internal identity management API |
| **Garage S3** | `garage.arch.svc:3900` | `30900` | `3900` | S3-compatible object storage API |
| **Garage Admin** | `garage.arch.svc:3903` | `30903` | `3903` | Node & layout admin API |
| **Quickwit REST** | `quickwit.arch.svc:7280` | `30280` | `7280` | Search/Ingest REST & Web UI |
| **Quickwit OTLP** | `quickwit.arch.svc:7281` | `30281` | `7281` | OpenTelemetry gRPC receiver |

---

## 4. Workflows & Sequence Diagrams

### 4.1 Master Data Read Workflow (Cache-Aside Pattern)

```mermaid
sequenceDiagram
    autonumber
    actor User as 👤 User
    participant App as 💻 Leptos SPA
    participant Axum as ⚙️ Axum (api-master)
    participant Cache as ⚡ Valkey (Cache)
    participant DB as 💾 pgEdge (Postgres)

    User->>App: Request Province List
    App->>Axum: GET /api/v1/master/provinces
    
    Axum->>Cache: GET master:provinces:th
    alt Cache Hit
        Cache-->>Axum: Cached JSON Payload
    else Cache Miss
        Axum->>DB: SELECT * FROM db_province ORDER BY code
        DB-->>Axum: Database Rows
        Axum->>Cache: SETEX master:provinces:th 3600 <JSON>
    end
    
    Axum-->>App: 200 OK (Province Array)
    App-->>User: Render Province Table
```

### 4.2 Report Generation Workflow

```mermaid
sequenceDiagram
    autonumber
    actor User as 👤 User
    participant App as 💻 Leptos SPA
    participant Axum as ⚙️ Axum (api-report)
    participant DB as 💾 pgEdge
    participant Typst as 📄 Typst Engine
    participant S3 as 🪣 Garage S3

    User->>App: Click "Generate Monthly Report"
    App->>Axum: POST /api/v1/reports/monthly { org_id, month }
    Axum->>DB: Query Aggregated Transactions
    DB-->>Axum: Report Dataset
    
    Note over Axum,Typst: Compile Typst template<br/>into vector PDF bytes
    Axum->>Typst: Render PDF
    Typst-->>Axum: PDF Bytes (In-Memory)
    
    Axum->>S3: PUT /rust-arch-bucket/reports/report_2026_09.pdf
    S3-->>Axum: Upload OK
    
    Axum-->>App: 200 OK { download_url: "/s3/..." }
    App-->>User: Initiate File Download
```

### 4.3 Asynchronous Report Generation & Realtime Notification Push

```mermaid
sequenceDiagram
    autonumber
    actor User as 👤 User
    participant App as 💻 Leptos SPA
    participant Master as ⚙️ Axum (api-master)
    participant RP as 🔴 Redpanda (Kafka)
    participant ReportWorker as ⚙️ Axum (api-report)
    participant S3 as 🪣 Garage S3
    participant DB as 💾 PostgreSQL (pgEdge)
    participant NotiService as 🔔 Axum (api-notification)

    User->>App: Request CSV Export (ALPG000)
    App->>Master: POST /api/v1/al/alpg000/export { filter }
    Master->>RP: Publish ReportJobMessage to topic `report.requests`
    Master-->>App: 202 Accepted { job_id }
    
    RP->>ReportWorker: Consume from `report.requests`
    ReportWorker->>DB: INSERT INTO rp_report_job (status: 'running')
    ReportWorker->>S3: Stream CSV chunks via CsvObjectWriter
    ReportWorker->>DB: UPDATE rp_report_job (status: 'succeeded', row_count, size_bytes)
    ReportWorker->>RP: Publish Notification to topic `notifications`
    
    RP->>NotiService: Consume from `notifications` (recipient: username)
    NotiService->>DB: INSERT INTO nt_notification
    NotiService-->>App: Push SSE Event `notification`
    App-->>User: Show Bell Counter & Desktop Alert
    
    User->>App: Click Notification (Navigates to /rp/rppg000)
    App->>ReportWorker: GET /api/v1/reports/jobs/{id}/download
    ReportWorker-->>App: Pre-signed Garage S3 URL
    App-->>User: Trigger Browser File Download
```

---

## 5. Local Operations Quickstart

To run this architecture locally:

```bash
# 1. Start cluster and bootstrap all services
make bootstrap

# 2. Check pod and service status
make status

# 3. Open Kubernetes dashboard TUI
make k9s

# 4. Stream service logs
make logs APP=arch-database

# 5. Stop cluster preserving data
make stop
```

---

## 6. Cross-Document References

- Review multi-region distributed design: [[04_MULTI_REGION_TOPOLOGY]]
- Examine data consistency models: [[05_DATA_CONSISTENCY_AND_STORAGE]]
- Review identity management: [[06_SECURITY_IDENTITY_AUTH]]
