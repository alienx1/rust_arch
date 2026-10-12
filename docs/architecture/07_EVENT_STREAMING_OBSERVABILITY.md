---
title: "Platform Architecture: Event Streaming & Observability"
aliases:
  - "Streaming Architecture"
  - "Observability Specification"
  - "Redpanda and Quickwit"
tags:
  - architecture
  - streaming
  - redpanda
  - quickwit
  - opentelemetry
  - tracing
  - kafka
author: "Platform Engineering & Architecture Team"
date_created: 2026-09-25
status: "Living Document"
version: "2.5.0"
---

# 📡 Chapter 7: Event Streaming & Observability

> [!info] Navigation
> - ⬅️ Back to: [[06_SECURITY_IDENTITY_AUTH|Identity, Authentication & Permissions]]
> - ➡️ Next: [[08_DISASTER_RECOVERY_AND_RESILIENCY|Disaster Recovery & Resiliency]]
> - 🧭 [[00_INDEX|Master Index]]

---

## 1. Streaming & Observability Architecture

```mermaid
flowchart TD
    classDef app fill:#eef9f0,stroke:#2d6a4f,stroke-width:2px;
    classDef stream fill:#fff4e6,stroke:#b07d62,stroke-width:2px;
    classDef obs fill:#f8f9fa,stroke:#495057,stroke-width:2px;
    classDef s3 fill:#e8f4f8,stroke:#1b4965,stroke-width:2px;

    subgraph AppCompute ["⚙️ Axum Application Layer"]
        AxumMaster["api-master :8000\n• Audit Event Producer\n• Report Request Producer"]:::app
        AxumReport["api-report :8001\n• Report Worker (Consumer)\n• Notification Producer"]:::app
        AxumNoti["api-notification :8002\n• Notification Ingest (Consumer)\n• Realtime SSE Hub"]:::app
    end

    subgraph StreamLayer ["🔴 Redpanda Streaming Pipeline"]
        direction TB
        Broker["Redpanda Broker\n(Port 9092)"]:::stream
        TopicReport["Topic: report.requests"]:::stream
        TopicNoti["Topic: notifications"]:::stream
        TopicAudit["Topic: audit.events"]:::stream
        TopicCDC["Topic: cdc.master"]:::stream
        Console["Redpanda Console UI\n(Port 8080)"]:::stream
        Broker --- TopicReport
        Broker --- TopicNoti
        Broker --- TopicAudit
        Broker --- TopicCDC
        Broker <--> Console
    end

    subgraph ObservabilityLayer ["🔎 Quickwit Search & Tracing Engine"]
        direction TB
        QWIndexer["Quickwit Native Kafka Ingest & Indexer"]:::obs
        QWSearcher["Quickwit Search Node\n(Port 7280 REST / UI)"]:::obs
        OTLPReceiver["OTLP gRPC Collector\n(Port 7281)"]:::obs
    end

    subgraph StorageLayer ["🪣 Garage S3 Storage"]
        S3Bucket[("rust-arch-bucket\n• /reports/*.csv\n• /indexes/split_*.qws")]:::s3
    end

    %% Pipeline flows
    AxumMaster -->|"1. Publish report.requests / audit"| Broker
    AxumReport <-->|"2. Consume report.requests / Publish noti"| Broker
    AxumReport -->|"3. Multipart Upload CSV"| S3Bucket
    AxumNoti <-->|"4. Consume notifications"| Broker

    AppCompute -.->|"OTLP Logs & Traces (gRPC)"| OTLPReceiver

    TopicAudit -->|"Native Kafka Consumer Ingest"| QWIndexer
    TopicReport -->|"Native Kafka Ingest (events-report-requests-v1)"| QWIndexer
    TopicNoti -->|"Native Kafka Ingest (events-notifications-v1)"| QWIndexer
    OTLPReceiver --> QWIndexer

    QWIndexer -->|"Write Immutable Index Splits"| S3Bucket
    QWSearcher -->|"Zero-Copy Range Requests"| S3Bucket
```

---

## 2. Redpanda Event Streaming

### 2.1 The Thread-Per-Core Advantage
Unlike traditional Apache Kafka implementations running on the Java Virtual Machine (JVM), Redpanda is engineered in C++ using the Seastar async framework:
- **Zero Garbage Collection (GC) Pauses**: Guarantees consistent p99 latencies (< 5ms) under heavy write load.
- **Direct I/O**: Bypasses the Linux page cache, writing directly to disk with kernel `io_uring`.
- **Built-in Raft Consensus**: Removes ZooKeeper/KRaft dependencies.

### 2.2 Standard Topic Topologies
1. `report.requests`:
   - Carries asynchronous report generation jobs (`ReportJobMessage`). Keyed by `job_id`.
   - Dispatched by `api-master` (e.g. ALPG000 CSV export) and consumed by `api-report` (`report-worker` group).
   - Indexed continuously into Quickwit via `events-report-requests-v1`.
2. `notifications`:
   - System and user alerts (`Notification` payload). Keyed by recipient `username`.
   - Dispatched by services (e.g. `api-report` on report completion) and consumed by `api-notification`.
   - Indexed continuously into Quickwit via `events-notifications-v1`.
3. `audit.events`:
   - Stores immutable audit logs of all business actions (who, what, when, old values, new values).
   - Retained for 30 days in Redpanda; persisted indefinitely in Quickwit/S3.
4. `cdc.master`:
   - Captures database change events for cross-region cache eviction in Valkey.

---

## 3. Quickwit Search Engine & Storage Decoupling

### 3.1 Decoupled Indexing & Querying
Quickwit replaces resource-heavy Elasticsearch/OpenSearch clusters:
- **Stateless Search Nodes**: Quickwit searchers do not hold local replicas. They query immutable index splits directly from Garage S3 using HTTP Range requests.
- **Index Split Format (`.qws`)**: Columnar data format with embedded Tantivy inverted indexes, bitmaps, and Fast Field columnar caches.

### 3.2 Ingestion Pipelines & Event Indexing
1. **Streaming Kafka Sources (Audit & Events)**:
   Quickwit polls Redpanda topics natively as a consumer group, batches messages, and creates immutable index files directly on S3:
   - `events-report-requests-v1` ➔ Topic `report.requests` (timestamp: `requested_at`)
   - `events-notifications-v1` ➔ Topic `notifications` (timestamp: `created_at`)
   - Config manifests in `kube/arch/applications/arch_logging/indexes/`, provisioned via `make topics` / `make event-indexes`.
2. **Direct OTLP Pipeline (Application Traces & Logs)**:
   Axum services push OpenTelemetry spans and log events directly over gRPC to Quickwit's OTLP receiver on port `7281`.

---

## 4. Rust Tracing Instrumentation

In the Rust codebase, tracing is configured to multiplex between local stdout and remote OTLP export:

```rust
// infrastructure/src/services/logging.rs
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use opentelemetry_otlp::WithExportConfig;

pub fn init_observability() {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://quickwit:7281"),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Failed to initialize OTLP tracer");

    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(telemetry_layer)
        .init();
}
```

---

## 5. Cross-Document References

- Review tech stack options: [[01_STACK_OVERVIEW]]
- Review multi-region data replication: [[05_DATA_CONSISTENCY_AND_STORAGE]]
- Review disaster recovery procedures: [[08_DISASTER_RECOVERY_AND_RESILIENCY]]
