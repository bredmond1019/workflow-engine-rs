# Security Policy

## Supported Versions

The AI Workflow Engine project actively maintains security updates for the following versions:

| Version | Supported          |
| ------- | ------------------ |
| 0.5.x   | :white_check_mark: |
| < 0.5   | :x:                |

## Reporting a Vulnerability

The AI Workflow Engine team takes security vulnerabilities seriously. We appreciate your efforts to responsibly disclose your findings.

### Where to Report

**Please DO NOT report security vulnerabilities through public GitHub issues.**

Instead, please report security vulnerabilities to:
- **Email**: security@workflow-engine.rs
- **Response Time**: We aim to respond within 48 hours

### What to Include

When reporting a vulnerability, please include:

1. **Description**: Clear description of the vulnerability
2. **Impact**: Potential impact and attack scenarios
3. **Affected Components**: Which crates or components are affected
4. **Reproduction Steps**: Detailed steps to reproduce the issue
5. **Proof of Concept**: Code or scripts demonstrating the vulnerability (if applicable)
6. **Affected Versions**: Which versions of the software are affected
7. **Mitigation**: Any suggested fixes or workarounds

### What to Expect

1. **Acknowledgment**: We'll acknowledge receipt within 48 hours
2. **Assessment**: We'll assess the vulnerability and its impact
3. **Updates**: We'll keep you informed of our progress
4. **Fix Development**: We'll work on a fix for confirmed vulnerabilities
5. **Disclosure**: We'll coordinate disclosure timing with you
6. **Credit**: We'll credit you for the discovery (unless you prefer to remain anonymous)

## Security Vulnerability Response

### Severity Levels

We use the following severity levels:

- **Critical**: Remote code execution, authentication bypass, data loss
- **High**: Privilege escalation, information disclosure, denial of service
- **Medium**: Limited information disclosure, limited denial of service
- **Low**: Minor security issues with minimal impact

### Response Timeline

- **Critical**: Fix within 7 days
- **High**: Fix within 14 days
- **Medium**: Fix within 30 days
- **Low**: Fix in next regular release

## Security Best Practices

When using the AI Workflow Engine:

### 1. Authentication & Authorization

- Always use strong JWT secrets (minimum 256 bits)
- Rotate JWT secrets regularly
- Implement proper role-based access control
- Never expose authentication tokens in logs

### 2. Database Security

- Use encrypted connections to PostgreSQL
- Follow principle of least privilege for database users
- Regularly update database passwords
- Enable audit logging for sensitive operations

### 3. MCP Server Security

- Run MCP servers in isolated environments
- Use TLS for all MCP communications
- Validate all inputs from external MCP servers
- Implement rate limiting on MCP endpoints

### 4. API Security

- Always use HTTPS in production
- Implement proper rate limiting
- Validate and sanitize all inputs
- Use correlation IDs for audit trails

### 5. Container Security

- Keep base images updated
- Run containers with non-root users
- Use security scanning on container images
- Implement resource limits

### 6. Secrets Management

- Never commit secrets to version control
- Use environment variables for sensitive configuration
- Consider using a secrets management service
- Rotate all secrets regularly

## Security Features

The AI Workflow Engine includes several security features:

1. **JWT Authentication**: Secure token-based authentication
2. **Rate Limiting**: Protection against denial of service
3. **Input Validation**: Comprehensive input validation
4. **Audit Logging**: Detailed logging with correlation IDs
5. **Circuit Breakers**: Protection against cascading failures
6. **TLS Support**: Encrypted communications
7. **CORS Protection**: Configurable CORS policies

## Dependencies

We regularly audit our dependencies for known vulnerabilities:

```bash
# Run security audit
cargo audit

# Check for outdated dependencies
cargo outdated
```

## Security Updates

Security updates are released as:

- **Patch Releases**: For non-breaking security fixes
- **Minor Releases**: For security fixes requiring small changes
- **Security Advisories**: Published for all security fixes

Subscribe to our security mailing list for updates:
- **Mailing List**: security-announce@workflow-engine.rs

## Compliance

The AI Workflow Engine aims to help you meet common compliance requirements:

- **Data Privacy**: No telemetry or data collection
- **Audit Trails**: Comprehensive logging capabilities
- **Access Control**: Role-based access control support
- **Encryption**: Support for encryption at rest and in transit

## Additional Resources

- [OWASP Top Ten](https://owasp.org/www-project-top-ten/)
- [Rust Security Guidelines](https://anssi-fr.github.io/rust-guide/)
- [Docker Security Best Practices](https://docs.docker.com/engine/security/)
- [PostgreSQL Security](https://www.postgresql.org/docs/current/security.html)

## Contact

For general security questions (not vulnerability reports):
- **GitHub Discussions**: Security category
- **Email**: security@workflow-engine.rs

Thank you for helping keep the AI Workflow Engine and its users safe!