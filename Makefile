.PHONY: help bootstrap up down stop status k9s lint logs deploy undeploy check build seed run-master master run-report report run-notification notification topics run-web web dev-master dev-report gen-entities gen-entity

.DEFAULT_GOAL := help

# Environment overlay to deploy: local (default), staging, production.
# Usage: make deploy ENV=staging
ENV ?= local
OVERLAY := kube/arch/overlays/$(ENV)

ifeq ($(ENV),local)
NS := arch
else
NS := arch-$(ENV)
endif

KUSTOMIZE_BUILD := kustomize build $(OVERLAY) --load-restrictor LoadRestrictionsNone

help: ## Show this help message
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

# --- Infrastructure & Cluster Targets ---

bootstrap: ## Bring up EVERYTHING: cluster + all apps + sql seeds
	@ENV=$(ENV) ./kube/arch/utils/bootstrap.sh

seed: ## Run sql/*.sql migrations and seed data into pgEdge / PostgreSQL (then flush the read cache)
	@ENV=$(ENV) ./kube/arch/utils/db-seed.sh
	@$(MAKE) -s cache-flush

cache-flush: ## Drop the API's Valkey read cache (arch:* keys); needed after changing data outside the API
	@n=$$(kubectl -n $(NS) exec deploy/arch-cache -- sh -c 'valkey-cli --scan --pattern "arch:*" | xargs -r valkey-cli unlink' 2>/dev/null | tail -1); \
	  echo "🧹 Read cache flushed (keys removed: $${n:-0})"

seed-admin: ## Seed default Ory Kratos admin identity (admin@arch.local / Admin@2026)
	@code=$$(curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:4434/admin/identities \
	  -H "Content-Type: application/json" \
	  -d '{"schema_id":"default","traits":{"email":"admin@arch.local","username":"admin","name":{"first":"Admin","last":"System"}},"credentials":{"password":{"config":{"password":"Admin@2026"}}}}'); \
	case "$$code" in \
	  201) echo "✅ Default admin identity seeded: admin@arch.local / Admin@2026" ;; \
	  409) echo "ℹ️  Admin already exists" ;; \
	  000) echo "❌ Kratos unreachable at localhost:4434" ; exit 1 ;; \
	  *)   echo "❌ Seeding admin failed (HTTP $$code)" ; exit 1 ;; \
	esac

seed-test-users: ## Seed ABAC test identity in Ory Kratos (test_abac / test_abac@arch.local, password Test@2026)
	@curl -s -o /dev/null -w "%{http_code}" -X POST http://localhost:4434/admin/identities \
	  -H "Content-Type: application/json" \
	  -d '{"schema_id":"default","traits":{"email":"test_abac@arch.local","username":"test_abac","name":{"first":"ทดสอบ","last":"เอแบค"}},"credentials":{"password":{"config":{"password":"Test@2026"}}}}' \
	  | grep -q '^201$$' && echo "✅ Test identity seeded: test_abac / Test@2026" || echo "ℹ️  test_abac already exists or Kratos unreachable"

seed-usernames: ## Backfill the `username` trait (email local-part) on Kratos identities that lack it
	@curl -s http://localhost:4434/admin/identities?page_size=250 | python3 -c 'import sys,json,urllib.request as u; \
	[ (lambda i: print("username", i["traits"]["email"].split("@")[0], "->", u.urlopen(u.Request("http://localhost:4434/admin/identities/"+i["id"], data=json.dumps([{"op":"add","path":"/traits/username","value":i["traits"]["email"].split("@")[0]}]).encode(), headers={"Content-Type":"application/json"}, method="PATCH")).status))(i) \
	  for i in json.load(sys.stdin) if "username" not in i["traits"] and "email" in i["traits"] ]'

up: ## Start Podman socket & spin up K3s cluster only
	@./kube/arch/utils/k3s-up.sh

down: ## Delete K3s cluster -- DESTROYS all PV data (use 'make stop' to keep it)
	@./kube/arch/utils/k3s-down.sh

stop: ## Stop the cluster without deleting it (PV data preserved)
	@./kube/arch/utils/k3s-stop.sh

status: ## View cluster and node status
	@./kube/arch/utils/k3s-status.sh

k9s: ## Open Kubernetes TUI dashboard
	@k9s

lint: ## Validate rendered manifests with kubeconform (usage: make lint ENV=local)
	@echo "🔍 Validating Kubernetes manifests for ENV=$(ENV)..."
	@$(KUSTOMIZE_BUILD) | kubeconform -summary -strict -ignore-missing-schemas

logs: ## Stream logs from pod (usage: make logs APP=arch-database)
	@stern -n $(NS) $(APP)

deploy: ## Apply the Arch stack for ENV (default local; usage: make deploy ENV=staging)
	@echo "🚀 Deploying ENV=$(ENV) from $(OVERLAY)..."
	@$(KUSTOMIZE_BUILD) | kubectl apply -f -

undeploy: ## Remove the Arch stack for ENV (usage: make undeploy ENV=staging)
	@$(KUSTOMIZE_BUILD) | kubectl delete -f -

# --- Development & Application Run Targets ---

run-master: ## Run api-master service (Port 8000)
	@echo "🚀 Starting api-master on port 8000..."
	@cargo run -p api-master

master: run-master ## Alias for run-master

run-report: ## Run api-report service (Port 8001)
	@echo "🚀 Starting api-report on port 8001..."
	@cargo run -p api-report

report: run-report ## Alias for run-report

run-notification: ## Run api-notification service (Port 8002)
	@echo "🚀 Starting api-notification on port 8002..."
	@cargo run -p api-notification

notification: run-notification ## Alias for run-notification

topics: ## Create the Redpanda topics the services use (idempotent)
	@for t in report.requests notifications; do \
		kubectl -n $(NS) exec deploy/arch-streaming -- rpk topic create $$t -p 1 2>&1 | grep -v "TOPIC_ALREADY_EXISTS" | tail -1; \
	done
	@# mail-outbox carries one-time password links: kept 1 hour only, and not indexed into Quickwit
	@kubectl -n $(NS) exec deploy/arch-streaming -- rpk topic create mail-outbox -p 1 -c retention.ms=3600000 2>&1 | grep -v "TOPIC_ALREADY_EXISTS" | tail -1
	@$(MAKE) -s event-indexes

QUICKWIT_URL ?= http://localhost:7280
EVENT_INDEXES := events-report-requests-v1 events-notifications-v1
event-indexes: ## Index the Redpanda topics into Quickwit (native Kafka source; idempotent)
	@for i in $(EVENT_INDEXES); do \
		d=kube/arch/applications/arch_logging/indexes; \
		curl -s -o /dev/null -w "index $$i: %{http_code} (200 created, 400 already exists)\n" -X POST -H 'Content-Type: application/json' --data-binary @$$d/$$i.json $(QUICKWIT_URL)/api/v1/indexes; \
		curl -s -o /dev/null -w "source $$i: %{http_code} (200 created, 400 already exists)\n" -X POST -H 'Content-Type: application/json' --data-binary @$$d/$$i.source.json $(QUICKWIT_URL)/api/v1/indexes/$$i/sources; \
	done

run-web: ## Run web frontend (Leptos master-app, port WEB_PORT=3000); regenerates the Trunk proxy config from .env
	@echo "🌐 Starting web application (master-app)..."
	@if command -v trunk >/dev/null 2>&1; then \
		set -a; [ -f .env ] && . ./.env; set +a; \
		cd handler/web/application/master && ./gen-trunk-config.sh && trunk serve; \
	else \
		echo "ℹ️  'trunk' not found on PATH. Running via cargo run..."; \
		cargo run -p master-app; \
	fi

web: run-web ## Alias for run-web

dev-master: ## Run api-master with cargo-watch (auto-reload)
	@if command -v cargo-watch >/dev/null 2>&1; then \
		cargo watch -x 'run -p api-master'; \
	else \
		cargo run -p api-master; \
	fi

dev-report: ## Run api-report with cargo-watch (auto-reload)
	@if command -v cargo-watch >/dev/null 2>&1; then \
		cargo watch -x 'run -p api-report'; \
	else \
		cargo run -p api-report; \
	fi

check: ## Run cargo check across Rust workspace
	@cargo check

build: ## Build Rust workspace binaries
	@cargo build

# --- Code Generation Targets ---

gen-entities: ## Generate Sea-ORM entities from PostgreSQL into domain/src/entity
	@if [ -f .env ]; then \
		set -a; . ./.env; set +a; \
	fi; \
	DB_URL="$${DATABASE_URL:-postgres://postgres:postgres@localhost:5432/arch?currentSchema=arch}"; \
	DB_SCHEMA="$${DATABASE_SCHEMA:-arch}"; \
	echo "📦 Generating Sea-ORM entities for schema '$$DB_SCHEMA' into domain/src/entity..."; \
	sea-orm-cli generate entity \
		-u "$$DB_URL" \
		-s "$$DB_SCHEMA" \
		-o "domain/src/entity" \
		--with-serde both \
		--date-time-crate chrono; \
	echo "✅ Sea-ORM entities generated successfully in domain/src/entity."

gen-entity: gen-entities ## Alias for gen-entities
