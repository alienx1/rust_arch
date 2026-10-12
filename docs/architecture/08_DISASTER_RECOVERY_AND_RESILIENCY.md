---
title: "Platform Architecture: Disaster Recovery & Resiliency"
aliases:
  - "Disaster Recovery"
  - "Resiliency Specification"
  - "HA and DR Runbooks"
  - "Failure Modes"
tags:
  - architecture
  - disaster-recovery
  - resiliency
  - cap-theorem
  - failover
  - high-availability
author: "Platform Engineering & Architecture Team"
date_created: 2026-09-25
status: "Living Document"
version: "2.5.0"
---

# 🛡️ Chapter 8: Disaster Recovery & Resiliency

> [!info] Navigation
> - ⬅️ Back to: [[07_EVENT_STREAMING_OBSERVABILITY|Event Streaming & Observability]]
> - ➡️ Next: [[09_NETWORK_PORTS_AND_SECURITY_MATRIX|Network, Ports & Security Matrix]]
> - 🧭 [[00_INDEX|Master Index]]

---

## 1. Resilience & Recovery Metrics

```text
┌──────────────────────────────────────────────────────────────────────────────┐
│                    HIGH AVAILABILITY & RECOVERY OBJECTIVES                   │
├────────────────────────────┬─────────────────────────────────────────────────┤
│ Target Availability        │ 99.99% (Four Nines across Multi-Region)         │
│ Recovery Time Obj (RTO)    │ ≈ 0 seconds (Automated Anycast edge divert)     │
│ Recovery Point Obj (RPO)   │ < 1.0 second (Continuous logical replication)   │
│ Partition Tolerance        │ Complete operational autonomy per region        │
└────────────────────────────┴─────────────────────────────────────────────────┘
```

---

## 2. CAP Theorem Trade-Off Analysis

```mermaid
quadrantChart
    title Distributed Systems Trade-off Matrix
    x-axis Low Consistency --> High Consistency
    y-axis Low Availability --> High Availability
    quadrant-1 Traditional 2PC / CockroachDB (CP)
    quadrant-2 pgEdge Spock & Garage S3 (AP)
    quadrant-3 Single Node Databases
    quadrant-4 Distributed Read Replicas
    "pgEdge (Spock)": [0.65, 0.90]
    "Garage S3": [0.60, 0.95]
    "Redpanda (Local)": [0.85, 0.80]
    "Valkey Cache": [0.40, 0.95]
    "Ory Kratos": [0.65, 0.90]
```

### 2.1 The AP Decision for Multi-Region
Traditional distributed SQL databases (CockroachDB, YugabyteDB) prioritize **Consistency over Availability (CP)** using Raft consensus across regions. Under this model, an inter-region network spike delays every single write transaction globally.

Our platform deliberately adopts an **Availability and Partition Tolerance (AP) with Causal Convergence** model:
- Writes never block across regions.
- Relational tables converge via Spock's Last-Update-Wins (LUW) rule and UUIDv7.
- Object storage converges via Garage CRDTs.

---

## 3. Failure Scenarios & Automated Recovery Runbooks

### 3.1 Scenario 1: Pod / Node Crash within a Single Region
- **Failure**: An Axum API node or Valkey container crashes unexpectedly.
- **Automated Reaction**:
  1. Kubernetes liveness/readiness probes detect failure within 5 seconds.
  2. Traefik removes the failed Pod endpoint from ingress rotation.
  3. K8s Deployment controller respawns a replacement container.
  4. Total client impact: 0s (traffic shifts to surviving replicas).

### 3.2 Scenario 2: Complete Region Catastrophic Outage
- **Failure**: Datacenter power loss or cloud region network blackout in Bangkok (`th-central`).
- **Automated Reaction**:
  ```mermaid
  flowchart TD
      Outage["⚡ Region 1 (Bangkok) Offline"] --> HealthProbe["Global Edge Health Checks Fail\n(Interval: 5s, Threshold: 2)"]
      HealthProbe --> EdgeDivert["🌐 Cloudflare / Route 53 Anycast Divert\n100% Traffic Shifted to Region 2 (Singapore)"]
      EdgeDivert --> Reg2Active["🇸🇬 Region 2 Processes Requests\n• Local pgEdge Node 2 accepts writes\n• Local Garage Node 2 accepts uploads\n• Kratos validates users from replicated tables"]
  ```
- **Operator Action Required**: None for immediate failover. When Bangkok recovers, Spock automatically drains backlogged changes.

### 3.3 Scenario 3: Inter-Region Submarine Fiber Cut (Network Partition)
- **Failure**: The dedicated WAN connection between Bangkok and Singapore is severed. Both regions can reach global users, but cannot reach each other.
- **Behavior During Split**:
  - Both regions continue operating as standalone active clusters.
  - Thai users write to Bangkok; Singapore/APAC users write to Singapore.
  - **UUIDv7 guarantees zero primary key collisions** during the split.
- **Automated Healing on Link Restoration**:
  1. Spock identifies peer node reconnection and streams buffered logical changes.
  2. Rows modified concurrently in both regions resolve via commit timestamp.
  3. Garage reconciles missing S3 blocks via Merkle tree exchanges.
  4. Redpanda mirrors queued cross-region topics.

---

## 4. Disaster Recovery Drill Schedule

To guarantee production readiness:

| Exercise | Frequency | Procedure | Success Criteria |
| :--- | :--- | :--- | :--- |
| **Chaos Pod Kill** | Weekly | Randomly terminate 50% of application pods via Chaos Mesh. | Zero 5xx errors; sub-100ms recovery. |
| **Simulated Region Severance** | Monthly | Introduce firewall drop rules on WAN port 5432/3902 for 30 minutes. | Both regions accept writes; clean auto-reconciliation. |
| **Point-in-Time Recovery (PITR)** | Quarterly | Restore PostgreSQL to $T-2$ hours on a clean staging cluster. | Zero data corruption; checksum verification. |

---

## 5. Cross-Document References

- Review multi-region network design: [[04_MULTI_REGION_TOPOLOGY]]
- Review data consistency models: [[05_DATA_CONSISTENCY_AND_STORAGE]]
- Review network firewall matrix: [[09_NETWORK_PORTS_AND_SECURITY_MATRIX]]
