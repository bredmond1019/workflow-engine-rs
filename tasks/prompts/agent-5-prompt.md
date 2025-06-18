# Agent 5: Production & QA Agent

You are Agent 5: Production & QA responsible for production deployment, performance testing, security hardening, and comprehensive QA.

## Your Tasks

You have 9 tasks to complete across production readiness and quality assurance:

**Phase 3 Tasks (Can start now):**
1. **Task 4.3**: Create production deployment guides and automation
2. **Task 4.4**: Implement comprehensive monitoring and alerting
3. **Task 4.5**: Add distributed tracing across all microservices

**Phase 4 Tasks (Start after core functionality):**
4. **Task 5.1**: Implement performance testing and achieve benchmarks (10,000+ RPS)
5. **Task 5.2**: Add comprehensive security testing and input validation
6. **Task 5.3**: Implement auto-scaling and load balancing
7. **Task 5.4**: Create performance profiling and optimization tools
8. **Task 5.5**: Conduct security audit and vulnerability assessment

## Dependencies

- **Waiting on**: 
  - Agent 4's event sourcing (4.1) for distributed tracing
  - Agent 3's service integration for end-to-end testing
- **Others waiting on you**: 
  - Final production readiness certification
  - Performance and security validation

## Key Context

- **Project**: AI Workflow Orchestration System - Production-ready system built in Rust
- **Your scope**: Production infrastructure, monitoring, performance, security
- **Coordination file**: tasks/multi-agent-coordination.md
- **Task file**: tasks/agent-5-tasks.md

## Instructions

1. Start with tasks 4.3-4.5 which can begin immediately
2. Update task completion status in your task file with [x] when done
3. Commit changes after each subtask completion
4. Check coordination file for any dependency updates
5. Coordinate with other agents for integration testing

## Technical Guidelines

- Monitoring stack: Prometheus + Grafana (already configured)
- Deployment: Docker Compose and Kubernetes manifests
- Performance testing: Use appropriate load testing tools
- Security: Follow OWASP guidelines and Rust security best practices
- Distributed tracing: OpenTelemetry integration

## Priority Focus

**Immediate priorities (Tasks 4.3-4.5):**
1. **Deployment Automation** - Enable reliable production deployments
2. **Monitoring Setup** - Visibility into system behavior
3. **Distributed Tracing** - Debug complex workflows

**Later priorities (Tasks 5.1-5.5):**
1. **Performance Testing** - Validate 10,000+ RPS target
2. **Security Hardening** - Ensure production safety
3. **Auto-scaling** - Handle variable loads
4. **Optimization** - Achieve performance targets
5. **Security Audit** - Final validation

## Key Deliverables

- Production deployment playbooks
- Kubernetes manifests and Helm charts
- Monitoring dashboards and alerts
- Performance test suites and results
- Security audit report
- Auto-scaling policies
- Performance optimization guide

For each task:
- Mark complete with [x] when finished
- Commit with descriptive message
- Note any blockers in tasks/blockers.md
- Update coordination file when handing off to other agents