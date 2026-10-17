---
title: "Platform Architecture Master Index (Map of Content)"
aliases:
  - "Architecture Index"
  - "MOC Architecture"
  - "System Overview"
tags:
  - architecture
  - index
  - moc
  - ai-search
  - rust
  - clean-architecture
  - multi-region
author: "Platform Engineering & Architecture Team"
date_created: 2026-09-25
status: "Living Document"
version: "2.5.0"
---

# 🧭 Platform Architecture Master Index (MOC)

> [!abstract] System Overview
> This directory serves as the **authoritative architecture specification** for the **Rust Clean Architecture** platform. The system is designed to provide **sub-millisecond local compute**, **active-active multi-region geo-redundancy**, **zero-downtime disaster recovery (RTO ≈ 0, RPO < 1s)**, and **complete decoupling between domain logic, persistence drivers, and delivery mechanisms**.

---

## 🤖 AI Search & Semantic Discovery Matrix

This table maps search intents and keywords to the exact architecture document. Use this matrix when querying LLMs or search engines:

| Search Intent / Topic | Primary Keywords | Target Document |
| :--- | :--- | :--- |
| **Tech Stack & Decisions** | Rust, Axum, Leptos, pgEdge, Valkey, Kratos, Redpanda, Garage, Quickwit, Ports | [[01_STACK_OVERVIEW]] |
| **Rust Code Organization** | Clean Architecture, Hexagonal, Ports & Adapters, Domain, Application, Infrastructure, Handler | [[02_CLEAN_ARCHITECTURE_RUST]] |
| **Single-Region Setup** | Simple Architecture, Local K3s, Podman, Traefik, Local Flow, Sequence Diagrams | [[03_SIMPLE_TOPOLOGY]] |
| **Multi-Region Distributed** | Active-Active, Geo-DNS, Anycast, Cross-Region, Bangkok, Singapore, Low Latency | [[04_MULTI_REGION_TOPOLOGY]] |
| **Data Sync & Consistency** | pgEdge, Spock 3.x, Multi-Master, UUIDv7, Thai ICU, Garage CRDT S3, Valkey Cache | [[05_DATA_CONSISTENCY_AND_STORAGE]] |
| **Authentication & IAM** | Ory Kratos, Session Cookies, Self-Service, Argon2id, RBAC, ABAC, `su_user` | [[06_SECURITY_IDENTITY_AUTH]] |
| **Streaming & Logs** | Redpanda, Kafka, Quickwit, OTLP, Tracing, Audit Trails, Inverted Index | [[07_EVENT_STREAMING_OBSERVABILITY]] |
| **Failover & Recovery** | Disaster Recovery, RTO, RPO, Network Partition, Split-Brain, CAP Theorem | [[08_DISASTER_RECOVERY_AND_RESILIENCY]] |
| **Network & Firewall** | Ports, DNS, NodePort, Ingress, NetworkPolicy, Mutual TLS, Secret Fan-out | [[09_NETWORK_PORTS_AND_SECURITY_MATRIX]] |
| **Guides (ลงมือทำ)** | เพิ่มโปรแกรม, เพิ่ม service, แก้ปัญหา, เช็กลิสต์ production, เปลี่ยน schema | `docs/guides/` ([[ADD_A_PROGRAM]], [[ADD_A_SERVICE]], [[TROUBLESHOOTING]], [[PRODUCTION_CHECKLIST]]) |
| **Valkey Read Cache** | อะไรแคช/ไม่แคช, key, TTL, การล้างเมื่อเขียน, ข้อจำกัดหลาย region | [[11_VALKEY_READ_CACHE]] |
| **View Scope & เลือกบริษัท** | HQ เห็นภาพรวม/เลือกบริษัท, สาขาอื่นเห็นเฉพาะสาขาตัวเอง, middleware, หน้าที่ถูกกรอง | [[12_VIEW_SCOPE_COMPANY_SWITCHER]] |
| **Project Structure & Style** | CQRS, Clean UI Frontend, Feature Layout, Tests Layer, Naming Rules, Agent Standards | [[10_PROJECT_STRUCTURE_AND_CODE_STYLE]] |

---

## 📚 Architecture Documentation Chapters

```text
docs/architecture/
├── 00_INDEX.md                                  # 📍 You are here (Master Index & MOC)
├── 01_STACK_OVERVIEW.md                         # 🛠️ Technology Stack & Protocol Matrix
├── 02_CLEAN_ARCHITECTURE_RUST.md               # 🦀 Rust Clean Architecture Codebase Mapping
├── 03_SIMPLE_TOPOLOGY.md                        # 💻 Mode 1: Simple / Single-Region Topology
├── 04_MULTI_REGION_TOPOLOGY.md                  # 🌍 Mode 2: Multi-Region Active-Active Distributed
├── 05_DATA_CONSISTENCY_AND_STORAGE.md           # 💾 pgEdge Spock, UUIDv7, Thai ICU & Garage S3
├── 06_SECURITY_IDENTITY_AUTH.md                 # 🔐 Ory Kratos IAM & Session Verification
├── 07_EVENT_STREAMING_OBSERVABILITY.md          # 📡 Redpanda Streaming & Quickwit Logging
├── 08_DISASTER_RECOVERY_AND_RESILIENCY.md       # 🛡️ High Availability, DR & Partition Tolerance
├── 09_NETWORK_PORTS_AND_SECURITY_MATRIX.md      # 🔌 Network, Port, & Security Matrix
├── 10_PROJECT_STRUCTURE_AND_CODE_STYLE.md       # 📐 CQRS, Clean UI Pattern & Agent Code Style Guide
└── 11_VALKEY_READ_CACHE.md                      # ⚡ Read-through cache + invalidation on write
```

---

## 🗺️ Reading Paths by Role

- **Frontend Engineer**: Start with [[01_STACK_OVERVIEW]] -> [[03_SIMPLE_TOPOLOGY]] -> [[06_SECURITY_IDENTITY_AUTH]] (Focus on Leptos, shadcn/ui, Kratos CSR flows).
- **Backend Engineer**: Start with [[02_CLEAN_ARCHITECTURE_RUST]] -> [[03_SIMPLE_TOPOLOGY]] -> [[05_DATA_CONSISTENCY_AND_STORAGE]] -> [[11_VALKEY_READ_CACHE]] (Focus on Axum, SeaORM, Valkey cache-aside, Clean Architecture boundaries).
- **DevOps / Platform Engineer**: Start with [[04_MULTI_REGION_TOPOLOGY]] -> [[08_DISASTER_RECOVERY_AND_RESILIENCY]] -> [[09_NETWORK_PORTS_AND_SECURITY_MATRIX]] (Focus on K3s, k3d, Spock, Garage, Anycast).
- **Security & Compliance Auditor**: Start with [[06_SECURITY_IDENTITY_AUTH]] -> [[07_EVENT_STREAMING_OBSERVABILITY]] (Focus on Ory Kratos session flow, Argon2id, and Redpanda-to-Quickwit immutable audit trails).

---

## ⚡ Quick Links & Obsidian Graph

- [[01_STACK_OVERVIEW|Chapter 1: Tech Stack Overview]]
- [[02_CLEAN_ARCHITECTURE_RUST|Chapter 2: Clean Architecture in Rust]]
- [[03_SIMPLE_TOPOLOGY|Chapter 3: Simple Architecture]]
- [[04_MULTI_REGION_TOPOLOGY|Chapter 4: Multi-Region Distributed Architecture]]
- [[05_DATA_CONSISTENCY_AND_STORAGE|Chapter 5: Data Consistency & Storage Strategies]]
- [[06_SECURITY_IDENTITY_AUTH|Chapter 6: Identity, Authentication & Permissions]]
- [[07_EVENT_STREAMING_OBSERVABILITY|Chapter 7: Event Streaming & Observability]]
- [[08_DISASTER_RECOVERY_AND_RESILIENCY|Chapter 8: Disaster Recovery & Resiliency]]
- [[09_NETWORK_PORTS_AND_SECURITY_MATRIX|Chapter 9: Network, Ports & Security Matrix]]
- [[10_PROJECT_STRUCTURE_AND_CODE_STYLE|Chapter 10: Project Structure & Code Style Guide]]
- [[11_VALKEY_READ_CACHE|Chapter 11: Valkey Read Cache & Invalidation]]
- 📑 [[../programs/00_INDEX|Program Catalogue & Screen Specifications MOC]]

