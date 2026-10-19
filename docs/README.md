# Documentation

This directory contains technical documentation and architecture specifications for the platform.

## Guides (Hands-On)

- **Start Here**: [`README.md`](../README.md) (at repo root)
- **Add a New Program / Screen**: [`guides/ADD_A_PROGRAM.md`](./guides/ADD_A_PROGRAM.md)
- **Add a New Service**: [`guides/ADD_A_SERVICE.md`](./guides/ADD_A_SERVICE.md)
- **Troubleshooting Common Issues**: [`guides/TROUBLESHOOTING.md`](./guides/TROUBLESHOOTING.md)
- **Production Readiness Checklist**: [`guides/PRODUCTION_CHECKLIST.md`](./guides/PRODUCTION_CHECKLIST.md)
- **Database Schema Changes**: [`../sql/README.md`](../sql/README.md)

## Architecture Specifications (Obsidian-ready)

All architectural documentation is organized modularly under [`docs/architecture/`](./architecture/):

- **Master Index & AI Search Matrix**: [`docs/architecture/00_INDEX.md`](./architecture/00_INDEX.md)
- **Tech Stack Overview**: [`docs/architecture/01_STACK_OVERVIEW.md`](./architecture/01_STACK_OVERVIEW.md)
- **Rust Clean Architecture**: [`docs/architecture/02_CLEAN_ARCHITECTURE_RUST.md`](./architecture/02_CLEAN_ARCHITECTURE_RUST.md)
- **Simple Single-Region Topology**: [`docs/architecture/03_SIMPLE_TOPOLOGY.md`](./architecture/03_SIMPLE_TOPOLOGY.md)
- **Multi-Region Active-Active Topology**: [`docs/architecture/04_MULTI_REGION_TOPOLOGY.md`](./architecture/04_MULTI_REGION_TOPOLOGY.md)
- **Data Consistency & Storage**: [`docs/architecture/05_DATA_CONSISTENCY_AND_STORAGE.md`](./architecture/05_DATA_CONSISTENCY_AND_STORAGE.md)
- **Identity, Auth & Permissions**: [`docs/architecture/06_SECURITY_IDENTITY_AUTH.md`](./architecture/06_SECURITY_IDENTITY_AUTH.md)
- **Event Streaming & Observability**: [`docs/architecture/07_EVENT_STREAMING_OBSERVABILITY.md`](./architecture/07_EVENT_STREAMING_OBSERVABILITY.md)
- **Disaster Recovery & Resiliency**: [`docs/architecture/08_DISASTER_RECOVERY_AND_RESILIENCY.md`](./architecture/08_DISASTER_RECOVERY_AND_RESILIENCY.md)
- **Network, Ports & Security Matrix**: [`docs/architecture/09_NETWORK_PORTS_AND_SECURITY_MATRIX.md`](./architecture/09_NETWORK_PORTS_AND_SECURITY_MATRIX.md)
- **Project Structure & Code Style Guide**: [`docs/architecture/10_PROJECT_STRUCTURE_AND_CODE_STYLE.md`](./architecture/10_PROJECT_STRUCTURE_AND_CODE_STYLE.md)
- **Valkey Read Cache & Invalidation**: [`docs/architecture/11_VALKEY_READ_CACHE.md`](./architecture/11_VALKEY_READ_CACHE.md)
- **View Scope & Company/Branch Switcher**: [`docs/architecture/12_VIEW_SCOPE_COMPANY_SWITCHER.md`](./architecture/12_VIEW_SCOPE_COMPANY_SWITCHER.md)
- **AI Agent Engineering Standards**: [`AGENTS.md`](../AGENTS.md)

## Program & Screen Specifications (Obsidian-ready)

All screen specifications and business scope templates are organized under [`docs/programs/`](./programs/):

- **Program Catalogue MOC**: [`docs/programs/00_INDEX.md`](./programs/00_INDEX.md)
- **SYPG000 (Dashboard)**: [`docs/programs/SYPG000_DASHBOARD.md`](./programs/SYPG000_DASHBOARD.md)
- **DBPG000 (Organizations & Companies Main)**: [`docs/programs/DBPG000_ORGANIZATIONS_MAIN.md`](./programs/DBPG000_ORGANIZATIONS_MAIN.md)
- **DBPG001 (Org & Company Details: /ORG, /COMPANY)**: [`docs/programs/DBPG001_ORGANIZATIONS.md`](./programs/DBPG001_ORGANIZATIONS.md)
- **DBPG002 (Branches & Depts)**: [`docs/programs/DBPG002_BRANCHES_DEPARTMENTS.md`](./programs/DBPG002_BRANCHES_DEPARTMENTS.md)
- **DBPG003 (Employees)**: [`docs/programs/DBPG003_EMPLOYEES.md`](./programs/DBPG003_EMPLOYEES.md)
- **DBPG004 (Reference Data)**: [`docs/programs/DBPG004_REFERENCE_DATA.md`](./programs/DBPG004_REFERENCE_DATA.md)
- **DBPG005 (Geography)**: [`docs/programs/DBPG005_GEOGRAPHY.md`](./programs/DBPG005_GEOGRAPHY.md)
- **DBPG006 (i18n Labels)**: [`docs/programs/DBPG006_I18N_LABELS.md`](./programs/DBPG006_I18N_LABELS.md)
- **SUPG001 (Systems & Modules)**: [`docs/programs/SUPG001_SYSTEMS_MODULES.md`](./programs/SUPG001_SYSTEMS_MODULES.md)
- **SUPG002 (Menus & Programs)**: [`docs/programs/SUPG002_MENUS_PROGRAMS.md`](./programs/SUPG002_MENUS_PROGRAMS.md)
- **SUPG003 (Permission Matrix)**: [`docs/programs/SUPG003_PERMISSION_MATRIX.md`](./programs/SUPG003_PERMISSION_MATRIX.md)
- **SUPG004 (User Accounts)**: [`docs/programs/SUPG004_USER_ACCOUNTS.md`](./programs/SUPG004_USER_ACCOUNTS.md)
- **SUPG005 (IAM & Session Monitor)**: [`docs/programs/SUPG005_IAM_SESSION.md`](./programs/SUPG005_IAM_SESSION.md)
- **ALPG000 (Audit Log)**: [`docs/programs/ALPG000_AUDIT_LOG.md`](./programs/ALPG000_AUDIT_LOG.md)
- **ALPG001 (Process Monitor)**: [`docs/programs/ALPG001_PROCESS_MONITOR.md`](./programs/ALPG001_PROCESS_MONITOR.md)
- **RPPG000 (My Reports)**: [`docs/programs/RPPG000_MY_REPORTS.md`](./programs/RPPG000_MY_REPORTS.md)
- **NTPG000 (Notifications)**: [`docs/programs/NTPG000_NOTIFICATIONS.md`](./programs/NTPG000_NOTIFICATIONS.md)

