# AI Workflow Engine - Open Source Readiness Task Plan

This document outlines the comprehensive task plan to prepare the AI Workflow Engine for open source release. The focus is on the core workflow engine functionality while maintaining MCP infrastructure but removing external service integrations that aren't essential for the initial release.

## 🎯 **Critical Priority (Must Fix Before Release)**

### **Task 1: Fix Compilation Errors** 
**Status:** Blocking all development
- **1.1:** Resolve utoipa-swagger-ui dependency issues in API layer
- **1.2:** Fix type mismatches and missing imports in workflow-engine-api  
- **1.3:** Resolve dependency conflicts in workflow-engine-nodes package
- **1.4:** Fix workspace dependency configuration issues

**Impact:** Without fixing these, the project cannot build or run.

### **Task 2: Remove External MCP Clients**
**Status:** Architectural cleanup required
- **2.1:** Remove Slack MCP client implementation and dependencies
- **2.2:** Remove Notion MCP client implementation and dependencies  
- **2.3:** Remove HelpScout MCP client implementation and dependencies
- **2.4:** Update documentation to reflect removal of external MCP clients

**Rationale:** These are not needed for core workflow engine functionality and add complexity.

### **Task 3: Fix Infrastructure Alignment**
**Status:** Documentation and deployment mismatch
- **3.1:** Add microservices to docker-compose.yml or update README deployment info
- **3.2:** Create missing MCP server infrastructure or update documentation
- **3.3:** Align docker-compose services with README claims

**Impact:** Users cannot follow deployment instructions successfully.

## 🔥 **High Priority (Core Functionality)**

### **Task 4: Complete Pricing Engine**
**Status:** Production implementation needed
- **4.1:** Implement live API pricing updates for OpenAI
- **4.2:** Implement live API pricing updates for Anthropic  
- **4.3:** Implement live API pricing updates for AWS Bedrock
- **4.4:** Add configuration for pricing update frequency and fallback handling

**Current State:** Uses hardcoded pricing data instead of live API calls.

### **Task 5: Fix Test Suite**
**Status:** 1,594 tests exist but cannot run
- **5.1:** Fix test compilation issues (utoipa-swagger-ui and dependency problems)
- **5.2:** Create test configuration that works without external services
- **5.3:** Document which tests require external infrastructure with setup instructions
- **5.4:** Add comprehensive API endpoint tests to fill coverage gaps
- **5.5:** Fix the 134 ignored tests by providing proper test infrastructure setup

**Expected Outcome:** 90%+ of tests should pass once compilation is fixed.

## 📚 **Medium Priority (Polish & Documentation)**

### **Task 6: Fix Documentation**
**Status:** Several broken links and missing files
- **6.1:** Create missing documentation files (DEVELOPMENT_SETUP.md, QUICK_START.md, monitoring/README.md)
- **6.2:** Sync version numbers between Cargo.toml (0.6.0) and CHANGELOG.md (0.5.0)
- **6.3:** Update README code examples to use correct import paths for workspace structure
- **6.4:** Fix broken documentation links throughout README

**Impact:** Poor developer experience and onboarding.

### **Task 7: Benchmark Validation**
**Status:** Performance claims need validation
- **7.1:** Create benchmarking framework to validate "15,000+ requests/second" claim
- **7.2:** Create benchmarks for "sub-millisecond node processing" claim
- **7.3:** Document benchmark setup and results in README

**Current Issue:** README makes performance claims without supporting data.

## 🔮 **Low Priority (Optional for v1.0)**

### **Task 8: AI Features Assessment**
**Status:** Evaluate necessity for initial release
- **8.1:** Assess if WebSocket AI streaming is needed for initial release
- **8.2:** Evaluate Gemini and Ollama provider implementations for v1.0
- **8.3:** Document which AI features are included vs roadmap items

**Decision Needed:** Determine which advanced AI features are essential vs nice-to-have.

### **Task 9: Demo Cleanup**
**Status:** Customer support workflow is intentionally basic
- **9.1:** Document customer support workflow as intentional demo/example
- **9.2:** Ensure customer support demo works reliably with rule-based implementations
- **9.3:** Add clear examples and documentation for customer support workflow

**Note:** This is working as intended - it's a demo of workflow capabilities.

## 📊 **Test Strategy Summary**

### **Current State:**
- **Total Tests:** 1,594 test functions across 178 files
- **Status:** Cannot compile due to dependency issues
- **Ignored Tests:** 134 tests requiring external infrastructure
- **Coverage:** Excellent quantitative coverage once compilation works

### **Path to Passing Tests:**

#### **Immediate Actions (Target: 90% passing)**
1. **Fix Compilation Issues**
   - Resolve utoipa-swagger-ui dependency conflicts
   - Fix workspace dependency mismatches
   - Update import paths for new crate structure

2. **Create Mock Infrastructure**
   - Mock external services for basic tests
   - Provide in-memory alternatives for database tests
   - Create test-specific configurations

#### **Short-term Actions (Target: 95% passing)**
3. **Database Test Infrastructure**
   - Set up PostgreSQL test database
   - Create test data fixtures
   - Add database cleanup between tests

4. **Integration Test Environment**
   - Docker test environment setup
   - MCP test server configuration
   - API endpoint integration tests

#### **Long-term Actions (Target: 99% passing)**
5. **Full Infrastructure Tests**
   - Redis integration for caching tests
   - External service simulation
   - Performance and load test infrastructure

### **Test Categories and Expected Status:**

| Test Category | Count | Expected Status | Dependencies |
|---------------|--------|----------------|--------------|
| Core Workflow Engine | ~400 | ✅ Should pass | None |
| Event Sourcing | ~300 | ✅ Should pass | PostgreSQL |
| MCP Protocol | ~200 | ✅ Should pass | Mock servers |
| API Endpoints | ~100 | ⚠️ Needs work | Web framework |
| Database Operations | ~300 | ✅ Should pass | PostgreSQL |
| Authentication | ~50 | ✅ Should pass | None |
| Monitoring | ~100 | ✅ Should pass | None |
| External Services | ~144 | ❌ Will ignore | External APIs |

## 🎯 **Success Criteria**

### **Minimum Viable Open Source Release:**
- ✅ Project compiles without errors
- ✅ Core workflow engine tests pass (estimated 800+ tests)
- ✅ Basic API functionality works end-to-end
- ✅ Documentation is accurate and matches implementation
- ✅ Docker setup works as documented
- ✅ Installation guide can be followed successfully

### **Ideal Open Source Release:**
- ✅ 95%+ tests passing (excluding external service tests)
- ✅ Performance benchmarks validated and documented
- ✅ Comprehensive developer documentation
- ✅ All infrastructure properly configured
- ✅ CI/CD pipeline working
- ✅ Community contribution guidelines in place

## 📈 **Implementation Phases**

### **Phase 1: Foundation (Week 1)**
- Fix all compilation errors
- Remove external MCP clients
- Get basic test suite running

### **Phase 2: Core Features (Week 2)**
- Complete pricing engine implementation
- Fix infrastructure alignment
- Get 90% of tests passing

### **Phase 3: Polish (Week 3)**
- Fix documentation issues
- Add benchmark validation
- Comprehensive testing

### **Phase 4: Release Preparation (Week 4)**
- Final testing and validation
- Community preparation
- Release process setup

## 🚫 **Explicitly Out of Scope**

The following items are **intentionally excluded** from the initial open source release:

1. **External Service Integrations:** Slack, Notion, HelpScout clients
2. **Production AI Features:** Customer support AI is intentionally demo-level
3. **Advanced Model Providers:** Gemini, Ollama can wait for post-v1.0
4. **Complex Streaming:** WebSocket AI streaming is optional
5. **Enterprise Features:** Multi-tenancy can be basic for initial release

## 📋 **Next Steps**

1. **Start with Task 1:** Fix compilation errors to unblock development
2. **Parallel execution:** Tasks 2 and 3 can be done simultaneously with Task 1
3. **Test early and often:** Set up basic test running as soon as compilation works
4. **Documentation updates:** Keep documentation in sync with code changes
5. **Community preparation:** Begin preparing contribution guidelines and issue templates

This plan prioritizes getting a solid, working open source workflow engine that demonstrates the architecture and capabilities while being realistic about what can be completed for an initial release.