# Security Guide

This document outlines the security practices, policies, and maintenance procedures for the AI Workflow Engine project.

## Table of Contents

- [Security Overview](#security-overview)
- [Dependency Security](#dependency-security)
- [Security Scanning](#security-scanning)
- [Security Update Process](#security-update-process)
- [Vulnerability Reporting](#vulnerability-reporting)
- [Security Best Practices](#security-best-practices)
- [Compliance](#compliance)

## Security Overview

The AI Workflow Engine is designed with security as a core principle. This includes:

- **Zero-trust architecture** with JWT-based authentication
- **Input validation** at all API boundaries
- **Rate limiting** to prevent abuse
- **Secure TLS communication** using modern cipher suites
- **Regular security audits** of dependencies
- **Automated vulnerability scanning** in CI/CD pipelines

## Dependency Security

### Current Security Status

✅ **Zero security vulnerabilities** - As of the last audit, all dependencies are free of known security vulnerabilities.

✅ **License compliance** - All dependencies use approved open-source licenses.

⚠️ **Deprecated packages** - Two deprecated packages identified:
- `serde_yaml v0.9.34+deprecated` - Functional but deprecated. Migration planned to `serde_yml`.
- `actix-web-actors v4.3.1+deprecated` - Still maintained for security fixes.

### Security Scanning Tools

The project uses multiple tools for comprehensive security scanning:

1. **cargo-audit** - Scans for known security vulnerabilities
2. **cargo-deny** - License compliance and dependency policy enforcement
3. **GitHub Dependency Scanning** - Automated vulnerability detection
4. **Dependabot** - Automated dependency updates

### Approved Licenses

The following licenses are explicitly allowed:

- MIT
- Apache-2.0
- Apache-2.0 WITH LLVM-exception
- BSD-2-Clause
- BSD-3-Clause
- ISC
- Unicode-DFS-2016
- Unicode-3.0
- Zlib

Copyleft licenses (GPL, AGPL) are not permitted to maintain commercial flexibility.

## Security Scanning

### Automated Scanning

Security scanning is automated through GitHub Actions workflows:

#### Daily Security Audit
- **Schedule**: Daily at 2 AM UTC
- **Triggers**: Changes to `Cargo.toml` or `Cargo.lock`
- **Actions**: 
  - Security vulnerability scanning
  - License compliance checking
  - Outdated dependency reporting
  - Supply chain security verification

#### CI/CD Integration
All pull requests and commits to main branches trigger:
- `cargo audit` for vulnerability scanning
- `cargo deny check` for policy compliance
- Dependency graph submission to GitHub Security

### Manual Security Checks

Run security audits manually:

```bash
# Quick security check (recommended)
./scripts/security-update.sh check-only

# Comprehensive security audit with detailed report
./scripts/security-audit.sh

# Individual commands
cargo audit                    # Check for security vulnerabilities
cargo audit --stale          # Check for unmaintained dependencies
cargo deny check              # Comprehensive policy check
cargo outdated               # Check for outdated dependencies (requires cargo-outdated)
```

### Automated Security Scripts

The project includes two automated security scripts:

#### Security Update Script (`scripts/security-update.sh`)
Safely updates dependencies and runs security checks:

```bash
# Examples
./scripts/security-update.sh                    # Patch updates with tests
./scripts/security-update.sh minor              # Minor updates with tests  
./scripts/security-update.sh check-only         # Security check only
./scripts/security-update.sh patch skip-tests   # Patch updates without tests
```

#### Security Audit Script (`scripts/security-audit.sh`)
Generates comprehensive security reports:

```bash
./scripts/security-audit.sh   # Generates detailed report in target/security-reports/
```

### Security Monitoring

The project monitors:
- **RustSec Advisory Database** for new vulnerabilities
- **GitHub Security Advisories** for ecosystem threats
- **Dependency maintenance status** for supply chain risks

## Security Update Process

### Immediate Response (Critical Vulnerabilities)
1. **Detection**: Automated alerts via GitHub Security or manual discovery
2. **Assessment**: Evaluate impact and affected components
3. **Mitigation**: Apply patches or workarounds within 24 hours
4. **Testing**: Comprehensive testing of security fixes
5. **Deployment**: Emergency release with security patches
6. **Communication**: Security advisory and changelog update

### Regular Maintenance (Non-Critical)
1. **Weekly Review**: Check for new advisories and updates
2. **Monthly Updates**: Batch update non-breaking dependency updates
3. **Quarterly Audit**: Comprehensive security review and dependency cleanup
4. **Documentation**: Update security documentation as needed

### Update Commands

```bash
# Update all dependencies to latest compatible versions
cargo update

# Check for breaking changes in major updates
cargo outdated --depth 1

# Update workspace dependencies
cargo upgrade --workspace

# Verify security after updates
cargo audit && cargo deny check
```

### Deprecated Package Migration Plan

| Package | Status | Migration Plan | Timeline |
|---------|--------|---------------|----------|
| `serde_yaml` | Deprecated | Migrate to `serde_yml` | Q1 2024 |
| `actix-web-actors` | Deprecated | Evaluate alternatives or maintain current | Q2 2024 |

## Vulnerability Reporting

### Reporting Security Issues

**Do NOT report security vulnerabilities through public GitHub issues.**

Instead, report security vulnerabilities privately:

1. **Email**: security@workflow-engine.dev
2. **GitHub Security**: Use GitHub's private vulnerability reporting
3. **GPG Key**: Available for encrypted communications

### Information to Include

When reporting a vulnerability, please include:

- Description of the vulnerability
- Steps to reproduce the issue
- Potential impact assessment
- Suggested mitigation (if any)
- Your contact information

### Response Timeline

- **Acknowledgment**: Within 24 hours
- **Initial Assessment**: Within 72 hours
- **Status Updates**: Weekly during investigation
- **Resolution**: Target 30 days for non-critical, 7 days for critical

## Security Best Practices

### Development Practices

1. **Dependency Management**
   - Pin dependency versions in production
   - Regular security audits
   - Minimal dependency footprint
   - Prefer well-maintained crates

2. **Code Security**
   - Input validation at boundaries
   - Proper error handling
   - Secure secret management
   - SQL injection prevention

3. **Authentication & Authorization**
   - JWT with secure secrets
   - Role-based access control
   - Rate limiting
   - Session management

### Deployment Security

1. **Infrastructure**
   - TLS 1.3 for all communications
   - Container security scanning
   - Network segmentation
   - Regular security updates

2. **Secrets Management**
   - Environment variable injection
   - Encrypted at rest and in transit
   - Regular rotation
   - Principle of least privilege

### Monitoring and Logging

1. **Security Monitoring**
   - Correlation ID tracking
   - Structured logging
   - Anomaly detection
   - Security metrics

2. **Incident Response**
   - Automated alerting
   - Incident playbooks
   - Post-incident reviews
   - Documentation updates

## Compliance

### Security Standards

The project follows these security standards:

- **OWASP Top 10** - Web application security
- **NIST Cybersecurity Framework** - Overall security posture
- **CIS Controls** - Security configuration guidelines
- **Supply Chain Security** - SLSA framework adoption

### Audit Requirements

- **Quarterly** internal security reviews
- **Annual** third-party security assessment
- **Continuous** automated vulnerability scanning
- **Real-time** dependency monitoring

### Documentation Maintenance

Security documentation is reviewed and updated:
- **Monthly** for accuracy and completeness
- **After incidents** to incorporate lessons learned
- **With major releases** to reflect new security features
- **Upon policy changes** to maintain compliance

## Contact Information

- **Security Team**: security@workflow-engine.dev
- **General Issues**: https://github.com/bredmond1019/workflow-engine-rs/issues
- **Documentation**: https://github.com/bredmond1019/workflow-engine-rs/docs

---

**Last Updated**: 2024-06-24  
**Review Schedule**: Monthly  
**Next Review**: 2024-07-24